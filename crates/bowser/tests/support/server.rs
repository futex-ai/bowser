//! Local HTTP fixture server helpers.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path as AxumPath, State};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use tokio::sync::Mutex;

use super::fixtures::{
    anthropic_mock, anthropic_pixel_mock, aria_hidden_visible_page, automation_globals_page,
    chatty_page, checkerboard_image, click_reveals_images_page, counter_page,
    delayed_redirect_page, download_cookie_page, download_cookie_report, download_redirect,
    download_report, dynamic_page, help_page, iframe_checkbox_child_page, iframe_child_page,
    iframe_form_child_page, iframe_image_child_page, iframe_parent_external_page,
    iframe_parent_page, image_page, item_page, keyboard_shortcuts_page, long_page,
    modal_overlay_page, mouse_click_page, next_page, orders_long_page, profile_state_page,
    red_vector_image, simple_page, slow_anthropic_mock, slow_data, slow_history_page, slow_page,
    slow_redirect, submit_page, wheel_page,
};

pub struct TestServer {
    base_url: String,
    task: tokio::task::JoinHandle<()>,
}

impl TestServer {
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Clone, Default)]
struct TestServerState {
    tracked_slow_page_hits: Arc<Mutex<HashMap<String, usize>>>,
}

pub async fn spawn_server() -> TestServer {
    let state = TestServerState::default();
    let app = Router::new()
        .route("/simple", get(simple_page))
        .route("/long", get(long_page))
        .route("/orders-long", get(orders_long_page))
        .route("/aria-hidden-visible", get(aria_hidden_visible_page))
        .route("/modal-overlay", get(modal_overlay_page))
        .route("/chatty", get(chatty_page))
        .route("/automation-globals", get(automation_globals_page))
        .route("/slow-page", get(slow_page))
        .route("/tracked-slow-page/{token}", get(tracked_slow_page))
        .route("/tracked-slow-page-hit/{token}", get(tracked_slow_page_hit))
        .route(
            "/tracked-slow-page-count/{token}",
            get(tracked_slow_page_count),
        )
        .route("/slow-history", get(slow_history_page))
        .route("/slow-redirect", get(slow_redirect))
        .route("/delayed-redirect", get(delayed_redirect_page))
        .route("/next", get(next_page))
        .route("/download/redirect", get(download_redirect))
        .route("/download/report.csv", get(download_report))
        .route("/download/cookie-page", get(download_cookie_page))
        .route("/download/cookie.csv", get(download_cookie_report))
        .route("/help", get(help_page))
        .route("/dynamic", get(dynamic_page))
        .route("/iframe-parent", get(iframe_parent_page))
        .route("/iframe-parent-external", get(iframe_parent_external_page))
        .route("/iframe-child", get(iframe_child_page))
        .route("/iframe-checkbox-child", get(iframe_checkbox_child_page))
        .route("/iframe-image-child", get(iframe_image_child_page))
        .route("/iframe-form-child", get(iframe_form_child_page))
        .route("/image-page", get(image_page))
        .route("/click-reveals-images", get(click_reveals_images_page))
        .route("/images/checkerboard.png", get(checkerboard_image))
        .route("/images/red.svg", get(red_vector_image))
        .route("/anthropic", post(anthropic_mock))
        .route("/anthropic-pixel", post(anthropic_pixel_mock))
        .route("/anthropic-slow", post(slow_anthropic_mock))
        .route("/slow-data/{id}", get(slow_data))
        .route("/mouse-click", get(mouse_click_page))
        .route("/keyboard-shortcuts", get(keyboard_shortcuts_page))
        .route("/counter", get(counter_page))
        .route("/wheel-page", get(wheel_page))
        .route("/submit-page", get(submit_page))
        .route("/profile-state", get(profile_state_page))
        .route("/item/{id}", get(item_page))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let addr: SocketAddr = listener.local_addr().expect("local addr");
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    TestServer {
        base_url: format!("http://{addr}"),
        task,
    }
}

async fn tracked_slow_page(
    State(state): State<TestServerState>,
    AxumPath(token): AxumPath<String>,
) -> impl IntoResponse {
    {
        let mut hits = state.tracked_slow_page_hits.lock().await;
        let hit_count = hits.entry(token).or_default();
        *hit_count += 1;
    }
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;
    Html("<html><head><title>Slow Page</title></head><body><p>Slow page</p></body></html>")
}

async fn tracked_slow_page_hit(
    State(state): State<TestServerState>,
    AxumPath(token): AxumPath<String>,
) -> &'static str {
    if state
        .tracked_slow_page_hits
        .lock()
        .await
        .get(&token)
        .copied()
        .unwrap_or_default()
        > 0
    {
        "true"
    } else {
        "false"
    }
}

async fn tracked_slow_page_count(
    State(state): State<TestServerState>,
    AxumPath(token): AxumPath<String>,
) -> String {
    state
        .tracked_slow_page_hits
        .lock()
        .await
        .get(&token)
        .copied()
        .unwrap_or_default()
        .to_string()
}
