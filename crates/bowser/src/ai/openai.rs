//! OpenAI-backed image summarization.

use std::time::Duration;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::error::{Error, Result};

use super::{
    ImageFormat, ImageSummarizer,
    factory::header_value,
    types::{request_error, require_success, response_parse_error},
};

/// OpenAI summarizer.
#[derive(Debug)]
pub struct OpenAiImageSummarizer {
    client: reqwest::Client,
    model: String,
    api_key: String,
    endpoint: String,
}

impl OpenAiImageSummarizer {
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
impl ImageSummarizer for OpenAiImageSummarizer {
    async fn describe(
        &self,
        image_bytes: &[u8],
        format: ImageFormat,
        timeout: Duration,
    ) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "input": [{
                "role": "user",
                "content": [
                    {"type": "input_text", "text": "Describe this image briefly for a CLI page summary."},
                    {"type": "input_image", "image_url": format!("data:{};base64,{}", format.media_type(), STANDARD.encode(image_bytes))}
                ]
            }]
        });
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            header_value(format!("Bearer {}", self.api_key))?,
        );
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
            Err(error) => return Err(request_error(error, "openai", timeout)),
        };
        let response = require_success(response, "openai")?;
        let response = response.json().await;
        let response: serde_json::Value = match response {
            Ok(response) => response,
            Err(error) => return Err(response_parse_error(error, "openai", timeout)),
        };
        response["output"][0]["content"][0]["text"]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| Error::AiSummarization {
                reason: "openai response missing text".to_string(),
            })
    }
}
