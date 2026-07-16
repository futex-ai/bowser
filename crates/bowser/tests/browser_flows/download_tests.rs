use bowser::{Browser, BrowserEngine, Error};
use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_native_downloads_follow_redirects_and_preserve_cookies() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let downloads_dir = tempdir().expect("downloads dir");
    let config = support::test_config(session_dir.path());
    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    let redirected_path = downloads_dir.path().join("redirected.csv");
    let redirected = page
        .download(&server.url("/download/redirect"), &redirected_path)
        .await
        .expect("redirected download");
    assert_eq!(redirected.filename, "report.csv");
    assert_eq!(
        tokio::fs::read_to_string(&redirected_path)
            .await
            .expect("redirected body"),
        "name,value\nalpha,1\n"
    );

    page.navigate(&server.url("/download/cookie-page"))
        .await
        .expect("seed cookie");
    page.wait_for_selector("a", std::time::Duration::from_secs(5))
        .await
        .expect("wait cookie page");
    let cookie_path = downloads_dir.path().join("cookie.csv");
    let cookie = page
        .download(&server.url("/download/cookie.csv"), &cookie_path)
        .await
        .expect("cookie download");
    assert_eq!(cookie.filename, "cookie.csv");
    assert_eq!(
        tokio::fs::read_to_string(&cookie_path)
            .await
            .expect("cookie body"),
        "name,value\ncookie,1\n"
    );

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_native_download_rejects_invalid_destinations() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());
    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    let missing_parent = session_dir.path().join("missing").join("file.csv");

    let err = page
        .download(&server.url("/download/report.csv"), &missing_parent)
        .await
        .expect_err("invalid destination");
    assert!(matches!(err, Error::DownloadDestination { .. }));

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_native_download_reports_navigation_start_failures() {
    let _guard = support::browser_test_guard();
    let session_dir = tempdir().expect("session dir");
    let downloads_dir = tempdir().expect("downloads dir");
    let config = support::test_config(session_dir.path());
    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    let destination = downloads_dir.path().join("invalid.csv");

    let error = page
        .download("http://[::1", &destination)
        .await
        .expect_err("invalid download URL");
    assert!(matches!(error, Error::DownloadStart { .. }));

    browser.close().await.expect("close browser");
}
