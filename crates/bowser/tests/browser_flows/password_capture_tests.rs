//! Password capture redaction tests.

use bowser::{Browser, BrowserEngine, Element, InputType, MetadataRecord, to_json, to_yaml};
use tempfile::tempdir;

use super::support;

const RAW_PASSWORD: &str = "do-not-expose-this-password";
const REDACTED_PASSWORD: &str = "[redacted]";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn password_values_are_redacted_from_capture_metadata_and_rendering() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());
    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");

    let capture = page.capture().await.expect("capture simple");
    let (password_id, password_value) =
        support::find(&capture.content.visible, &|element| match element {
            Element::Input {
                id: Some(id),
                name: Some(name),
                input_type: InputType::Password,
                value,
                ..
            } if name == "account_password" => Some((*id, value.clone())),
            _ => None,
        })
        .expect("password input");
    assert_eq!(password_value, REDACTED_PASSWORD);

    match page.metadata(password_id).await.expect("password metadata") {
        MetadataRecord::Input { value, .. } => assert_eq!(value, REDACTED_PASSWORD),
        other => panic!("unexpected password metadata: {other:?}"),
    }
    for rendered in [
        to_yaml(&capture).expect("password YAML"),
        to_json(&capture).expect("password JSON"),
    ] {
        assert!(rendered.contains(REDACTED_PASSWORD));
        assert!(!rendered.contains(RAW_PASSWORD));
    }

    browser.close().await.expect("close browser");
}
