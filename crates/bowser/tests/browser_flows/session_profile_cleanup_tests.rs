use std::sync::Arc;
use std::time::Duration;

use bowser::{FileSessionStore, SessionMetadata, SessionStore, cleanup_expired_sessions};
use chrono::{Duration as ChronoDuration, Utc};
use tempfile::tempdir;

#[tokio::test]
async fn expiry_removes_owned_ephemeral_profile() {
    let session_dir = tempdir().expect("session dir");
    let profile = session_dir
        .path()
        .join("profiles")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&profile).expect("profile directory");
    std::fs::write(profile.join("marker"), "owned").expect("profile marker");
    let store: Arc<dyn SessionStore> =
        Arc::new(FileSessionStore::new(session_dir.path().to_path_buf()));
    let mut metadata = expired_metadata(profile.clone());
    metadata.owns_user_data_dir = true;
    store.save(&metadata).await.expect("save session");

    cleanup_expired_sessions(store.clone(), session_dir.path(), Duration::from_secs(60))
        .await
        .expect("cleanup sessions");

    assert!(!profile.exists());
    assert!(store.load(&metadata.id).await.is_err());
}

#[tokio::test]
async fn expiry_preserves_unowned_profile() {
    let session_dir = tempdir().expect("session dir");
    let external_dir = tempdir().expect("external profile parent");
    let profile = external_dir.path().join("explicit-profile");
    std::fs::create_dir_all(&profile).expect("profile directory");
    let store: Arc<dyn SessionStore> =
        Arc::new(FileSessionStore::new(session_dir.path().to_path_buf()));
    let metadata = expired_metadata(profile.clone());
    store.save(&metadata).await.expect("save session");

    cleanup_expired_sessions(store.clone(), session_dir.path(), Duration::from_secs(60))
        .await
        .expect("cleanup sessions");

    assert!(profile.exists());
    assert!(store.load(&metadata.id).await.is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn cleanup_rejects_a_symlinked_profiles_root() {
    let session_dir = tempdir().expect("session dir");
    let external_dir = tempdir().expect("external profile root");
    let profile_id = uuid::Uuid::new_v4().to_string();
    let external_profile = external_dir.path().join(&profile_id);
    std::fs::create_dir_all(&external_profile).expect("external profile");
    std::fs::write(external_profile.join("marker"), "keep").expect("external marker");
    std::os::unix::fs::symlink(external_dir.path(), session_dir.path().join("profiles"))
        .expect("profiles symlink");
    let profile = session_dir.path().join("profiles").join(profile_id);
    let mut metadata = expired_metadata(profile);
    metadata.owns_user_data_dir = true;

    let result = bowser::remove_owned_session_profile(session_dir.path(), &metadata).await;

    assert!(matches!(
        result,
        Err(bowser::Error::InvalidOwnedProfilePath { .. })
    ));
    assert!(external_profile.join("marker").exists());
}

fn expired_metadata(profile: std::path::PathBuf) -> SessionMetadata {
    let mut metadata = SessionMetadata::new(
        "http://127.0.0.1:65534".to_string(),
        "ws://127.0.0.1:65534/devtools/browser/test".to_string(),
        u32::MAX,
        profile,
    );
    metadata.updated_at = Utc::now() - ChronoDuration::minutes(10);
    metadata
}
