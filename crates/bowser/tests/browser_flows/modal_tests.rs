use bowser::{Browser, BrowserEngine, Element};
use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn captures_blocking_modal_and_obscured_background() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/modal-overlay"))
        .await
        .expect("navigate modal overlay");
    let capture = page.capture().await.expect("capture");

    assert_eq!(capture.title, "Modal Overlay Fixture");
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Section { tag, .. } if tag == "dialog" => Some(()),
            _ => None,
        })
        .is_some()
    );
    assert!(support::contains_text(
        &capture.content.visible,
        "Before you continue"
    ));
    assert!(support::contains_text(
        &capture.content.visible,
        "We use cookies and data to keep the service secure."
    ));
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Button { text, .. } if text == "Reject all" => Some(()),
            _ => None,
        })
        .is_some()
    );
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Button { text, .. } if text == "Accept all" => Some(()),
            _ => None,
        })
        .is_some()
    );
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Link { text, .. } if text == "About" => Some(()),
            _ => None,
        })
        .is_none()
    );
    assert!(!support::contains_text(
        &capture.content.visible,
        "Background Search"
    ));
    assert!(
        support::find(&capture.content.visible, &|element| match element {
            Element::Input {
                name: Some(name), ..
            } if name == "q" => Some(()),
            _ => None,
        })
        .is_none()
    );
    assert!(
        support::find(&capture.content.obscured, &|element| match element {
            Element::Link { text, .. } if text == "About" => Some(()),
            _ => None,
        })
        .is_some()
    );
    assert!(support::contains_text(
        &capture.content.obscured,
        "Background Search"
    ));
    assert!(
        support::find(&capture.content.obscured, &|element| match element {
            Element::Input {
                name: Some(name), ..
            } if name == "q" => Some(()),
            _ => None,
        })
        .is_some()
    );

    browser.close().await.expect("close");
}
