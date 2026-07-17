use crate::cdp_trace::is_high_risk_google_method;

#[test]
fn google_cdp_risk_marks_runtime_enable_only() {
    assert!(is_high_risk_google_method("Runtime.enable"));
    assert!(!is_high_risk_google_method("Runtime.disable"));
    assert!(!is_high_risk_google_method("Page.navigate"));
    assert!(!is_high_risk_google_method("Input.dispatchKeyEvent"));
}
