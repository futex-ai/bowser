mod support;

use std::time::{Duration, Instant};

use bowser::{Browser, BrowserEngine, Element, Error};
use tempfile::tempdir;
use tokio::sync::{Mutex, MutexGuard};

static NAVIGATION_TEST_LOCK: Mutex<()> = Mutex::const_new(());

struct NavigationTestGuard {
    _lock: MutexGuard<'static, ()>,
    _browser: support::BrowserTestGuard,
}

async fn navigation_test_guard() -> NavigationTestGuard {
    let lock = NAVIGATION_TEST_LOCK.lock().await;
    let browser = support::browser_test_guard();
    NavigationTestGuard {
        _lock: lock,
        _browser: browser,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn navigate_invalidates_cached_capture_state() {
    let _guard = navigation_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    let simple = page.capture().await.expect("capture simple");
    let next_link_id = find_link_id(&simple.content.visible, "next").expect("next link id");
    page.evaluate_js("window.__bowserNavigationMarker = true")
        .await
        .expect("set navigation marker");

    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    assert_eq!(
        page.title().await.expect("counter title"),
        "Counter Fixture"
    );
    let marker_visible: bool = serde_json::from_str(
        &page
            .evaluate_js("Object.prototype.hasOwnProperty.call(window, '__bowserNavigationMarker')")
            .await
            .expect("marker visibility"),
    )
    .expect("parse marker visibility");
    assert!(!marker_visible);

    match page.capture_subtree(next_link_id).await {
        Ok(Element::Link { text, .. }) if text == "next" => {
            panic!("navigation reused the stale simple-page capture")
        }
        Ok(_) | Err(Error::ElementNotFound { .. }) => {}
        Err(err) => panic!("unexpected capture error after navigation: {err}"),
    }

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn navigate_about_blank_invalidates_cached_capture_state() {
    let _guard = navigation_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    let simple = page.capture().await.expect("capture simple");
    let next_link_id = find_link_id(&simple.content.visible, "next").expect("next link id");

    page.navigate("about:blank")
        .await
        .expect("navigate about blank");
    assert_eq!(page.url().await.expect("blank url"), "about:blank");

    match page.capture_subtree(next_link_id).await {
        Ok(Element::Link { text, .. }) if text == "next" => {
            panic!("about:blank reused the stale simple-page capture")
        }
        Ok(_) | Err(Error::ElementNotFound { .. }) => {}
        Err(err) => panic!("unexpected capture error after about:blank: {err}"),
    }

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reload_about_blank_completes_without_waiting_for_non_blank_url() {
    let _guard = navigation_test_guard().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate("about:blank")
        .await
        .expect("navigate about blank");
    page.reload().await.expect("reload about blank");

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_navigation_invalidates_cached_capture_state() {
    let _guard = navigation_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(5);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    let simple = page.capture().await.expect("capture simple");
    let next_link_id = find_link_id(&simple.content.visible, "next").expect("next link id");

    let result = page.navigate(&server.url("/slow-page")).await;
    assert!(result.is_err());
    tokio::time::sleep(Duration::from_secs(7)).await;

    match page.capture_subtree(next_link_id).await {
        Ok(Element::Link { text, .. }) if text == "next" => {
            panic!("failed navigation reused the stale simple-page capture")
        }
        Ok(_) | Err(_) => {}
    }

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unrelated_redirect_does_not_satisfy_failed_navigation() {
    let _guard = navigation_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(5);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/delayed-redirect"))
        .await
        .expect("navigate delayed redirect");
    let started_at = Instant::now();
    let result = page.navigate(&server.url("/slow-page")).await;
    assert!(result.is_err());
    assert!(started_at.elapsed() >= Duration::from_secs(4));

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn back_invalidates_cached_capture_state() {
    let _guard = navigation_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    let counter = page.capture().await.expect("capture counter");
    let message_input_id =
        find_input_id_by_name(&counter.content.visible, "message").expect("message input id");

    page.back().await.expect("go back");

    match page.capture_subtree(message_input_id).await {
        Ok(Element::Input {
            name: Some(name), ..
        }) if name == "message" => panic!("back reused the stale counter-page capture"),
        Ok(_) | Err(Error::ElementNotFound { .. }) => {}
        Err(err) => panic!("unexpected capture error after back: {err}"),
    }

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn forward_invalidates_cached_capture_state() {
    let _guard = navigation_test_guard().await;
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    page.back().await.expect("go back");
    let simple = page.capture().await.expect("capture simple after back");
    let next_link_id = find_link_id(&simple.content.visible, "next").expect("next link id");

    page.forward().await.expect("go forward");

    match page.capture_subtree(next_link_id).await {
        Ok(Element::Link { text, .. }) if text == "next" => {
            panic!("forward reused the stale simple-page capture")
        }
        Ok(_) | Err(Error::ElementNotFound { .. }) => {}
        Err(err) => panic!("unexpected capture error after forward: {err}"),
    }

    browser.close().await.expect("close browser");
}

fn find_link_id(elements: &[Element], expected_text: &str) -> Option<u32> {
    elements.iter().find_map(|element| match element {
        Element::Link { id, text, .. } if text == expected_text => Some(*id),
        Element::List { items, .. } => items
            .iter()
            .find_map(|item| find_link_id(&item.children, expected_text)),
        Element::Table { headers, rows, .. } => headers
            .iter()
            .find_map(|cell| find_link_id(&cell.children, expected_text))
            .or_else(|| {
                rows.iter().find_map(|row| {
                    row.cells
                        .iter()
                        .find_map(|cell| find_link_id(&cell.children, expected_text))
                })
            }),
        element => element
            .children()
            .and_then(|children| find_link_id(children, expected_text)),
    })
}

fn find_input_id_by_name(elements: &[Element], expected_name: &str) -> Option<u32> {
    elements.iter().find_map(|element| match element {
        Element::Input {
            id: Some(id),
            name: Some(name),
            ..
        } if name == expected_name => Some(*id),
        Element::List { items, .. } => items
            .iter()
            .find_map(|item| find_input_id_by_name(&item.children, expected_name)),
        Element::Table { headers, rows, .. } => headers
            .iter()
            .find_map(|cell| find_input_id_by_name(&cell.children, expected_name))
            .or_else(|| {
                rows.iter().find_map(|row| {
                    row.cells
                        .iter()
                        .find_map(|cell| find_input_id_by_name(&cell.children, expected_name))
                })
            }),
        element => element
            .children()
            .and_then(|children| find_input_id_by_name(children, expected_name)),
    })
}
