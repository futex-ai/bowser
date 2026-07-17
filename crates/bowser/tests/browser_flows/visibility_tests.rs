use bowser::{Browser, BrowserEngine, Element};
use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn captures_visible_content_inside_aria_hidden_wrappers() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/aria-hidden-visible"))
        .await
        .expect("navigate");
    let capture = page.capture().await.expect("capture");

    assert_eq!(capture.title, "Aria Hidden Visible Fixture");
    assert!(support::contains_text(
        &capture.content.visible,
        "Visible content survives aria-hidden wrappers"
    ));
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Link { text, .. } if text == "About" => Some(()),
            _ => None,
        })
        .is_some()
    );
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Link { text, .. } if text == "[link]" => Some(()),
            _ => None,
        })
        .is_some()
    );
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Link { text, .. } if text.contains(".junk") => Some(()),
            Element::Text { text } if text.contains(".junk") => Some(()),
            _ => None,
        })
        .is_none()
    );

    browser.close().await.expect("close");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn captures_obscured_content_and_all_mode_disables_truncation() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.output.include_hidden = true;
    config.output.truncate = false;

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    let simple = page.capture().await.expect("capture simple");
    assert!(
        support::find(&simple.content.obscured, &|element| match element {
            Element::Link { text, .. } if text == "Secret link" => Some(()),
            _ => None,
        })
        .is_some()
    );
    assert!(
        support::find(&simple.content.obscured, &|element| match element {
            Element::Button { text, .. } if text == "Hidden action" => Some(()),
            _ => None,
        })
        .is_some()
    );

    page.navigate(&server.url("/long"))
        .await
        .expect("navigate long");
    let long_capture = page.capture().await.expect("capture long");
    match long_capture.content.visible.first().expect("list root") {
        Element::List {
            items, truncation, ..
        } => {
            assert_eq!(items.len(), 22);
            assert!(truncation.is_none());
        }
        other => panic!("unexpected long root: {other:?}"),
    }

    browser.close().await.expect("close");
}
