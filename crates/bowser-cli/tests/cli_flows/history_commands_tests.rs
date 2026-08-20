use std::process::Output;

use serde_json::Value;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn history_commands_emit_captures_and_typed_exhaustion() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let chrome = support::chrome_path();
    let session_root = session_dir.path().to_str().expect("session dir");
    let chrome_path = chrome.to_str().expect("chrome path");

    let first = support::run_bowser_with_retry(
        &[
            "--json-envelope",
            "--no-ai",
            "--chrome-path",
            chrome_path,
            "--session-dir",
            session_root,
            "get",
            "--format",
            "json",
            &server.url("/simple"),
        ],
        5,
    );
    let first_envelope = assert_single_envelope(&first, true, "first get");
    let session_id = first_envelope["session"]
        .as_str()
        .expect("session id")
        .to_string();

    let second = support::run_bowser_with_retry(
        &[
            "--json-envelope",
            "--no-ai",
            "--chrome-path",
            chrome_path,
            "--session-dir",
            session_root,
            "--session",
            &session_id,
            "get",
            "--format",
            "json",
            &server.url("/long"),
        ],
        5,
    );
    assert_single_envelope(&second, true, "second get");

    let exhausted = history_command(session_root, &session_id, "forward");
    let exhausted_envelope = assert_single_envelope(&exhausted, false, "exhausted forward");
    assert_eq!(exhausted_envelope["error"]["code"], "history_exhausted");
    assert_eq!(
        exhausted_envelope["error"]["detail"]["direction"],
        "forward"
    );

    let back = history_command(session_root, &session_id, "back");
    let back_envelope = assert_single_envelope(&back, true, "back");
    assert_eq!(
        back_envelope["result"]["capture"]["title"],
        "Fixture Simple"
    );

    let forward = history_command(session_root, &session_id, "forward");
    let forward_envelope = assert_single_envelope(&forward, true, "forward");
    assert_eq!(
        forward_envelope["result"]["capture"]["title"],
        "Fixture Long"
    );

    let reload = history_command(session_root, &session_id, "reload");
    let reload_envelope = assert_single_envelope(&reload, true, "reload");
    assert_eq!(
        reload_envelope["result"]["capture"]["title"],
        "Fixture Long"
    );
}

fn history_command(session_root: &str, session_id: &str, command: &str) -> Output {
    support::run_bowser_with_retry(
        &[
            "--json-envelope",
            "--no-ai",
            "--session-dir",
            session_root,
            "--session",
            session_id,
            command,
            "--format",
            "json",
        ],
        5,
    )
}

fn assert_single_envelope(output: &Output, expected_success: bool, step: &str) -> Value {
    let stdout = std::str::from_utf8(&output.stdout).expect("UTF-8 stdout");
    let envelope: Value = serde_json::from_str(stdout).expect("single JSON document");
    assert_eq!(
        output.status.success(),
        expected_success,
        "unexpected {step} status; envelope={envelope}; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout.trim().lines().count(), 1);
    assert_eq!(envelope["ok"], expected_success);
    envelope
}
