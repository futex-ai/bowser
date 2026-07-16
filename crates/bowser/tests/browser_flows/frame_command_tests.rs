use std::io::Cursor;

use bowser::{
    Browser, BrowserEngine, Element, ElementVisibility, InputType, MetadataRecord, ScrollTarget,
};
use tempfile::tempdir;
use url::form_urlencoded::byte_serialize;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn form_commands_work_inside_cross_origin_iframe() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&form_parent_url(&parent_server, &child_server))
        .await
        .expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    let message_id = support::find_input_id_by_name(&capture.content.visible, "message")
        .expect("message input id");
    page.type_text(message_id, "hello")
        .await
        .expect("type message");

    let after_type = page.capture().await.expect("capture after type");
    assert_eq!(
        input_value(&after_type.content.visible, "message"),
        Some("hello".to_string())
    );
    let message_id = support::find_input_id_by_name(&after_type.content.visible, "message")
        .expect("message input id after type");
    page.clear(message_id).await.expect("clear message");

    let after_clear = page.capture().await.expect("capture after clear");
    assert_eq!(
        input_value(&after_clear.content.visible, "message"),
        Some(String::new())
    );
    let message_id = support::find_input_id_by_name(&after_clear.content.visible, "message")
        .expect("message input id after clear");
    let choice_id =
        support::find_input_id_by_name(&after_clear.content.visible, "choice").expect("choice id");
    let send_id = find_button_id(&after_clear.content.visible, "Send").expect("send id");

    page.type_text(message_id, "sent")
        .await
        .expect("type submitted value");
    page.select_option(choice_id, "Two")
        .await
        .expect("select option");
    page.submit(send_id).await.expect("submit form");

    let after_submit = page.capture().await.expect("capture after submit");
    assert!(support::contains_text(
        &after_submit.content.visible,
        "submitted:sent:Two"
    ));

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn visual_commands_work_inside_cross_origin_iframe() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&form_parent_url(&parent_server, &child_server))
        .await
        .expect("navigate parent");

    let capture = page.capture().await.expect("capture parent");
    let color_id = find_button_id(&capture.content.visible, "Color block").expect("color id");
    let screenshot = page
        .screenshot_element(color_id)
        .await
        .expect("screenshot color button");
    assert!(contains_rgb(&screenshot, (0, 204, 0)));

    let deep_id = find_button_id(&capture.content.visible, "Deep target").expect("deep id");
    page.scroll(ScrollTarget::ToElement(deep_id))
        .await
        .expect("scroll to deep target");
    let record = page.metadata(deep_id).await.expect("deep metadata");
    assert_visible(record);

    browser.close().await.expect("close browser");
}

fn form_parent_url(
    parent_server: &support::TestServer,
    child_server: &support::TestServer,
) -> String {
    let child_url = child_server
        .url("/iframe-form-child")
        .replace("127.0.0.1", "localhost");
    let encoded_child_url: String = byte_serialize(child_url.as_bytes()).collect();
    format!(
        "{}/iframe-parent-external?src={encoded_child_url}",
        parent_server.url("")
    )
}

fn find_button_id(elements: &[Element], target_text: &str) -> Option<u32> {
    support::find(elements, &|element| match element {
        Element::Button { id, text, .. } if text == target_text => Some(*id),
        _ => None,
    })
}

fn input_value(elements: &[Element], target_name: &str) -> Option<String> {
    support::find(elements, &|element| match element {
        Element::Input {
            name: Some(name),
            value,
            input_type: InputType::Text | InputType::Select,
            ..
        } if name == target_name => Some(value.clone()),
        _ => None,
    })
}

fn assert_visible(record: MetadataRecord) {
    let visibility = match record {
        MetadataRecord::Button {
            visibility: Some(visibility),
            ..
        } => visibility,
        record => panic!("unexpected metadata record: {record:?}"),
    };
    assert_eq!(
        visibility,
        ElementVisibility {
            present: true,
            in_viewport: true,
            obscured: false,
            enabled: true,
            visible: true,
            clickable: true,
        }
    );
}

fn contains_rgb(bytes: &[u8], expected: (u8, u8, u8)) -> bool {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("png info");
    let mut buffer = vec![0; reader.output_buffer_size().expect("png buffer size")];
    let info = reader.next_frame(&mut buffer).expect("png frame");
    let frame = &buffer[..info.buffer_size()];
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        color_type => panic!("unexpected screenshot color type: {color_type:?}"),
    };
    frame
        .chunks_exact(channels)
        .any(|pixel| (pixel[0], pixel[1], pixel[2]) == expected)
}
