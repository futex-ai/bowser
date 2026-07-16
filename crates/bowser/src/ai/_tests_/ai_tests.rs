use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Value, json};

use super::{ImageFormat, build_image_summarizer};
use crate::config::{AiConfig, AiProvider};

#[test]
fn build_image_summarizer_skips_missing_credentials() {
    let mut anthropic = sample_config(AiProvider::Anthropic);
    anthropic.api_key_env = "BOWSER_TEST_MISSING_KEY".to_string();
    assert!(build_image_summarizer(&anthropic).is_none());

    let mut disabled = sample_config(AiProvider::OpenAi);
    disabled.enabled = false;
    assert!(build_image_summarizer(&disabled).is_none());

    let ollama = sample_config(AiProvider::Ollama);
    assert!(build_image_summarizer(&ollama).is_some());
}

#[tokio::test]
async fn provider_clients_use_mock_endpoints() {
    let server = mock_server().await;

    let mut anthropic = sample_config(AiProvider::Anthropic);
    anthropic.endpoint = Some(server.url("/anthropic"));
    let anthropic_client = build_image_summarizer(&anthropic).expect("anthropic client");
    assert_eq!(
        anthropic_client
            .describe(b"png", ImageFormat::Png)
            .await
            .expect("anthropic response"),
        "anthropic mock"
    );

    let mut openai = sample_config(AiProvider::OpenAi);
    openai.endpoint = Some(server.url("/openai"));
    let openai_client = build_image_summarizer(&openai).expect("openai client");
    assert_eq!(
        openai_client
            .describe(b"png", ImageFormat::Png)
            .await
            .expect("openai response"),
        "openai mock"
    );

    let mut ollama = sample_config(AiProvider::Ollama);
    ollama.endpoint = Some(server.url("/ollama"));
    let ollama_client = build_image_summarizer(&ollama).expect("ollama client");
    assert_eq!(
        ollama_client
            .describe(b"png", ImageFormat::Png)
            .await
            .expect("ollama response"),
        "ollama mock"
    );
}

#[tokio::test]
async fn provider_clients_report_http_error_statuses() {
    let server = mock_server().await;

    for provider in [
        AiProvider::Anthropic,
        AiProvider::OpenAi,
        AiProvider::Ollama,
    ] {
        let mut config = sample_config(provider);
        config.endpoint = Some(server.url("/provider-error"));
        let client = build_image_summarizer(&config).expect("provider client");

        let error = client
            .describe(b"png", ImageFormat::Png)
            .await
            .expect_err("provider HTTP error");

        assert!(
            error.to_string().contains("429"),
            "provider error omitted HTTP status: {error}"
        );
    }
}

fn sample_config(provider: AiProvider) -> AiConfig {
    AiConfig {
        enabled: true,
        provider,
        model: "test-model".to_string(),
        api_key_env: "HOME".to_string(),
        endpoint: None,
    }
}

struct MockServer {
    base_url: String,
    task: tokio::task::JoinHandle<()>,
}

impl MockServer {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn mock_server() -> MockServer {
    let app = Router::new()
        .route("/anthropic", post(anthropic_handler))
        .route("/openai", post(openai_handler))
        .route("/ollama", post(ollama_handler))
        .route("/provider-error", post(provider_error_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock server");
    let addr = listener.local_addr().expect("mock server addr");
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    MockServer {
        base_url: format!("http://{addr}"),
        task,
    }
}

async fn anthropic_handler(headers: HeaderMap, Json(body): Json<Value>) -> Json<Value> {
    assert!(headers.contains_key("x-api-key"));
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["messages"][0]["content"][0]["type"], "text");
    Json(json!({ "content": [{ "text": "anthropic mock" }] }))
}

async fn openai_handler(headers: HeaderMap, Json(body): Json<Value>) -> Json<Value> {
    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(authorization.starts_with("Bearer "));
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["input"][0]["content"][0]["type"], "input_text");
    Json(json!({ "output": [{ "content": [{ "text": "openai mock" }] }] }))
}

async fn ollama_handler(Json(body): Json<Value>) -> Json<Value> {
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["stream"], false);
    Json(json!({ "response": "ollama mock" }))
}

async fn provider_error_handler() -> (StatusCode, Json<Value>) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({ "error": { "message": "rate limited" } })),
    )
}
