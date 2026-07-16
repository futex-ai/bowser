//! Factory for provider-backed image summarizers.

use std::{env, sync::Arc};

use reqwest::header::HeaderValue;

use crate::{
    config::{AiConfig, AiProvider},
    error::{Error, Result},
};

use super::{
    AnthropicImageSummarizer, ImageSummarizer, OllamaImageSummarizer, OpenAiImageSummarizer,
};

/// Builds an optional configured summarizer.
pub fn build_image_summarizer(config: &AiConfig) -> Option<Arc<dyn ImageSummarizer>> {
    if !config.enabled {
        return None;
    }
    match config.provider {
        AiProvider::Anthropic => {
            let api_key = env::var(&config.api_key_env).ok()?;
            if api_key.is_empty() {
                return None;
            }
            Some(Arc::new(AnthropicImageSummarizer::new(
                config.model.clone(),
                api_key,
                config
                    .endpoint
                    .clone()
                    .unwrap_or_else(|| "https://api.anthropic.com/v1/messages".to_string()),
            )))
        }
        AiProvider::OpenAi => {
            let api_key = env::var(&config.api_key_env).ok()?;
            if api_key.is_empty() {
                return None;
            }
            Some(Arc::new(OpenAiImageSummarizer::new(
                config.model.clone(),
                api_key,
                config
                    .endpoint
                    .clone()
                    .unwrap_or_else(|| "https://api.openai.com/v1/responses".to_string()),
            )))
        }
        AiProvider::Ollama => Some(Arc::new(OllamaImageSummarizer::new(
            config.model.clone(),
            config
                .endpoint
                .clone()
                .unwrap_or_else(|| "http://127.0.0.1:11434/api/generate".to_string()),
        ))),
    }
}

pub(super) fn header_value(value: impl AsRef<str>) -> Result<HeaderValue> {
    HeaderValue::from_str(value.as_ref()).map_err(|err| Error::AiSummarization {
        reason: format!("invalid auth header: {err}"),
    })
}
