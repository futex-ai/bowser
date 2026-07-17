use std::fs;
use std::path::Path;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_reports_resumable_session_when_output_write_fails() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let missing_output = session_dir.path().join("missing/output.yaml");

    let output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/simple"),
            "--output",
            missing_output.to_str().expect("output path"),
        ])
        .output()
        .expect("run failing bowser get");
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(!output.status.success());
    assert!(
        stderr.lines().any(|line| line.starts_with("Session: ")),
        "failed get did not report a resumable session: {stderr}"
    );
    let session_id = support::parse_session_id(&stderr);
    assert!(
        session_dir
            .path()
            .join(format!("{session_id}.json"))
            .exists(),
        "reported session metadata was not stored: {stderr}"
    );

    let close_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "session",
            "close",
            &session_id,
        ])
        .output()
        .expect("close failed get session");

    assert!(close_output.status.success());
    assert!(!has_profile_children(session_dir.path()));
    assert!(
        stderr.contains(&format!("Session: {session_id}")),
        "{stderr}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resumed_commands_detach_when_operations_or_outputs_fail() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let session_dir_arg = session_dir.path().to_str().expect("session dir");
    let chrome_path_arg = chrome_path.to_str().expect("chrome path");

    let get_output = support::run_bowser_with_retry(
        &[
            "--no-ai",
            "--chrome-path",
            chrome_path_arg,
            "--session-dir",
            session_dir_arg,
            "get",
            &server.url("/simple"),
        ],
        5,
    );
    assert!(get_output.status.success());
    let session_id =
        support::parse_session_id(&String::from_utf8(get_output.stderr).expect("stderr"));
    let missing_output = session_dir.path().join("missing/output.yaml");
    let missing_output_arg = missing_output.to_str().expect("missing output");

    let capture_output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            chrome_path_arg,
            "--session-dir",
            session_dir_arg,
            "--session",
            &session_id,
            "capture",
            "--output",
            missing_output_arg,
        ])
        .output()
        .expect("run failing capture");
    let capture_pages_stale = all_pages_stale(session_dir.path(), &session_id);

    let scroll_output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            chrome_path_arg,
            "--session-dir",
            session_dir_arg,
            "--session",
            &session_id,
            "scroll",
            "--direction",
            "down",
            "--output",
            missing_output_arg,
        ])
        .output()
        .expect("run failing scroll");
    let scroll_pages_stale = all_pages_stale(session_dir.path(), &session_id);

    let before_page_failure = load_metadata(session_dir.path(), &session_id).updated_at;
    let page_output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            chrome_path_arg,
            "--session-dir",
            session_dir_arg,
            "--session",
            &session_id,
            "page",
            "select",
            "pg_missing",
        ])
        .output()
        .expect("run failing page select");
    let after_page_failure = load_metadata(session_dir.path(), &session_id).updated_at;

    let close_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir_arg,
            "session",
            "close",
            &session_id,
        ])
        .output()
        .expect("close resumed session");

    assert!(!capture_output.status.success());
    assert!(capture_pages_stale, "capture failure skipped detach");
    assert!(!scroll_output.status.success());
    assert!(scroll_pages_stale, "interaction failure skipped detach");
    assert!(!page_output.status.success());
    assert!(
        after_page_failure > before_page_failure,
        "page operation failure skipped detach"
    );
    assert!(close_output.status.success());
}

fn load_metadata(session_dir: &Path, session_id: &str) -> bowser::SessionMetadata {
    let path = session_dir.join(format!("{session_id}.json"));
    let contents = fs::read_to_string(path).expect("read session metadata");
    serde_json::from_str(&contents).expect("parse session metadata")
}

fn all_pages_stale(session_dir: &Path, session_id: &str) -> bool {
    let metadata = load_metadata(session_dir, session_id);
    !metadata.pages.is_empty()
        && metadata
            .pages
            .iter()
            .all(|page| page.requires_fresh_capture)
}

fn has_profile_children(session_dir: &std::path::Path) -> bool {
    let profiles = session_dir.join("profiles");
    profiles.exists()
        && fs::read_dir(profiles)
            .expect("read profiles directory")
            .next()
            .is_some()
}
