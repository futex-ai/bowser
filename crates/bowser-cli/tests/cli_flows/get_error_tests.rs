use std::fs;

use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_reports_resumable_session_when_output_write_fails() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
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

fn has_profile_children(session_dir: &std::path::Path) -> bool {
    let profiles = session_dir.join("profiles");
    profiles.exists()
        && fs::read_dir(profiles)
            .expect("read profiles directory")
            .next()
            .is_some()
}
