use std::io::Cursor;
use std::time::Duration;

use bowser::{AiProvider, Browser, BrowserEngine, Element, InputType, MetadataRecord, to_yaml};
use tempfile::tempdir;
use url::form_urlencoded::byte_serialize;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn captures_cross_origin_iframe_content_with_shared_ids() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let child_url = child_server
        .url("/iframe-child")
        .replace("127.0.0.1", "localhost");
    let encoded_child_url: String = byte_serialize(child_url.as_bytes()).collect();
    let parent_url = format!(
        "{}/iframe-parent-external?src={encoded_child_url}",
        parent_server.url("")
    );

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&parent_url).await.expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    let yaml = to_yaml(&capture).expect("render yaml");

    assert!(yaml.contains("button#1: Outer"), "yaml: {yaml}");
    assert!(yaml.contains("iframe#2:"), "yaml: {yaml}");
    assert!(yaml.contains("article#3:"), "yaml: {yaml}");
    assert!(yaml.contains("h2: Frame Child"), "yaml: {yaml}");
    assert!(yaml.contains("link#4: Inside"), "yaml: {yaml}");
    assert!(yaml.contains("link#5: After"), "yaml: {yaml}");

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clicks_aria_checkbox_inside_cross_origin_iframe() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let child_url = child_server
        .url("/iframe-checkbox-child")
        .replace("127.0.0.1", "localhost");
    let encoded_child_url: String = byte_serialize(child_url.as_bytes()).collect();
    let parent_url = format!(
        "{}/iframe-parent-external?src={encoded_child_url}",
        parent_server.url("")
    );

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&parent_url).await.expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    let checkbox_id = find_checkbox_id(&capture.content.visible).expect("checkbox id");

    page.click(checkbox_id).await.expect("click checkbox");

    let after = page.capture().await.expect("capture after click");
    assert!(has_checked_checkbox(&after.content.visible));

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn screenshots_image_inside_cross_origin_iframe() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let child_url = child_server
        .url("/iframe-image-child")
        .replace("127.0.0.1", "localhost");
    let encoded_child_url: String = byte_serialize(child_url.as_bytes()).collect();
    let parent_url = format!(
        "{}/iframe-parent-external?src={encoded_child_url}",
        parent_server.url("")
    );

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&parent_url).await.expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    let image_id =
        support::find_image_id_by_alt(&capture.content.visible, "Red target").expect("image id");
    let bytes = page
        .screenshot_element(image_id)
        .await
        .expect("screenshot image");

    assert!(bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
    assert_eq!(center_rgb(&bytes), (230, 0, 0));

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn main_screenshot_includes_cross_origin_iframe_surface() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let child_url = child_server
        .url("/iframe-image-child")
        .replace("127.0.0.1", "localhost");
    let encoded_child_url: String = byte_serialize(child_url.as_bytes()).collect();
    let parent_url = format!(
        "{}/iframe-parent-external?src={encoded_child_url}",
        parent_server.url("")
    );

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&parent_url).await.expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    support::find_image_id_by_alt(&capture.content.visible, "Red target").expect("image id");
    let mut bytes = Vec::new();
    for attempt in 0..10 {
        bytes = page.screenshot().await.expect("main screenshot");
        if contains_rgb(&bytes, (230, 0, 0)) {
            break;
        }
        if attempt < 9 {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    assert!(contains_rgb(&bytes, (230, 0, 0)));

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn metadata_reads_image_inside_cross_origin_iframe() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let child_url = child_server
        .url("/iframe-image-child")
        .replace("127.0.0.1", "localhost");
    let encoded_child_url: String = byte_serialize(child_url.as_bytes()).collect();
    let parent_url = format!(
        "{}/iframe-parent-external?src={encoded_child_url}",
        parent_server.url("")
    );

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&parent_url).await.expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    let image_id =
        support::find_image_id_by_alt(&capture.content.visible, "Red target").expect("image id");
    let record = page.metadata(image_id).await.expect("metadata image");

    let MetadataRecord::Image {
        src,
        visibility: Some(visibility),
        bounds: Some(bounds),
        ..
    } = record
    else {
        panic!("unexpected metadata record");
    };
    assert!(src.ends_with("/images/red.svg"), "src: {src}");
    assert!(visibility.present);
    assert!(visibility.visible);
    assert!(visibility.in_viewport);
    assert!(bounds.bottom_right.0 > bounds.top_left.0);
    assert!(bounds.bottom_right.1 > bounds.top_left.1);

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn describes_image_inside_cross_origin_iframe() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.ai.enabled = true;
    config.ai.provider = AiProvider::Anthropic;
    config.ai.model = "test-model".to_string();
    config.ai.api_key_env = "HOME".to_string();
    config.ai.endpoint = Some(parent_server.url("/anthropic-pixel"));

    let child_url = child_server
        .url("/iframe-image-child")
        .replace("127.0.0.1", "localhost");
    let encoded_child_url: String = byte_serialize(child_url.as_bytes()).collect();
    let parent_url = format!(
        "{}/iframe-parent-external?src={encoded_child_url}",
        parent_server.url("")
    );

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&parent_url).await.expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    let image_id =
        support::find_image_id_by_alt(&capture.content.visible, "Red target").expect("image id");
    let description = page.describe(image_id).await.expect("describe image");

    assert_eq!(description.element_id, image_id);
    assert_eq!(description.alt, "Red target");
    assert_eq!(description.description, "center rgb 230 0 0");

    browser.close().await.expect("close browser");
}

fn find_checkbox_id(elements: &[Element]) -> Option<u32> {
    support::find(elements, &|element| match element {
        Element::Input {
            id: Some(id),
            input_type: InputType::Checkbox,
            label: Some(label),
            ..
        } if label == "I'm not a robot" => Some(*id),
        _ => None,
    })
}

fn has_checked_checkbox(elements: &[Element]) -> bool {
    support::find(elements, &|element| match element {
        Element::Input {
            input_type: InputType::Checkbox,
            label: Some(label),
            value,
            ..
        } if label == "I'm not a robot" && value == "true" => Some(()),
        _ => None,
    })
    .is_some()
}

fn center_rgb(bytes: &[u8]) -> (u8, u8, u8) {
    let (frame, info, channels) = decode_png(bytes);
    let center = ((info.height / 2 * info.width + info.width / 2) as usize) * channels;
    (frame[center], frame[center + 1], frame[center + 2])
}

fn contains_rgb(bytes: &[u8], expected: (u8, u8, u8)) -> bool {
    let (frame, _info, channels) = decode_png(bytes);
    frame
        .chunks_exact(channels)
        .any(|pixel| (pixel[0], pixel[1], pixel[2]) == expected)
}

fn decode_png(bytes: &[u8]) -> (Vec<u8>, png::OutputInfo, usize) {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("png info");
    let mut buffer = vec![0; reader.output_buffer_size().expect("png buffer size")];
    let info = reader.next_frame(&mut buffer).expect("png frame");
    let frame = buffer[..info.buffer_size()].to_vec();
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        color_type => panic!("unexpected screenshot color type: {color_type:?}"),
    };
    (frame, info, channels)
}
