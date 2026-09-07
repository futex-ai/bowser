//! Live document and declaration race regressions.

use std::time::{Duration, Instant};

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;

use super::browser_support;
use super::support::spawn_server;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn global_budget_returns_null_for_slow_icons() {
    let _guard = browser_support::browser_test_guard();
    let server = spawn_server(false).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Browser::launch(browser_support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/slow"))
        .await
        .expect("navigate slow icon page");
    server.wait_for_slow_requests(1).await;
    let baseline_requests = server.slow_requests();
    let started = Instant::now();

    let pages = browser
        .live_session_pages()
        .await
        .expect("bounded favicon inventory");

    assert!(started.elapsed() < Duration::from_millis(1_250));
    assert!(pages[0].favicon.is_none());
    tokio::time::sleep(Duration::from_millis(250)).await;
    let settled_chunks = server.slow_chunks_after(baseline_requests);
    let settled_requests = server.slow_requests();
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(server.slow_chunks_after(baseline_requests), settled_chunks);
    assert_eq!(server.slow_requests(), settled_requests);
    assert!(settled_requests > baseline_requests);
    let body = page
        .evaluate_js("fetch('/replacement.png').then(r => r.status)")
        .await
        .expect("page traffic after inventory timeout");
    assert_eq!(body, "200");
    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn declaration_changes_during_acquisition_discard_the_old_icon() {
    let _guard = browser_support::browser_test_guard();
    let server = spawn_server(false).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Browser::launch(browser_support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/racing"))
        .await
        .expect("navigate slow icon page");
    server.wait_for_slow_requests(1).await;
    let baseline_requests = server.slow_requests();
    let change = async {
        server.wait_for_slow_requests(baseline_requests + 1).await;
        page.evaluate_js("document.querySelector('link').href = '/replacement.png'; true")
            .await
            .expect("replace declaration during acquisition");
    };
    let (pages, ()) = tokio::join!(browser.live_session_pages(), change);
    let pages = pages.expect("favicon inventory");

    assert!(pages[0].favicon.is_none());
    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn same_url_document_replacement_discards_the_old_icon() {
    let _guard = browser_support::browser_test_guard();
    let server = spawn_server(false).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Browser::launch(browser_support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/racing"))
        .await
        .expect("navigate slow icon page");
    server.wait_for_slow_requests(1).await;
    let baseline_requests = server.slow_requests();
    let change = async {
        server.wait_for_slow_requests(baseline_requests + 1).await;
        page.evaluate_js("location.reload(); true")
            .await
            .expect("reload during acquisition");
    };
    let (pages, ()) = tokio::join!(browser.live_session_pages(), change);
    let pages = pages.expect("favicon inventory");

    assert!(pages[0].favicon.is_none());
    browser.close().await.expect("close browser");
}
