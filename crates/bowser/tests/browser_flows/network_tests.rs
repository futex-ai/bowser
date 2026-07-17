use std::time::{Duration, Instant};

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capture_does_not_wait_for_chatty_background_requests_to_finish() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/chatty"))
        .await
        .expect("navigate chatty");
    let started = Instant::now();
    let capture = page.capture().await.expect("capture chatty");
    let elapsed = started.elapsed();

    assert_eq!(capture.title, "Chatty Fixture");
    assert!(support::contains_text(
        &capture.content.visible,
        "Chatty ready"
    ));
    assert!(elapsed < Duration::from_secs(4));

    browser.close().await.expect("close");
}
