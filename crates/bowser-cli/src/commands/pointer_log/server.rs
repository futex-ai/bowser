//! Axum routes for the local pointer telemetry capture page.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{StatusCode, header::CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use tokio::net::TcpListener;

use crate::error::{CliError, Result};

use super::event::{PointerEventBatch, PointerLogAccepted};
use super::html::{APP_JS, INDEX_HTML, STYLE_CSS};
use super::log::PointerEventLog;

/// Shared route state for the local pointer telemetry server.
#[derive(Clone)]
pub(super) struct PointerLogState {
    /// Event log sink used by the `/events` route.
    pub(super) log: Arc<dyn PointerEventLog + Send + Sync>,
}

pub(super) async fn serve(
    listener: TcpListener,
    addr: SocketAddr,
    log: Arc<dyn PointerEventLog + Send + Sync>,
) -> Result<()> {
    let app = app(log);
    match axum::serve(listener, app).await {
        Ok(()) => Ok(()),
        Err(source) => Err(CliError::PointerLogServe { addr, source }),
    }
}

fn app(log: Arc<dyn PointerEventLog + Send + Sync>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/pointer-log.css", get(style))
        .route("/pointer-log.js", get(script))
        .route("/events", post(append_events))
        .with_state(PointerLogState { log })
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn style() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/css; charset=utf-8")], STYLE_CSS)
}

async fn script() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        APP_JS,
    )
}

pub(super) async fn append_events(
    State(state): State<PointerLogState>,
    Json(batch): Json<PointerEventBatch>,
) -> Response {
    if let Err(err) = batch.validate() {
        return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
    }
    match state.log.append_batch(&batch) {
        Ok(accepted) => Json(PointerLogAccepted { accepted }).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}
