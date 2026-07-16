//! Shared AI image summarization contracts and value types.

use async_trait::async_trait;

use crate::error::{Error, Result};

/// Rejects a provider response when its HTTP status is not successful.
pub(super) fn require_success(
    response: reqwest::Response,
    provider: &str,
) -> Result<reqwest::Response> {
    match response.error_for_status() {
        Ok(response) => Ok(response),
        Err(error) => Err(Error::AiSummarization {
            reason: format!("{provider} request failed: {error}"),
        }),
    }
}

/// Image format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG image bytes.
    Png,
    /// JPEG image bytes.
    Jpeg,
    /// WEBP image bytes.
    Webp,
}

impl ImageFormat {
    pub(super) fn media_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

/// AI image summarization boundary.
#[async_trait]
pub trait ImageSummarizer: Send + Sync {
    /// Produces a short textual description for the rendered image.
    async fn describe(&self, image_bytes: &[u8], format: ImageFormat) -> Result<String>;
}

/// Disabled summarizer.
#[derive(Debug, Default)]
pub struct NoopImageSummarizer;

#[async_trait]
impl ImageSummarizer for NoopImageSummarizer {
    async fn describe(&self, _image_bytes: &[u8], _format: ImageFormat) -> Result<String> {
        Err(Error::AiSummarization {
            reason: "AI image summarization is disabled".to_string(),
        })
    }
}
