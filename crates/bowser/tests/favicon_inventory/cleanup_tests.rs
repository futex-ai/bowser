//! Cancellation must restore page traffic even when the renderer is busy.

use std::time::Duration;

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;

use super::browser_support;
use super::support::spawn_server;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn busy_page_cannot_prevent_interception_cleanup() {
    let _guard = browser_support::browser_test_guard();
    let server = spawn_server(false).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Browser::launch(browser_support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/slow")).await.expect("navigate");
    server.wait_for_slow_requests(1).await;
    let baseline = server.slow_requests();
    let stall = async {
        server.wait_for_slow_requests(baseline + 1).await;
        page.evaluate_js("setTimeout(() => { const end = performance.now() + 1200; while (performance.now() < end) {} }, 20); true")
            .await.expect("schedule busy renderer");
    };
    let (pages, ()) = tokio::join!(browser.live_session_pages(), stall);
    assert!(pages.expect("bounded inventory")[0].favicon.is_none());
    tokio::time::sleep(Duration::from_millis(500)).await;
    let response = tokio::time::timeout(
        Duration::from_secs(2),
        page.evaluate_js("fetch('/replacement.png').then(response => response.status)"),
    )
    .await;
    browser.close().await.expect("close browser");
    assert_eq!(
        response
            .expect("page traffic must resume")
            .expect("fetch response"),
        "200"
    );
}
