//! AI image summarization support.

mod anthropic;
mod factory;
mod ollama;
mod openai;
mod types;

pub use anthropic::AnthropicImageSummarizer;
pub use factory::build_image_summarizer;
pub use ollama::OllamaImageSummarizer;
pub use openai::OpenAiImageSummarizer;
pub use types::{ImageFormat, ImageSummarizer, NoopImageSummarizer};

#[cfg(test)]
#[path = "_tests_/ai_tests.rs"]
mod ai_tests;
