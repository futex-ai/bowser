//! Live browser favicon discovery and source decoding regressions.

use std::sync::Arc;

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;

use super::browser_support;
use super::support::spawn_server;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resolves_declared_base_and_root_icons_for_selected_and_background_tabs() {
    let _guard = browser_support::browser_test_guard();
    let server = spawn_server(true).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Browser::launch(browser_support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let first = browser.current_page().await.expect("current page");
    first
        .navigate(&server.url("/relative/page"))
        .await
        .expect("navigate relative page");
    let first_id = browser
        .session_info()
        .await
        .expect("session info")
        .selected_page_id
        .expect("selected page");
    browser
        .new_page(Some(&server.url("/root-only")))
        .await
        .expect("new background page");
    browser
        .select_page(&first_id)
        .await
        .expect("restore selected page");

    let pages = browser
        .live_session_pages()
        .await
        .expect("favicon inventory");

    assert_eq!(pages.len(), 2);
    assert_eq!(pages.iter().filter(|page| page.selected).count(), 1);
    assert!(pages.iter().all(|page| page.favicon.is_some()));
    assert_eq!(
        browser
            .session_info()
            .await
            .expect("selection after inventory")
            .selected_page_id
            .as_deref(),
        Some(first_id.as_str())
    );
    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn loads_svg_ico_data_and_cross_origin_cookie_resources() {
    let _guard = browser_support::browser_test_guard();
    let icon_server = spawn_server(false).await;
    let page_server = spawn_server(true).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Arc::new(
        Browser::launch(browser_support::test_config(session_dir.path()))
            .await
            .expect("launch browser"),
    );
    let page = browser.current_page().await.expect("current page");
    page.navigate(&icon_server.url("/auth/seed"))
        .await
        .expect("seed browser cookie");
    let cross_url = format!(
        "{}?icon={}",
        page_server.url("/cross"),
        url::form_urlencoded::byte_serialize(icon_server.url("/auth/icon.png").as_bytes())
            .collect::<String>()
    );
    page.navigate(&cross_url).await.expect("cross-origin page");
    browser
        .new_page(Some(&page_server.url("/formats/svg")))
        .await
        .expect("SVG page");
    browser
        .new_page(Some(&page_server.url("/formats/ico")))
        .await
        .expect("ICO page");
    browser
        .new_page(Some(&page_server.url("/data")))
        .await
        .expect("data icon page");

    let pages = browser
        .live_session_pages()
        .await
        .expect("favicon inventory");

    for suffix in ["/cross", "/formats/svg", "/formats/ico", "/data"] {
        let page = pages
            .iter()
            .find(|page| page.url.starts_with(&page_server.url(suffix)))
            .unwrap_or_else(|| panic!("missing {suffix} page: {pages:?}"));
        assert!(page.favicon.is_some(), "missing favicon for {suffix}");
    }
    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn allows_three_redirects_and_rejects_a_fourth() {
    let _guard = browser_support::browser_test_guard();
    let server = spawn_server(false).await;
    let session_dir = tempdir().expect("session dir");
    let browser = Browser::launch(browser_support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/broken"))
        .await
        .expect("navigate test page");
    page.evaluate_js("document.querySelector('link').href = '/redirect/3'; true")
        .await
        .expect("set three-redirect icon");
    let allowed = browser
        .live_session_pages()
        .await
        .expect("three redirect inventory");
    assert!(allowed[0].favicon.is_some());

    let page = browser
        .current_page()
        .await
        .expect("current page after inventory");
    page.evaluate_js("document.querySelector('link').href = '/redirect/4'; true")
        .await
        .expect("set four-redirect icon");
    let rejected = browser
        .live_session_pages()
        .await
        .expect("four redirect inventory");
    assert!(rejected[0].favicon.is_none());
    browser.close().await.expect("close browser");
}
