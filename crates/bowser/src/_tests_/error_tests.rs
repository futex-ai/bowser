use super::is_target_session_missing;

#[test]
fn recognizes_chrome_missing_target_session_error() {
    assert!(is_target_session_missing(
        "Error -32001: Session with given id not found."
    ));
    assert!(is_target_session_missing(
        "activate page: send failed because receiver is gone"
    ));
    assert!(!is_target_session_missing(
        "Execution context was destroyed"
    ));
}
