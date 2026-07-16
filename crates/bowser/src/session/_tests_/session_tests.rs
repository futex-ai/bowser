//! File-backed session persistence and cleanup tests.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use tempfile::tempdir;

use crate::{Error, SessionPageType};

use super::cleanup::xvfb_cmdline_matches;
use super::metadata::LEGACY_PAGE_ID;
pub(crate) use super::store::SessionStoreMock;
use super::{FileSessionStore, SessionMetadata, SessionStore, cleanup_expired_sessions};

#[tokio::test]
async fn file_store_round_trips_metadata() {
    let dir = tempdir().expect("tempdir");
    let store = FileSessionStore::new(dir.path().to_path_buf());
    let metadata = SessionMetadata::new(
        "http://127.0.0.1:9222".to_string(),
        "ws://127.0.0.1:9222/devtools/browser/test".to_string(),
        4242,
        PathBuf::from("/tmp/bowser-profile"),
    );

    store.save(&metadata).await.expect("save metadata");
    let loaded = store.load(&metadata.id).await.expect("load metadata");

    assert_eq!(loaded.id, metadata.id);
    assert_eq!(loaded.pid, 4242);
    assert_eq!(loaded.http_url, "http://127.0.0.1:9222");
}

#[tokio::test]
async fn file_store_rejects_parent_directory_session_ids() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path().join("sessions");
    let store = FileSessionStore::new(root);
    let outside_path = dir.path().join("outside.json");
    let mut metadata = SessionMetadata::new(
        "http://127.0.0.1:9222".to_string(),
        "ws://127.0.0.1:9222/devtools/browser/test".to_string(),
        4242,
        PathBuf::from("/tmp/bowser-profile"),
    );
    metadata.id = "../outside".to_string();
    std::fs::write(
        &outside_path,
        serde_json::to_string(&metadata).expect("serialize metadata"),
    )
    .expect("write outside metadata");

    let load_result = store.load("../outside").await;
    let remove_result = store.remove("../outside").await;

    assert!(matches!(
        load_result,
        Err(Error::InvalidSessionId { session_id }) if session_id == "../outside"
    ));
    assert!(matches!(
        remove_result,
        Err(Error::InvalidSessionId { session_id }) if session_id == "../outside"
    ));
    assert!(outside_path.exists());
}

#[tokio::test]
async fn file_store_rejects_invalid_metadata_ids_before_saving() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path().join("sessions");
    let store = FileSessionStore::new(root);
    let mut metadata = SessionMetadata::new(
        "http://127.0.0.1:9222".to_string(),
        "ws://127.0.0.1:9222/devtools/browser/test".to_string(),
        4242,
        PathBuf::from("/tmp/bowser-profile"),
    );
    metadata.id = "../escaped".to_string();

    let result = store.save(&metadata).await;

    assert!(matches!(
        result,
        Err(Error::InvalidSessionId { session_id }) if session_id == "../escaped"
    ));
    assert!(!dir.path().join("escaped.json").exists());
}

#[tokio::test]
async fn file_store_rejects_metadata_whose_id_does_not_match_its_filename() {
    let dir = tempdir().expect("tempdir");
    let store = FileSessionStore::new(dir.path().to_path_buf());
    let metadata = SessionMetadata::new(
        "http://127.0.0.1:9222".to_string(),
        "ws://127.0.0.1:9222/devtools/browser/test".to_string(),
        4242,
        PathBuf::from("/tmp/bowser-profile"),
    );
    std::fs::write(
        dir.path().join("bsr_expected.json"),
        serde_json::to_string(&metadata).expect("serialize metadata"),
    )
    .expect("write mismatched metadata");

    let result = store.load("bsr_expected").await;

    assert!(matches!(
        result,
        Err(Error::SessionIdMismatch {
            expected_session_id,
            actual_session_id,
        }) if expected_session_id == "bsr_expected" && actual_session_id == metadata.id
    ));
}

