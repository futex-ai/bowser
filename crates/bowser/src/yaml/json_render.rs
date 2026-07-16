//! JSON rendering helpers for captures.

use crate::error::{Error, Result};
use crate::model::PageCapture;

/// Serializes a capture to JSON.
pub fn to_json(capture: &PageCapture) -> Result<String> {
    serde_json::to_string_pretty(capture).map_err(|err| Error::Config {
        reason: format!("failed to serialize JSON: {err}"),
    })
}

/// Deserializes a capture from JSON.
pub fn from_json(json: &str) -> Result<PageCapture> {
    serde_json::from_str(json).map_err(|err| Error::Config {
        reason: format!("failed to deserialize JSON: {err}"),
    })
}
