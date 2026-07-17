mod support;

use bowser::{Browser, BrowserEngine, Element};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn captures_below_fold_content_and_expands_truncated_body_root() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.output.max_children = 4;

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/orders-long"))
        .await
        .expect("navigate orders");
    let capture = page.capture().await.expect("capture orders");

    let body_id = match capture.content.visible.as_slice() {
        [
            Element::Section {
                id,
                tag,
                children,
                truncation: Some(truncation),
                ..
            },
        ] => {
            assert_eq!(tag, "body");
            assert_eq!(children.len(), 4);
            assert_eq!(truncation.shown, 4);
            assert_eq!(truncation.total, 10);
            *id
        }
        other => panic!("expected truncated body root, got {other:?}"),
    };

    assert!(contains_text(&capture.content.visible, "My orders"));
    assert!(!contains_text(&capture.content.visible, "Order 6"));

    match page.expand(body_id).await.expect("expand body") {
        Element::Section {
            tag,
            children,
            truncation,
            ..
        } => {
            assert_eq!(tag, "body");
            assert!(truncation.is_none());
            assert!(contains_text(&children, "Order 1"));
            assert!(contains_text(&children, "Order 6"));
            assert!(contains_text(
                &children,
                "Thursday 5th February 2026 at 10:00am - 11:00am"
            ));
        }
        other => panic!("expected expanded body root, got {other:?}"),
    }

    browser.close().await.expect("close browser");
}

fn contains_text(elements: &[Element], needle: &str) -> bool {
    elements.iter().any(|element| match element {
        Element::Heading { text, .. } | Element::Text { text } => text.contains(needle),
        Element::Link { text, .. } | Element::Button { text, .. } => text.contains(needle),
        Element::Input {
            value,
            label,
            placeholder,
            ..
        } => {
            value.contains(needle)
                || label.as_deref().is_some_and(|value| value.contains(needle))
                || placeholder
                    .as_deref()
                    .is_some_and(|value| value.contains(needle))
        }
        Element::Image {
            alt, description, ..
        } => {
            alt.contains(needle)
                || description
                    .as_deref()
                    .is_some_and(|value| value.contains(needle))
        }
        Element::Table { headers, rows, .. } => {
            headers
                .iter()
                .any(|header| contains_text(&header.children, needle))
                || rows.iter().any(|row| {
                    row.cells
                        .iter()
                        .any(|cell| contains_text(&cell.children, needle))
                })
        }
        Element::List { items, .. } => items
            .iter()
            .any(|item| contains_text(&item.children, needle)),
        Element::Nav { children, .. }
        | Element::Form { children, .. }
        | Element::Section { children, .. }
        | Element::Iframe { children, .. } => contains_text(children, needle),
    })
}