#[tokio::test]
async fn cleanup_removes_expired_sessions_and_keeps_recent_ones() {
    let dir = tempdir().expect("tempdir");
    let store: Arc<dyn SessionStore> = Arc::new(FileSessionStore::new(dir.path().to_path_buf()));
    let mut expired = SessionMetadata::new(
        "http://127.0.0.1:9223".to_string(),
        "ws://127.0.0.1:9223/devtools/browser/expired".to_string(),
        1111,
        PathBuf::from("/tmp/expired-profile"),
    );
    expired.updated_at = Utc::now() - ChronoDuration::minutes(10);

    let mut fresh = SessionMetadata::new(
        "http://127.0.0.1:9224".to_string(),
        "ws://127.0.0.1:9224/devtools/browser/fresh".to_string(),
        2222,
        PathBuf::from("/tmp/fresh-profile"),
    );
    fresh.updated_at = Utc::now();

    store.save(&expired).await.expect("save expired");
    store.save(&fresh).await.expect("save fresh");

    let summaries = cleanup_expired_sessions(store.clone(), dir.path(), Duration::from_secs(60))
        .await
        .expect("cleanup");

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].id, fresh.id);
    assert!(store.load(&expired.id).await.is_err());
    assert!(store.load(&fresh.id).await.is_ok());
}

#[test]
fn xvfb_cleanup_validates_program_and_display() {
    let cmdline = vec![
        "/usr/bin/Xvfb".to_string(),
        ":99".to_string(),
        "-screen".to_string(),
        "0".to_string(),
    ];
    let chrome_cmdline = vec!["/usr/bin/google-chrome".to_string(), ":99".to_string()];
    let other_display = vec!["/usr/bin/Xvfb".to_string(), ":100".to_string()];

    assert!(xvfb_cmdline_matches(&cmdline, ":99"));
    assert!(!xvfb_cmdline_matches(&chrome_cmdline, ":99"));
    assert!(!xvfb_cmdline_matches(&other_display, ":99"));
}

#[tokio::test]
async fn loading_legacy_metadata_migrates_the_single_page_shape() {
    let dir = tempdir().expect("tempdir");
    let store = FileSessionStore::new(dir.path().to_path_buf());
    let path = store.session_path("bsr_legacy").expect("session path");
    let now = Utc::now().to_rfc3339();
    let legacy = format!(
        r#"{{
  "id": "bsr_legacy",
  "created_at": "{now}",
  "updated_at": "{now}",
  "http_url": "http://127.0.0.1:9222",
  "websocket_url": "ws://127.0.0.1:9222/devtools/browser/test",
  "pid": 4242,
  "user_data_dir": "/tmp/bowser-profile",
  "preview_capture": {{
    "url": "https://example.com/",
    "title": "Example Domain",
    "children": []
  }},
  "full_capture": {{
    "url": "https://example.com/",
    "title": "Example Domain",
    "children": []
  }},
  "metadata_records": [],
  "requires_fresh_capture": true
}}"#
    );
    std::fs::write(&path, legacy).expect("write legacy metadata");

    let metadata = store
        .load("bsr_legacy")
        .await
        .expect("load migrated metadata");

    assert_eq!(metadata.selected_page_id.as_deref(), Some(LEGACY_PAGE_ID));
    assert_eq!(metadata.pages.len(), 1);
    assert_eq!(metadata.pages[0].id, LEGACY_PAGE_ID);
    assert_eq!(metadata.pages[0].page_type, SessionPageType::Tab);
    assert_eq!(
        metadata.pages[0].url().as_deref(),
        Some("https://example.com/")
    );
    assert!(metadata.pages[0].requires_fresh_capture);
    assert_eq!(metadata.next_page_ordinal, 2);
    assert!(!metadata.owns_user_data_dir);
}

#[tokio::test]
async fn listing_legacy_metadata_uses_the_same_migration_as_loading() {
    let dir = tempdir().expect("tempdir");
    let store = FileSessionStore::new(dir.path().to_path_buf());
    let path = store.session_path("bsr_legacy").expect("session path");
    let now = Utc::now().to_rfc3339();
    let legacy = format!(
        r#"{{
  "id": "bsr_legacy",
  "created_at": "{now}",
  "updated_at": "{now}",
  "http_url": "http://127.0.0.1:9222",
  "websocket_url": "ws://127.0.0.1:9222/devtools/browser/test",
  "pid": 4242,
  "user_data_dir": "/tmp/bowser-profile",
  "preview_capture": null,
  "full_capture": null,
  "metadata_records": [],
  "requires_fresh_capture": true
}}"#
    );
    std::fs::write(&path, legacy).expect("write legacy metadata");

    let sessions = store.list().await.expect("list migrated metadata");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "bsr_legacy");
    assert_eq!(
        sessions[0].selected_page_id.as_deref(),
        Some(LEGACY_PAGE_ID)
    );
    assert_eq!(sessions[0].pages.len(), 1);
    assert_eq!(sessions[0].pages[0].id, LEGACY_PAGE_ID);
}
