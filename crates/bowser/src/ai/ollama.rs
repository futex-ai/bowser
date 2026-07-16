//! Ollama-backed image summarization.

use std::time::Duration;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::error::{Error, Result};

use super::{
    ImageFormat, ImageSummarizer,
    types::{request_error, require_success, response_parse_error},
};

/// Ollama summarizer.
#[derive(Debug)]
pub struct OllamaImageSummarizer {
    client: reqwest::Client,
    model: String,
    endpoint: String,
}

impl OllamaImageSummarizer {
    pub(super) fn new(model: String, endpoint: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            model,
            endpoint,
        }
    }
}

#[async_trait]
impl ImageSummarizer for OllamaImageSummarizer {
    async fn describe(
        &self,
        image_bytes: &[u8],
        _format: ImageFormat,
        timeout: Duration,
    ) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "prompt": "Describe this image briefly for a CLI page summary.",
            "images": [STANDARD.encode(image_bytes)],
            "stream": false
        });
        let response = self
            .client
            .post(&self.endpoint)
            .json(&body)
            .timeout(timeout)
            .send()
            .await;
        let response = match response {
            Ok(response) => response,
            Err(error) => return Err(request_error(error, "ollama", timeout)),
        };
        let response = require_success(response, "ollama")?;
        let response = response.json().await;
        let response: serde_json::Value = match response {
            Ok(response) => response,
            Err(error) => return Err(response_parse_error(error, "ollama", timeout)),
        };
        response["response"]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| Error::AiSummarization {
                reason: "ollama response missing text".to_string(),
            })
    }
}
