//! OpenAI-backed image summarization.

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::error::{Error, Result};

use super::{ImageFormat, ImageSummarizer, factory::header_value, types::require_success};

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
    async fn describe(&self, image_bytes: &[u8], format: ImageFormat) -> Result<String> {
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
            .send()
            .await
            .map_err(|err| Error::AiSummarization {
                reason: format!("openai request failed: {err}"),
            })?;
        let response = require_success(response, "openai")?;
        let response: serde_json::Value =
            response
                .json()
                .await
                .map_err(|err| Error::AiSummarization {
                    reason: format!("openai response parse failed: {err}"),
                })?;
        response["output"][0]["content"][0]["text"]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| Error::AiSummarization {
                reason: "openai response missing text".to_string(),
            })
    }
}
