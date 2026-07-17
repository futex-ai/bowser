use std::ffi::OsStr;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::process::{Command as ProcessCommand, Stdio};
#[cfg(unix)]
use std::time::Duration;

use super::support;

#[test]
fn bowser_command_forces_headless_browser_defaults() {
    let command = support::bowser_command();
    let envs: Vec<(&OsStr, Option<&OsStr>)> = command.get_envs().collect();

    assert!(envs.contains(&(OsStr::new("BOWSER_HEADLESS"), Some(OsStr::new("true")))));
    assert!(envs.contains(&(
        OsStr::new("BOWSER_INTERNAL_STEALTH_FEATURES"),
        Some(OsStr::new("-launch-headed,-launch-native-window"))
    )));
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_session_directory_closes_detached_browsers_on_drop() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            support::chrome_path().to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/simple"),
        ])
        .output()
        .expect("run bowser get");
    assert!(output.status.success());
    let session_id =
        support::parse_session_id(std::str::from_utf8(&output.stderr).expect("utf-8 stderr"));
    let metadata: bowser::SessionMetadata = serde_json::from_slice(
        &fs::read(session_dir.path().join(format!("{session_id}.json"))).expect("session metadata"),
    )
    .expect("decode session metadata");

    drop(session_dir);

    for _ in 0..20 {
        if !process_is_alive(metadata.pid) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!process_is_alive(metadata.pid));
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    ProcessCommand::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
