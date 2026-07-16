mod support;

use std::time::Duration;

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;
use tokio::sync::{Mutex, MutexGuard};

static NAVIGATION_RESUMED_TEST_LOCK: Mutex<()> = Mutex::const_new(());

struct NavigationResumedTestGuard {
    _lock: MutexGuard<'static, ()>,
    _browser: support::BrowserTestGuard,
}

async fn navigation_resumed_test_guard() -> NavigationResumedTestGuard {
    let lock = NAVIGATION_RESUMED_TEST_LOCK.lock().await;
    let browser = support::browser_test_guard();
    NavigationResumedTestGuard {
        _lock: lock,
        _browser: browser,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resumed_redirect_navigation_accepts_changed_document_tests() {
    let _guard = navigation_resumed_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(15);

    let browser = Browser::launch(config.clone())
        .await
        .expect("launch browser");
    let session = browser.session_info().await.expect("session info");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    browser.detach().await.expect("detach browser");

    config.session.id = Some(session.id);
    let resumed = Browser::launch(config).await.expect("resume browser");
    let resumed_page = resumed.current_page().await.expect("resumed page");
    resumed_page
        .navigate(&server.url("/slow-redirect"))
        .await
        .expect("navigate resumed redirect");
    assert_eq!(
        resumed_page.title().await.expect("redirect target title"),
        "Counter Fixture"
    );

    resumed.close().await.expect("close resumed browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resumed_normalized_url_navigation_accepts_changed_document_tests() {
    let _guard = navigation_resumed_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(5);

    let browser = Browser::launch(config.clone())
        .await
        .expect("launch browser");
    let session = browser.session_info().await.expect("session info");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    browser.detach().await.expect("detach browser");

    config.session.id = Some(session.id);
    let resumed = Browser::launch(config).await.expect("resume browser");
    let resumed_page = resumed.current_page().await.expect("resumed page");
    resumed_page
        .navigate(&server.url("/./counter"))
        .await
        .expect("navigate normalized URL");
    assert_eq!(
        resumed_page.title().await.expect("normalized target title"),
        "Counter Fixture"
    );

    resumed.close().await.expect("close resumed browser");
}
