//! Local browser and HTTP fixtures for favicon integration tests.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use base64::{Engine, engine::general_purpose::STANDARD};
use futures::stream;
use serde::Deserialize;

use super::images::{
    green_png_response, ico_response, image_response, red_png_response, solid_png, svg_response,
};

pub(super) struct FaviconServer {
    base_url: String,
    slow_chunks: Arc<Mutex<Vec<Arc<AtomicUsize>>>>,
    slow_requests: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}

impl FaviconServer {
    pub(super) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub(super) fn slow_chunks_after(&self, first: usize) -> usize {
        self.slow_chunks
            .lock()
            .expect("slow counters")
            .iter()
            .skip(first)
            .map(|chunks| chunks.load(Ordering::SeqCst))
            .sum()
    }

    pub(super) async fn wait_for_slow_requests(&self, count: usize) {
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while self.slow_requests() < count {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("fixture received expected image request");
    }

    pub(super) fn slow_requests(&self) -> usize {
        self.slow_requests.load(Ordering::SeqCst)
    }
}

impl Drop for FaviconServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Clone)]
struct ServerState {
    root_icon: bool,
    slow_chunks: Arc<Mutex<Vec<Arc<AtomicUsize>>>>,
    slow_requests: Arc<AtomicUsize>,
}

pub(super) async fn spawn_server(root_icon: bool) -> FaviconServer {
    let slow_chunks = Arc::new(Mutex::new(Vec::new()));
    let slow_requests = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/relative/page", get(relative_page))
        .route("/assets/exact.png", get(red_png_response))
        .route("/root-only", get(root_only_page))
        .route("/favicon.ico", get(root_icon_response))
        .route("/formats/svg", get(svg_page))
        .route("/formats/icon.svg", get(svg_response))
        .route("/formats/ico", get(ico_page))
        .route("/formats/icon.ico", get(ico_response))
        .route("/data", get(data_page))
        .route("/auth/seed", get(seed_cookie))
        .route("/auth/icon.png", get(authenticated_icon))
        .route("/cross", get(cross_origin_page))
        .route("/broken", get(broken_page))
        .route("/slow", get(slow_page))
        .route("/racing", get(racing_page))
        .route("/slow-icon.png", get(slow_icon))
        .route("/replacement.png", get(green_png_response))
        .route("/redirect/{remaining}", get(redirect_icon))
        .with_state(ServerState {
            root_icon,
            slow_chunks: Arc::clone(&slow_chunks),
            slow_requests: Arc::clone(&slow_requests),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind favicon server");
    let address: SocketAddr = listener.local_addr().expect("favicon server address");
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    FaviconServer {
        base_url: format!("http://{address}"),
        slow_chunks,
        slow_requests,
        task,
    }
}

async fn relative_page() -> Html<&'static str> {
    Html(
        r#"<html><head><base href="/assets/"><title>Relative</title>
        <link rel="icon" sizes="16x16" href="wrong.png">
        <link rel="shortcut icon" sizes="32x32" href="exact.png">
        </head><body>relative</body></html>"#,
    )
}

async fn root_only_page() -> Html<&'static str> {
    Html("<html><head><title>Root only</title></head><body>root</body></html>")
}

async fn svg_page() -> Html<&'static str> {
    Html(
        r#"<html><head><title>SVG</title><link rel="icon" sizes="any" href="/formats/icon.svg"></head><body>svg</body></html>"#,
    )
}

async fn ico_page() -> Html<&'static str> {
    Html(
        r#"<html><head><title>ICO</title><link rel="icon" sizes="32x32" href="/formats/icon.ico"></head><body>ico</body></html>"#,
    )
}

async fn data_page() -> Html<String> {
    let data = STANDARD.encode(solid_png([20, 40, 220, 255]));
    Html(format!(
        r#"<html><head><title>Data</title><link rel="icon" sizes="32x32" href="data:image/png;base64,{data}"></head><body>data</body></html>"#
    ))
}

async fn seed_cookie() -> impl IntoResponse {
    (
        [(SET_COOKIE, "favicon_auth=ok; Path=/; SameSite=Lax")],
        Html("<html><head><title>Seed</title></head><body>seed</body></html>"),
    )
}

#[derive(Deserialize)]
struct CrossQuery {
    icon: String,
}

async fn cross_origin_page(Query(query): Query<CrossQuery>) -> impl IntoResponse {
    (
        [(
            CONTENT_SECURITY_POLICY,
            "default-src 'self'; connect-src 'none'; img-src * data:",
        )],
        Html(format!(
            r#"<html><head><title>Cross</title><link rel="icon" sizes="32x32" href="{}"></head><body>cross</body></html>"#,
            query.icon
        )),
    )
}

async fn authenticated_icon(headers: HeaderMap) -> Response {
    let authenticated = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("favicon_auth=ok"));
    if authenticated {
        image_response("image/png", solid_png([180, 30, 190, 255]))
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn broken_page() -> Html<&'static str> {
    Html(
        r#"<html><head><title>Broken</title><link rel="icon" href="/missing.png"></head><body>broken</body></html>"#,
    )
}

async fn slow_page() -> Html<&'static str> {
    Html(
        r#"<html><head><title>Slow</title><link rel="icon" href="/slow-icon.png"></head><body>slow</body></html>"#,
    )
}

async fn racing_page() -> Html<&'static str> {
    Html(
        "<html><head><title>Racing</title><link rel='icon' href='/slow-icon.png?complete=true'></head><body>race</body></html>",
    )
}

#[derive(Default, Deserialize)]
struct SlowQuery {
    #[serde(default)]
    complete: bool,
}

async fn slow_icon(State(state): State<ServerState>, Query(query): Query<SlowQuery>) -> Response {
    let chunks = Arc::new(AtomicUsize::new(0));
    state
        .slow_chunks
        .lock()
        .expect("slow counters")
        .push(Arc::clone(&chunks));
    state.slow_requests.fetch_add(1, Ordering::SeqCst);
    if query.complete {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return red_png_response().await;
    }
    let body = stream::unfold(0, move |index| {
        let chunks = Arc::clone(&chunks);
        async move {
            if index == 200 {
                return None;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            chunks.fetch_add(1, Ordering::SeqCst);
            Some((
                Ok::<Bytes, Infallible>(Bytes::from_static(b"unfinished-icon")),
                index + 1,
            ))
        }
    });
    Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .header(CACHE_CONTROL, "no-store")
        .body(Body::from_stream(body))
        .expect("slow icon response")
}

async fn redirect_icon(AxumPath(remaining): AxumPath<u8>) -> Response {
    if remaining == 0 {
        return red_png_response().await;
    }
    Redirect::temporary(&format!("/redirect/{}", remaining - 1)).into_response()
}

async fn root_icon_response(State(state): State<ServerState>) -> Response {
    if state.root_icon {
        green_png_response().await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
