mod support;

use bowser::{Browser, BrowserEngine, Element, PageEngine, ScrollTarget};
use serde::Deserialize;
use tempfile::tempdir;
use tokio::time::{Duration, Instant, sleep};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scroll_dispatches_trusted_wheel_events() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/wheel-page"))
        .await
        .expect("navigate wheel page");
    page.capture().await.expect("capture wheel page");

    page.scroll(ScrollTarget::Down).await.expect("scroll down");
    page.scroll(ScrollTarget::Up).await.expect("scroll up");

    let mut stats = wheel_stats(page.as_ref()).await;
    let deadline = Instant::now() + Duration::from_millis(500);
    while stats.count < 6 && Instant::now() < deadline {
        sleep(Duration::from_millis(25)).await;
        stats = wheel_stats(page.as_ref()).await;
    }
    assert!(stats.count >= 6);
    assert_eq!(stats.count, stats.trusted_count);
    assert!(stats.deltas.iter().any(|delta| *delta > 0.0));
    assert!(stats.deltas.iter().any(|delta| *delta < 0.0));

    browser.close().await.expect("close browser");
}

async fn wheel_stats(page: &dyn PageEngine) -> WheelStats {
    serde_json::from_str(
        &page
            .evaluate_js("window.__bowserWheelStats")
            .await
            .expect("wheel stats"),
    )
    .expect("parse wheel stats")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn submit_prefers_enter_for_form_controls() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/submit-page"))
        .await
        .expect("navigate submit page");
    let capture = page.capture().await.expect("capture submit page");
    let input_id =
        find_input_id_by_name(&capture.content.visible, "query").expect("query input id");
    let form_id = find_form_id(&capture.content.visible).expect("form id");

    page.type_text(input_id, "bowser")
        .await
        .expect("type query");
    page.submit(form_id).await.expect("submit form");

    let stats: SubmitStats = serde_json::from_str(
        &page
            .evaluate_js("window.__bowserSubmitStats")
            .await
            .expect("submit stats"),
    )
    .expect("parse submit stats");
    assert_eq!(stats.submit_count, 1);
    assert_eq!(stats.trusted_submit_count, 1);
    assert_eq!(stats.last_value, "bowser");

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stealth_clicks_plain_buttons_without_backend_focus() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.stealth = true;

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/click-reveals-images"))
        .await
        .expect("navigate click page");
    let capture = page.capture().await.expect("capture click page");
    let button_id =
        find_button_id_by_text(&capture.content.visible, "Accept").expect("accept button id");

    page.click(button_id).await.expect("click accept button");

    let status: String = serde_json::from_str(
        &page
            .evaluate_js("document.getElementById('status').textContent")
            .await
            .expect("status text"),
    )
    .expect("parse status text");
    assert_eq!(status, "clicked");

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn persistent_profiles_keep_local_state_across_relaunches() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.persistent_profile = true;

    let browser = Browser::launch(config.clone())
        .await
        .expect("launch first browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/profile-state"))
        .await
        .expect("navigate first profile page");
    page.wait_for_selector("#count", std::time::Duration::from_secs(5))
        .await
        .expect("wait for first profile marker");
    page.capture().await.expect("capture first profile page");
    let first_count: String = serde_json::from_str(
        &page
            .evaluate_js("localStorage.getItem('bowser-profile-count')")
            .await
            .expect("first count"),
    )
    .expect("parse first count");
    assert_eq!(first_count, "1");
    browser.close().await.expect("close first browser");

    let browser = Browser::launch(config)
        .await
        .expect("launch second browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/profile-state"))
        .await
        .expect("navigate second profile page");
    page.wait_for_selector("#count", std::time::Duration::from_secs(5))
        .await
        .expect("wait for second profile marker");
    page.capture().await.expect("capture second profile page");
    let second_count: String = serde_json::from_str(
        &page
            .evaluate_js("localStorage.getItem('bowser-profile-count')")
            .await
            .expect("second count"),
    )
    .expect("parse second count");
    assert_eq!(second_count, "2");

    browser.close().await.expect("close second browser");
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WheelStats {
    count: u64,
    trusted_count: u64,
    deltas: Vec<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitStats {
    submit_count: u64,
    trusted_submit_count: u64,
    last_value: String,
}

fn find_input_id_by_name(elements: &[Element], target_name: &str) -> Option<u32> {
    find(elements, &|element| match element {
        Element::Input {
            id: Some(id),
            name: Some(name),
            ..
        } if name == target_name => Some(*id),
        _ => None,
    })
}

fn find_form_id(elements: &[Element]) -> Option<u32> {
    find(elements, &|element| match element {
        Element::Form { id, .. } => Some(*id),
        _ => None,
    })
}

fn find_button_id_by_text(elements: &[Element], target_text: &str) -> Option<u32> {
    find(elements, &|element| match element {
        Element::Button { id, text, .. } if text == target_text => Some(*id),
        _ => None,
    })
}

fn find<T>(elements: &[Element], predicate: &dyn Fn(&Element) -> Option<T>) -> Option<T> {
    for element in elements {
        if let Some(found) = predicate(element) {
            return Some(found);
        }
        match element {
            Element::Table { headers, rows, .. } => {
                for header in headers {
                    if let Some(found) = find(&header.children, predicate) {
                        return Some(found);
                    }
                }
                for row in rows {
                    for cell in &row.cells {
                        if let Some(found) = find(&cell.children, predicate) {
                            return Some(found);
                        }
                    }
                }
            }
            Element::List { items, .. } => {
                for item in items {
                    if let Some(found) = find(&item.children, predicate) {
                        return Some(found);
                    }
                }
            }
            Element::Nav { children, .. }
            | Element::Form { children, .. }
            | Element::Section { children, .. }
            | Element::Iframe { children, .. } => {
                if let Some(found) = find(children, predicate) {
                    return Some(found);
                }
            }
            Element::Heading { .. }
            | Element::Link { .. }
            | Element::Button { .. }
            | Element::Input { .. }
            | Element::Text { .. }
            | Element::Image { .. } => {}
        }
    }
    None
}
