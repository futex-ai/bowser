#[path = "support/mod.rs"]
mod support;

use bowser::{Browser, BrowserEngine, Element, MetadataRecord};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clicking_text_input_sets_focus_in_capture_yaml_and_metadata() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    let capture = page.capture().await.expect("capture counter");
    let input_id =
        find_input_id_by_name(&capture.content.visible, "message").expect("message input id");

    page.click(input_id).await.expect("click input");

    let focused_capture = page.capture().await.expect("capture focused input");
    assert!(input_has_focus(
        &focused_capture.content.visible,
        "message",
        true
    ));

    let yaml = bowser::to_yaml(&focused_capture).expect("render yaml");
    assert!(yaml.contains("name: message"));
    assert!(yaml.contains("focused: true"));

    match page.metadata(input_id).await.expect("input metadata") {
        MetadataRecord::Input {
            focused,
            visibility,
            bounds,
            ..
        } => {
            assert!(focused);
            let visibility = visibility.expect("input visibility");
            assert!(visibility.present);
            assert!(visibility.visible);
            assert!(visibility.clickable);
            assert!(bounds.is_some());
        }
        other => panic!("unexpected metadata: {other:?}"),
    }

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resumed_metadata_rebuilds_live_ids_before_runtime_lookup() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());

    let browser = Browser::launch(config.clone())
        .await
        .expect("launch browser");
    let session = browser.session_info().await.expect("session info");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    let capture = page.capture().await.expect("capture counter");
    let input_id =
        find_input_id_by_name(&capture.content.visible, "message").expect("message input id");

    browser.detach().await.expect("detach browser");

    config.session.id = Some(session.id);
    let resumed = Browser::launch(config).await.expect("resume browser");
    let resumed_page = resumed.current_page().await.expect("resumed page");

    match resumed_page
        .metadata(input_id)
        .await
        .expect("resumed metadata")
    {
        MetadataRecord::Input {
            visibility, bounds, ..
        } => {
            let visibility = visibility.expect("resumed input visibility");
            assert!(visibility.present);
            assert!(visibility.visible);
            assert!(visibility.clickable);
            assert!(bounds.is_some());
        }
        other => panic!("unexpected resumed metadata: {other:?}"),
    }

    resumed.close().await.expect("close resumed browser");
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

fn input_has_focus(elements: &[Element], target_name: &str, expected: bool) -> bool {
    find(elements, &|element| match element {
        Element::Input {
            name: Some(name),
            focused,
            ..
        } if name == target_name => Some(*focused == expected),
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
            Element::Heading { .. } => {}
            Element::Link { .. }
            | Element::Button { .. }
            | Element::Input { .. }
            | Element::Text { .. }
            | Element::Image { .. } => {}
        }
    }
    None
}
