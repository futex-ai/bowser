//! Live inventory must preserve existing browser target sessions.

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;

use super::browser_support;
use super::support::spawn_server;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn repeated_inventory_preserves_a_preexisting_page_handle() {
    let _guard = browser_support::browser_test_guard();
    let server = spawn_server(true).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Browser::launch(browser_support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/relative/page"))
        .await
        .expect("navigate");
    let mut failure = None;
    for _ in 0..3 {
        browser
            .live_session_metadata()
            .await
            .expect("live metadata");
        if let Err(error) = page.evaluate_js("document.title").await {
            failure = Some(error);
            break;
        }
    }
    browser.close().await.expect("close browser");
    assert!(
        failure.is_none(),
        "inventory invalidated an existing page: {failure:?}"
    );
}
