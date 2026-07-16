//! Parsing helpers for config values and overrides.

use crate::config::types::{AiProvider, BrowserConfig};
use crate::error::{Error, Result};
use crate::model::Viewport;

pub(super) fn parse_bool(name: &str, value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Error::config(format!("{name} must be `true` or `false`"))),
    }
}

pub(super) fn parse_u64(name: &str, value: &str) -> Result<u64> {
    value
        .parse::<u64>()
        .map_err(|_| Error::config(format!("{name} must be an integer")))
}

pub(super) fn parse_usize(name: &str, value: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .map_err(|_| Error::config(format!("{name} must be an integer")))
}

pub(crate) fn parse_viewport(value: &str) -> Result<Viewport> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| Error::config("viewport must use WIDTHxHEIGHT"))?;
    let viewport = Viewport {
        width: width
            .parse()
            .map_err(|_| Error::config("viewport width must be numeric"))?,
        height: height
            .parse()
            .map_err(|_| Error::config("viewport height must be numeric"))?,
    };
    validate_viewport(&viewport)?;
    Ok(viewport)
}

pub(crate) fn validate_config(config: &BrowserConfig) -> Result<()> {
    validate_viewport(&config.viewport)
}

fn validate_viewport(viewport: &Viewport) -> Result<()> {
    if viewport.width == 0 || viewport.height == 0 {
        return Err(Error::config(
            "viewport width and height must be greater than zero",
        ));
    }
    Ok(())
}

pub(super) fn parse_provider(value: &str) -> Result<AiProvider> {
    match value {
        "anthropic" => Ok(AiProvider::Anthropic),
        "openai" => Ok(AiProvider::OpenAi),
        "ollama" => Ok(AiProvider::Ollama),
        _ => Err(Error::config(
            "AI provider must be anthropic, openai, or ollama",
        )),
    }
}
