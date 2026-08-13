//! Stateful interaction and checkpoint fixture pages.

use axum::http::header::SET_COOKIE;
use axum::response::{Html, IntoResponse};

pub async fn mouse_click_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/mouse_click.html"))
}

pub async fn keyboard_shortcuts_page() -> Html<&'static str> {
    Html(include_str!("../fixtures/keyboard_shortcuts.html"))
}

pub async fn counter_page() -> impl IntoResponse {
    Html(include_str!("../fixtures/counter.html"))
}

pub async fn checkpoint_state_page() -> impl IntoResponse {
    (
        [(
            SET_COOKIE,
            "checkpoint_http_only=secret; HttpOnly; Path=/; SameSite=Lax",
        )],
        Html(include_str!("../fixtures/counter.html")),
    )
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
