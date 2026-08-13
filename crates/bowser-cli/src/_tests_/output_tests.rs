//! Machine output mapping tests.

use super::CommandOutput;
use crate::error::CliError;

#[test]
fn command_output_serializes_embedded_results() {
    let output =
        CommandOutput::result(serde_json::json!({ "value": 7 })).expect("serialize command result");
    assert_eq!(output.result, serde_json::json!({ "value": 7 }));
    assert!(output.files.is_empty());
}

#[test]
fn session_failures_have_stable_machine_codes_and_details() {
    let error = CliError::Bowser(bowser::Error::SessionNotFound {
        session_id: "bsr_missing".to_string(),
    });
    let (code, detail) = error.machine_code_and_detail();
    assert_eq!(code, "session_not_found");
    assert_eq!(detail, serde_json::json!({ "session_id": "bsr_missing" }));
}

#[test]
fn checkpoint_failures_have_stable_machine_codes_and_details() {
    let error = CliError::Bowser(bowser::Error::CheckpointUnsupported { version: 99 });
    let (code, detail) = error.machine_code_and_detail();
    assert_eq!(code, "checkpoint_unsupported");
    assert_eq!(detail, serde_json::json!({ "version": 99 }));
}
