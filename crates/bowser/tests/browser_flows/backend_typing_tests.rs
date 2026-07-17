//! Backend-focused typing tests.

use bowser::{Browser, BrowserEngine, Element};
use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stealth_typing_replaces_prefilled_same_page_text() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.stealth = true;

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    page.evaluate_js("document.querySelector('input[name=message]').value = 'old'")
        .await
        .expect("prefill input");
    let capture = page.capture().await.expect("capture counter");
    let input_id = support::find(&capture.content.visible, &|element| match element {
        Element::Input {
            id: Some(id),
            name,
            label,
            ..
        } if name.as_deref() == Some("message") || label.as_deref() == Some("Message") => Some(*id),
        _ => None,
    })
    .expect("message input id");

    page.type_text(input_id, "replacement")
        .await
        .expect("replace text");

    let value: String = serde_json::from_str(
        &page
            .evaluate_js("document.querySelector('input[name=message]').value")
            .await
            .expect("input value"),
    )
    .expect("parse input value");
    assert_eq!(value, "replacement");

    browser.close().await.expect("close browser");
}
