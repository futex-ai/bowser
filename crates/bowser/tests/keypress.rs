mod support;

use bowser::{Browser, BrowserEngine, Element};
use serde::Deserialize;
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn press_keys_dispatches_combined_shortcuts_on_the_focused_element() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/keyboard-shortcuts"))
        .await
        .expect("navigate keyboard page");
    let capture = page.capture().await.expect("capture keyboard page");
    let input_id =
        find_input_id_by_name(&capture.content.visible, "message").expect("message input id");

    page.click(input_id).await.expect("focus textarea");
    let focused_capture = page.capture().await.expect("capture focused textarea");
    assert!(input_has_focus(&focused_capture.content.visible, "message"));

    page.press_keys(&["cmd".to_string(), "enter".to_string()])
        .await
        .expect("press cmd enter");
    let status: String = serde_json::from_str(
        &page
            .evaluate_js("document.getElementById('status').textContent")
            .await
            .expect("status after cmd enter"),
    )
    .expect("parse status after cmd enter");
    assert_eq!(status, "Meta+Enter fired");

    let combo_capture = page.capture().await.expect("capture combo result");
    let combo_yaml = bowser::to_yaml(&combo_capture).expect("combo yaml");
    assert!(combo_yaml.contains("Meta+Enter fired"));

    page.press_keys(&["space".to_string()])
        .await
        .expect("press space");
    let space_status: String = serde_json::from_str(
        &page
            .evaluate_js("document.getElementById('status').textContent")
            .await
            .expect("status after space"),
    )
    .expect("parse status after space");
    assert_eq!(space_status, "Space fired");

    let space_capture = page.capture().await.expect("capture space result");
    let space_yaml = bowser::to_yaml(&space_capture).expect("space yaml");
    assert!(space_yaml.contains("Space fired"));

    let log: Vec<KeypressEvent> = serde_json::from_str(
        &page
            .evaluate_js("window.__bowserKeypressLog")
            .await
            .expect("keypress log"),
    )
    .expect("parse keypress log");
    assert!(log.iter().any(|event| {
        event.event_type == "keydown"
            && event.combo == "Meta+Enter"
            && event.meta_key
            && event.is_trusted
    }));
    assert!(log.iter().any(|event| {
        event.event_type == "keyup" && event.key == "Meta" && event.meta_key && event.is_trusted
    }));
    assert!(
        log.iter().any(|event| event.event_type == "keydown"
            && event.combo == "Space"
            && event.is_trusted)
    );

    browser.close().await.expect("close browser");
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeypressEvent {
    #[serde(rename = "type")]
    event_type: String,
    key: String,
    is_trusted: bool,
    meta_key: bool,
    combo: String,
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

fn input_has_focus(elements: &[Element], target_name: &str) -> bool {
    find(elements, &|element| match element {
        Element::Input {
            name: Some(name),
            focused,
            ..
        } if name == target_name => Some(*focused),
        _ => None,
    })
    .unwrap_or(false)
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
