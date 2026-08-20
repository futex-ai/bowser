use std::process::Output;

use serde_json::Value;

use super::support;

#[test]
fn capabilities_and_argument_errors_are_single_json_documents() {
    let capabilities = support::bowser_command()
        .args(["--json-envelope", "capabilities"])
        .output()
        .expect("run capabilities");
    let envelope = assert_single_envelope(&capabilities, true);
    assert_eq!(envelope["envelope"], 1);
    assert_eq!(
        envelope["result"]["envelope_versions"],
        serde_json::json!([1])
    );
    assert_eq!(
        envelope["result"]["checkpoint_versions"],
        serde_json::json!([1])
    );
    assert_eq!(envelope["result"]["features"]["checkpoint"], true);
    assert_eq!(envelope["result"]["features"]["kiosk"], false);
    assert_eq!(envelope["result"]["features"]["kiosk_launch"], true);
    assert_eq!(envelope["result"]["features"]["history"], true);
    assert_eq!(envelope["result"]["features"]["live_inventory"], true);
    assert_eq!(envelope["result"]["features"]["window_target"], false);

    let invalid = support::bowser_command()
        .args(["--json-envelope", "capture", "--unknown-option"])
        .output()
        .expect("run invalid command");
    let envelope = assert_single_envelope(&invalid, false);
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(envelope["error"]["code"], "invalid_arguments");
}

#[test]
fn runtime_failures_include_known_context_in_one_json_document() {
    let output = support::bowser_command()
        .args([
            "--json-envelope",
            "--session",
            "bsr_missing",
            "capture",
            "--format",
            "json",
        ])
        .output()
        .expect("run missing session capture");
    let envelope = assert_single_envelope(&output, false);
    assert_eq!(envelope["session"], "bsr_missing");
    assert_eq!(envelope["page"], Value::Null);
    assert_eq!(envelope["result"], Value::Null);
    assert_eq!(envelope["error"]["code"], "session_not_found");
    assert_eq!(envelope["error"]["detail"]["session_id"], "bsr_missing");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_commands_and_checkpoint_round_trip_use_machine_envelopes() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let source_dir = support::test_session_dir();
    let checkpoint_dir = tempfile::tempdir().expect("checkpoint dir");
    let checkpoint = checkpoint_dir.path().join("session.json");
    let chrome = support::chrome_path();

    let get = support::run_bowser_with_retry(
        &[
            "--json-envelope",
            "--no-ai",
            "--chrome-path",
            chrome.to_str().expect("chrome path"),
            "--session-dir",
            source_dir.path().to_str().expect("source dir"),
            "get",
            "--format",
            "json",
            &server.url("/counter"),
        ],
        5,
    );
    let get_envelope = assert_single_envelope(&get, true);
    let source_session = get_envelope["session"]
        .as_str()
        .expect("source session")
        .to_string();
    assert!(get_envelope["page"].as_str().is_some());
    assert_eq!(
        get_envelope["result"]["capture"]["title"],
        "Counter Fixture"
    );

    let export = support::bowser_command()
        .args([
            "--json-envelope",
            "--chrome-path",
            chrome.to_str().expect("chrome path"),
            "--session-dir",
            source_dir.path().to_str().expect("source dir"),
            "--session",
            &source_session,
            "session",
            "export",
            "--to",
            checkpoint.to_str().expect("checkpoint path"),
        ])
        .output()
        .expect("export checkpoint");
    let export_envelope = assert_single_envelope(&export, true);
    assert_eq!(export_envelope["result"]["checkpoint"]["version"], 1);
    assert_eq!(export_envelope["result"]["checkpoint"]["pages"], 1);
    assert!(checkpoint.is_file());

    let info = support::bowser_command()
        .args([
            "--json-envelope",
            "--session-dir",
            source_dir.path().to_str().expect("source dir"),
            "session",
            "info",
            &source_session,
        ])
        .output()
        .expect("inspect source session");
    let info_envelope = assert_single_envelope(&info, true);
    assert_eq!(info_envelope["session"], source_session);
    assert!(info_envelope["result"]["session"].is_object());

    let close = support::bowser_command()
        .args([
            "--json-envelope",
            "--session-dir",
            source_dir.path().to_str().expect("source dir"),
            "session",
            "close",
            &source_session,
        ])
        .output()
        .expect("close source session");
    let close_envelope = assert_single_envelope(&close, true);
    assert_eq!(close_envelope["result"]["closed"], true);

    let restored_dir = support::test_session_dir();
    let restore = support::bowser_command()
        .args([
            "--json-envelope",
            "--no-ai",
            "--chrome-path",
            chrome.to_str().expect("chrome path"),
            "--chrome-args=--disable-gpu",
            "--session-dir",
            restored_dir.path().to_str().expect("restored dir"),
            "--timeout",
            "60",
            "session",
            "restore",
            "--from",
            checkpoint.to_str().expect("checkpoint path"),
        ])
        .output()
        .expect("restore checkpoint");
    let restore_envelope = assert_single_envelope(&restore, true);
    assert_ne!(restore_envelope["session"], source_session);
    assert!(restore_envelope["session"].as_str().is_some());
    assert!(restore_envelope["page"].as_str().is_some());
    assert_eq!(restore_envelope["result"]["restored"], true);
    assert_eq!(restore_envelope["result"]["checkpoint"]["version"], 1);
}

fn assert_single_envelope(output: &Output, expected_success: bool) -> Value {
    let stdout = std::str::from_utf8(&output.stdout).expect("UTF-8 stdout");
    let envelope: Value = serde_json::from_str(stdout).expect("single JSON document");
    assert_eq!(
        output.status.success(),
        expected_success,
        "unexpected command status; envelope={envelope}; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout.trim().lines().count(), 1);
    assert_eq!(envelope["ok"], expected_success);
    assert_eq!(envelope["envelope"], 1);
    envelope
}
