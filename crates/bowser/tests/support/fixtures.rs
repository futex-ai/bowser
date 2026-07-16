//! HTML fixture pages served to browser integration tests.

use std::io::Cursor;

use axum::Json;
use axum::extract::Path as AxumPath;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE, SET_COOKIE};
use axum::response::{Html, IntoResponse, Redirect};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::{Value, json};

pub async fn simple_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/simple.html"))
}

pub async fn long_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/long.html"))
}

pub async fn orders_long_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/orders_long.html"))
}

pub async fn aria_hidden_visible_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/aria_hidden_visible.html"))
}

pub async fn modal_overlay_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/modal_overlay.html"))
}

pub async fn chatty_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/chatty.html"))
}

pub async fn automation_globals_page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Automation Globals</title>
  <script>
    window.cdc_bowser_test = true;
    window.__webdriver_bowser_test = true;
  </script>
</head>
<body>
  <p>Automation globals fixture</p>
</body>
</html>"#,
    )
}

pub async fn slow_page() -> impl IntoResponse {
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;
    Html("<html><head><title>Slow Page</title></head><body><p>Slow page</p></body></html>")
}

pub async fn slow_redirect() -> Redirect {
    tokio::time::sleep(std::time::Duration::from_secs(12)).await;
    Redirect::temporary("/counter")
}

pub async fn slow_history_page() -> impl IntoResponse {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    (
        [(CACHE_CONTROL, "no-store")],
        Html(
            "<html><head><title>Slow History</title><script>window.addEventListener('unload', () => {});</script></head><body><p>Slow history</p></body></html>",
        ),
    )
}

pub async fn delayed_redirect_page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Delayed Redirect</title>
  <script>
    setTimeout(() => {
      window.location.href = "/counter";
    }, 250);
  </script>
</head>
<body>
  <p>Delayed redirect</p>
</body>
</html>"#,
    )
}

pub async fn next_page() -> Html<&'static str> {
    Html("<html><head><title>Next</title></head><body><p>Next page</p></body></html>")
}

pub async fn download_redirect() -> Redirect {
    Redirect::temporary("/download/report.csv")
}

pub async fn download_report() -> impl IntoResponse {
    (
        [
            (CONTENT_TYPE, "text/csv"),
            (CONTENT_DISPOSITION, "attachment; filename=\"report.csv\""),
        ],
        "name,value\nalpha,1\n",
    )
}

pub async fn download_cookie_page() -> impl IntoResponse {
    (
        [(SET_COOKIE, "bowser_download=ok; Path=/; SameSite=Lax")],
        Html(
            "<html><head><title>Download Cookie</title></head><body><a href=\"/download/cookie.csv\">download</a></body></html>",
        ),
    )
}

pub async fn download_cookie_report(headers: HeaderMap) -> impl IntoResponse {
    let allowed = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("bowser_download=ok"))
        .unwrap_or(false);
    if !allowed {
        return (
            StatusCode::UNAUTHORIZED,
            [(CONTENT_TYPE, "text/plain"), (CONTENT_DISPOSITION, "")],
            "missing cookie",
        );
    }
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "text/csv"),
            (CONTENT_DISPOSITION, "attachment; filename=\"cookie.csv\""),
        ],
        "name,value\ncookie,1\n",
    )
}

pub async fn help_page() -> Html<&'static str> {
    Html("<html><head><title>Help</title></head><body><p>Help page</p></body></html>")
}

pub async fn dynamic_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/dynamic.html"))
}

pub async fn slow_data(AxumPath(_id): AxumPath<String>) -> impl IntoResponse {
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    (StatusCode::OK, "ok")
}

pub async fn iframe_parent_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/iframe_parent.html"))
}

#[derive(Deserialize)]
pub struct IframeParentQuery {
    src: String,
}

pub async fn iframe_parent_external_page(
    Query(query): Query<IframeParentQuery>,
) -> impl IntoResponse {
    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Iframe Parent External</title></head><body><button>Outer</button><iframe src=\"{}\" title=\"child frame\"></iframe><a href=\"/after\">After</a></body></html>",
        query.src
    ))
}

pub async fn iframe_child_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/iframe_child.html"))
}

pub async fn iframe_checkbox_child_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/iframe_checkbox_child.html"))
}

pub async fn iframe_image_child_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/iframe_image_child.html"))
}

pub async fn iframe_form_child_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/iframe_form_child.html"))
}

pub async fn image_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/image_page.html"))
}

pub async fn click_reveals_images_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/click_reveals_images.html"))
}

pub async fn checkerboard_image() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/png")],
        [
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 4, 0, 0, 0, 181, 28, 12, 2, 0, 0, 0, 11, 73, 68, 65, 84, 120, 218, 99, 252, 255, 31,
            0, 3, 3, 2, 0, 239, 86, 148, 171, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ],
    )
}

pub async fn red_vector_image() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "image/svg+xml")],
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40">
<rect width="80" height="40" fill="#e60000"/>
</svg>"##,
    )
}

pub async fn anthropic_mock(headers: HeaderMap, Json(body): Json<Value>) -> Json<Value> {
    assert!(headers.contains_key("x-api-key"));
    assert_eq!(body["messages"][0]["content"][0]["type"], "text");
    Json(json!({ "content": [{ "text": "anthropic mock" }] }))
}

pub async fn anthropic_pixel_mock(headers: HeaderMap, Json(body): Json<Value>) -> Json<Value> {
    assert!(headers.contains_key("x-api-key"));
    assert_eq!(body["messages"][0]["content"][0]["type"], "text");
    let description = image_center_description(&body);
    Json(json!({ "content": [{ "text": description }] }))
}

pub async fn slow_anthropic_mock(headers: HeaderMap, Json(body): Json<Value>) -> Json<Value> {
    assert!(headers.contains_key("x-api-key"));
    assert_eq!(body["messages"][0]["content"][0]["type"], "text");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    Json(json!({ "content": [{ "text": "anthropic mock" }] }))
}

fn image_center_description(body: &Value) -> String {
    let data = body["messages"][0]["content"][1]["source"]["data"]
        .as_str()
        .expect("image data");
    let bytes = STANDARD.decode(data).expect("base64 image data");
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
    let center = ((info.height / 2 * info.width + info.width / 2) as usize) * channels;
    let red = frame[center];
    let green = frame[center + 1];
    let blue = frame[center + 2];
    format!("center rgb {red} {green} {blue}")
}

pub async fn mouse_click_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/mouse_click.html"))
}

pub async fn keyboard_shortcuts_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/keyboard_shortcuts.html"))
}

pub async fn counter_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/counter.html"))
}

pub async fn wheel_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/wheel_page.html"))
}

pub async fn submit_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/submit_page.html"))
}

pub async fn profile_state_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/profile_state.html"))
}

pub async fn item_page(AxumPath(id): AxumPath<u32>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Html(format!(
            "<html><head><title>Item {id}</title></head><body><p>Item {id}</p></body></html>"
        )),
    )
}
