mod support;

use std::time::Duration;

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn back_waits_for_slow_history_navigation() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(8);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/slow-history"))
        .await
        .expect("navigate slow history");
    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    page.back().await.expect("back to slow history");
    assert_eq!(
        page.title().await.expect("slow history title"),
        "Slow History"
    );

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn forward_waits_for_slow_history_navigation() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(8);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    page.navigate(&server.url("/slow-history"))
        .await
        .expect("navigate slow history");
    page.back().await.expect("back to counter");
    assert_eq!(
        page.title().await.expect("counter title"),
        "Counter Fixture"
    );
    page.forward().await.expect("forward to slow history");
    assert_eq!(
        page.title().await.expect("slow history title"),
        "Slow History"
    );

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exhausted_history_returns_typed_errors_without_waiting_for_timeout() {
    let _guard = support::browser_test_guard();
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(8);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    let started = std::time::Instant::now();
    let back_error = page.back().await.expect_err("back history is exhausted");
    assert!(matches!(back_error, bowser::Error::HistoryBackExhausted));
    assert!(started.elapsed() < Duration::from_secs(2));

    let started = std::time::Instant::now();
    let forward_error = page
        .forward()
        .await
        .expect_err("forward history is exhausted");
    assert!(matches!(
        forward_error,
        bowser::Error::HistoryForwardExhausted
    ));
    assert!(started.elapsed() < Duration::from_secs(2));

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn back_uses_browser_history_when_the_page_overrides_history_back() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(3);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    page.navigate(&server.url("/long"))
        .await
        .expect("navigate long");
    page.evaluate_js("history.back = () => {}; true")
        .await
        .expect("override history back");

    page.back().await.expect("browser-owned back navigation");

    assert_eq!(page.title().await.expect("simple title"), "Fixture Simple");
    browser.close().await.expect("close browser");
}
