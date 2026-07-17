//! Anthropic-backed image summarization.

use std::time::Duration;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::error::{Error, Result};

use super::{
    ImageFormat, ImageSummarizer,
    factory::header_value,
    types::{request_error, require_success, response_parse_error},
};

/// Anthropic summarizer.
#[derive(Debug)]
pub struct AnthropicImageSummarizer {
    client: reqwest::Client,
    model: String,
    api_key: String,
    endpoint: String,
}

impl AnthropicImageSummarizer {
    pub(super) fn new(model: String, api_key: String, endpoint: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            model,
            api_key,
            endpoint,
        }
    }
}

#[async_trait]
impl ImageSummarizer for AnthropicImageSummarizer {
    async fn describe(
        &self,
        image_bytes: &[u8],
        format: ImageFormat,
        timeout: Duration,
    ) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 128,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": "Describe this image briefly for a CLI page summary."},
                    {"type": "image", "source": {
                        "type": "base64",
                        "media_type": format.media_type(),
                        "data": STANDARD.encode(image_bytes)
                    }}
                ]
            }]
        });
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", header_value(&self.api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let response = self
            .client
            .post(&self.endpoint)
            .headers(headers)
            .json(&body)
            .timeout(timeout)
            .send()
            .await;
        let response = match response {
            Ok(response) => response,
            Err(error) => return Err(request_error(error, "anthropic", timeout)),
        };
        let response = require_success(response, "anthropic")?;
        let response = response.json().await;
        let response: serde_json::Value = match response {
            Ok(response) => response,
            Err(error) => return Err(response_parse_error(error, "anthropic", timeout)),
        };
        response["content"][0]["text"]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| Error::AiSummarization {
                reason: "anthropic response missing text".to_string(),
            })
    }
}
