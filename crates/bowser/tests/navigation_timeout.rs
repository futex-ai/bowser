mod support;

use std::time::Duration;

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;
use tokio::sync::{Mutex, MutexGuard};

static NAVIGATION_TIMEOUT_TEST_LOCK: Mutex<()> = Mutex::const_new(());

struct NavigationTimeoutTestGuard {
    _lock: MutexGuard<'static, ()>,
    _browser: support::BrowserTestGuard,
}

async fn navigation_timeout_test_guard() -> NavigationTimeoutTestGuard {
    let lock = NAVIGATION_TIMEOUT_TEST_LOCK.lock().await;
    let browser = support::browser_test_guard();
    NavigationTimeoutTestGuard {
        _lock: lock,
        _browser: browser,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reload_waits_for_slow_page_within_configured_timeout() {
    let _guard = navigation_timeout_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(20);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/slow-page"))
        .await
        .expect("navigate slow page");
    page.evaluate_js("window.__bowserReloadMarker = true")
        .await
        .expect("set reload marker");
    page.reload().await.expect("reload slow page");
    let marker_visible: bool = serde_json::from_str(
        &page
            .evaluate_js("Object.prototype.hasOwnProperty.call(window, '__bowserReloadMarker')")
            .await
            .expect("marker visibility"),
    )
    .expect("parse marker visibility");
    assert!(!marker_visible);

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn navigate_waits_for_slow_redirect_within_configured_timeout() {
    let _guard = navigation_timeout_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(15);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/slow-redirect"))
        .await
        .expect("navigate slow redirect");
    assert_eq!(
        page.title().await.expect("redirect target title"),
        "Counter Fixture"
    );

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn slow_exact_url_navigation_is_not_restarted_after_location_assignment() {
    let _guard = navigation_timeout_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(12);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    let token = "single-request";

    page.navigate(&server.url(&format!("/tracked-slow-page/{token}")))
        .await
        .expect("navigate tracked slow page");
    assert_eq!(page.title().await.expect("tracked slow title"), "Slow Page");
    assert_eq!(tracked_slow_page_count(&server, token).await, 1);

    browser.close().await.expect("close browser");
}

async fn tracked_slow_page_count(server: &support::TestServer, token: &str) -> usize {
    reqwest::get(server.url(&format!("/tracked-slow-page-count/{token}")))
        .await
        .expect("tracked slow page count response")
        .text()
        .await
        .expect("tracked slow page count body")
        .parse()
        .expect("tracked slow page count")
}
