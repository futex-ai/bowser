use std::time::Duration;

use bowser::{Browser, BrowserEngine, Element, ListType, MetadataRecord};
use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn captures_semantics_truncation_resume_and_runtime_features() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());

    let browser = Browser::launch(config.clone())
        .await
        .expect("launch browser");
    let session = browser.session_info().await.expect("session info");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    let capture = page.capture().await.expect("capture simple");
    assert_eq!(capture.title, "Fixture Simple");
    assert!(support::contains_hidden_input(&capture.content.visible));
    assert!(support::table_contains_coupon_input(
        &capture.content.visible
    ));
    let bowser_globals_visible: bool = serde_json::from_str(
        &page
            .evaluate_js(
                r#"(() => ['__bowserElements', '__bowserMetadata', '__bowserStability'].some((key) => key in window) ||
  Object.prototype.hasOwnProperty.call(document.body, '__bowserCaptureId'))()"#,
            )
            .await
            .expect("bowser globals visibility"),
    )
    .expect("parse bowser globals visibility");
    assert!(!bowser_globals_visible);
    let next_link_id = support::find(&capture.content.visible, &|element| match element {
        Element::Link { id, text, .. } if text == "next" => Some(*id),
        _ => None,
    })
    .expect("next link id");

    match page.metadata(next_link_id).await.expect("link metadata") {
        MetadataRecord::Link {
            href,
            text,
            focused,
            visibility,
            bounds,
            ..
        } => {
            assert_eq!(text, "next");
            assert!(href.ends_with("/next"));
            assert!(!focused);
            let visibility = visibility.expect("link visibility");
            assert!(visibility.present);
            assert!(visibility.visible);
            assert!(visibility.clickable);
            support::assert_has_bounds(bounds);
        }
        other => panic!("unexpected metadata: {other:?}"),
    }

    page.navigate(&server.url("/long"))
        .await
        .expect("navigate long");
    let long_capture = page.capture().await.expect("capture long");
    let list_id = match long_capture.content.visible.first().expect("list root") {
        Element::List {
            id,
            list_type,
            items,
            truncation: Some(truncation),
            ..
        } => {
            assert_eq!(*list_type, ListType::Unordered);
            assert_eq!(items.len(), 20);
            assert_eq!(truncation.shown, 20);
            assert_eq!(truncation.total, 22);
            *id
        }
        other => panic!("unexpected long root: {other:?}"),
    };
    match page.expand(list_id).await.expect("expand list") {
        Element::List {
            items, truncation, ..
        } => {
            assert_eq!(items.len(), 22);
            assert!(truncation.is_none());
        }
        other => panic!("unexpected expanded element: {other:?}"),
    }
    match page.metadata(list_id).await.expect("list metadata") {
        MetadataRecord::List {
            items,
            focused,
            visibility,
            bounds,
            ..
        } => {
            assert_eq!(items, 22);
            assert!(!focused);
            let visibility = visibility.expect("list visibility");
            assert!(visibility.present);
            assert!(visibility.visible);
            assert!(visibility.clickable);
            support::assert_has_bounds(bounds);
        }
        other => panic!("unexpected list metadata: {other:?}"),
    }

    browser.detach().await.expect("detach session");

    config.session.id = Some(session.id.clone());
    let resumed = Browser::launch(config).await.expect("resume browser");
    assert!(resumed.session_info().await.expect("resumed info").resumed);
    let resumed_page = resumed.current_page().await.expect("resumed page");
    assert_eq!(resumed_page.title().await.expect("title"), "Fixture Long");

    resumed_page
        .navigate(&server.url("/dynamic"))
        .await
        .expect("navigate dynamic");
    resumed_page
        .wait_for_selector("#loaded", Duration::from_secs(5))
        .await
        .expect("wait for dynamic content");
    let dynamic = resumed_page.capture().await.expect("capture dynamic");
    assert!(support::contains_text(
        &dynamic.content.visible,
        "Loaded later"
    ));

    resumed_page
        .navigate(&server.url("/iframe-parent"))
        .await
        .expect("navigate iframe");
    let iframe = resumed_page.capture().await.expect("capture iframe");
    assert!(support::contains_text(
        &iframe.content.visible,
        "Frame Child"
    ));
    let iframe_window_matches_top: bool = serde_json::from_str(
        &resumed_page
            .evaluate_js("document.querySelector('iframe').contentWindow === window")
            .await
            .expect("iframe window identity"),
    )
    .expect("parse iframe window identity");
    assert!(!iframe_window_matches_top);

    resumed_page
        .navigate(&server.url("/image-page"))
        .await
        .expect("navigate image");
    resumed_page
        .wait_for_selector("img", Duration::from_secs(5))
        .await
        .expect("wait for image");
    let image = resumed_page.capture().await.expect("capture image");
    let image_id = support::find_image_id(&image.content.visible).expect("image id");
    match resumed_page
        .metadata(image_id)
        .await
        .expect("image metadata")
    {
        MetadataRecord::Image {
            alt,
            src,
            description,
            describable,
            focused,
            visibility,
            bounds,
            ..
        } => {
            assert_eq!(alt, "Checkerboard");
            assert!(src.contains("/images/checkerboard.png"));
            assert!(description.is_none());
            assert!(!describable);
            assert!(!focused);
            let visibility = visibility.expect("image visibility");
            assert!(visibility.present);
            assert!(visibility.visible);
            support::assert_has_bounds(bounds);
        }
        other => panic!("unexpected image metadata: {other:?}"),
    }

    resumed_page
        .navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    resumed_page
        .wait_for_selector("input[name='message']", Duration::from_secs(5))
        .await
        .expect("wait for counter input");
    let counter = resumed_page.capture().await.expect("capture counter");
    let message_input_id = support::find_input_id_by_name(&counter.content.visible, "message")
        .expect("message input id");
    resumed_page
        .type_text(message_input_id, "fast")
        .await
        .expect("type fast");
    let focused_capture = resumed_page.capture().await.expect("capture focused input");
    assert!(support::input_has_focus(
        &focused_capture.content.visible,
        "message",
        true
    ));
    match resumed_page
        .metadata(message_input_id)
        .await
        .expect("focused input metadata")
    {
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
            support::assert_has_bounds(bounds);
        }
        other => panic!("unexpected focused input metadata: {other:?}"),
    }
    let typing_stats: support::TypingStats = serde_json::from_str(
        &resumed_page
            .evaluate_js(
                r#"(() => ({
  value: document.querySelector('input[name="message"]').value,
  keydownCount: window.__bowserTypingStats.keydownCount,
  trustedKeydownCount: window.__bowserTypingStats.trustedKeydownCount,
  inputCount: window.__bowserTypingStats.inputCount,
  trustedInputCount: window.__bowserTypingStats.trustedInputCount,
  keyupCount: window.__bowserTypingStats.keyupCount,
  trustedKeyupCount: window.__bowserTypingStats.trustedKeyupCount,
  changeCount: window.__bowserTypingStats.changeCount,
  trustedChangeCount: window.__bowserTypingStats.trustedChangeCount,
  elapsedMs: window.__bowserTypingStats.inputTimes.length > 1
    ? window.__bowserTypingStats.inputTimes[window.__bowserTypingStats.inputTimes.length - 1] - window.__bowserTypingStats.inputTimes[0]
    : 0
}))()"#,
            )
            .await
            .expect("typing stats"),
    )
    .expect("parse typing stats");
    assert_eq!(typing_stats.value, "fast");
    assert_eq!(typing_stats.keydown_count, 4);
    assert_eq!(typing_stats.trusted_keydown_count, 4);
    assert_eq!(typing_stats.input_count, 4);
    assert_eq!(typing_stats.trusted_input_count, 4);
    assert_eq!(typing_stats.keyup_count, 4);
    assert_eq!(typing_stats.trusted_keyup_count, 4);
    assert_eq!(typing_stats.change_count, 0);
    assert_eq!(typing_stats.trusted_change_count, 0);
    assert!(typing_stats.elapsed_ms >= 20.0);

    resumed.close().await.expect("close resumed browser");
}
