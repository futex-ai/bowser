//! Environment and CLI override application.

use std::env;
use std::path::PathBuf;
use std::time::Duration;

use super::types::{BrowserConfig, ConfigOverrides};
use super::validation::{parse_bool, parse_provider, parse_u64, parse_usize, parse_viewport};
use crate::error::Result;

pub(super) fn apply_env(config: &mut BrowserConfig) -> Result<()> {
    if let Some(value) = read_env("BOWSER_CHROME_PATH") {
        config.chrome_path = Some(PathBuf::from(value));
    }
    if let Some(value) = read_env("BOWSER_CHROME_ARGS") {
        config.chrome_args = value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect();
    }
    if let Some(value) = read_env("BOWSER_SESSION") {
        config.session.id = Some(value);
    }
    if let Some(value) = read_env("BOWSER_SESSION_DIR") {
        config.session.dir = Some(PathBuf::from(value));
    }
    if let Some(value) = read_env("BOWSER_SESSION_IDLE_TTL") {
        config.session.idle_ttl =
            Duration::from_secs(parse_u64("BOWSER_SESSION_IDLE_TTL", &value)?);
    }
    if let Some(value) = read_env("BOWSER_OUTPUT_TRUNCATE") {
        config.output.truncate = parse_bool("BOWSER_OUTPUT_TRUNCATE", &value)?;
    }
    if let Some(value) = read_env("BOWSER_OUTPUT_INCLUDE_HIDDEN") {
        config.output.include_hidden = parse_bool("BOWSER_OUTPUT_INCLUDE_HIDDEN", &value)?;
    }
    if let Some(value) = read_env("BOWSER_OUTPUT_MAX_CHILDREN") {
        config.output.max_children = parse_usize("BOWSER_OUTPUT_MAX_CHILDREN", &value)?;
    }
    if let Some(value) = read_env("BOWSER_OUTPUT_MAX_LIST_ITEMS") {
        config.output.max_list_items = parse_usize("BOWSER_OUTPUT_MAX_LIST_ITEMS", &value)?;
    }
    if let Some(value) = read_env("BOWSER_OUTPUT_MAX_TABLE_ROWS") {
        config.output.max_table_rows = parse_usize("BOWSER_OUTPUT_MAX_TABLE_ROWS", &value)?;
    }
    if let Some(value) = read_env("BOWSER_USER_DATA_DIR") {
        config.user_data_dir = Some(PathBuf::from(value));
    }
    if let Some(value) = read_env("BOWSER_HEADLESS") {
        config.headless = parse_bool("BOWSER_HEADLESS", &value)?;
    }
    if let Some(value) = read_env("BOWSER_PERSISTENT_PROFILE") {
        config.persistent_profile = parse_bool("BOWSER_PERSISTENT_PROFILE", &value)?;
    }
    if let Some(value) = read_env("BOWSER_VIEWPORT") {
        config.viewport = parse_viewport(&value)?;
    }
    if let Some(value) = read_env("BOWSER_TIMEOUT") {
        config.timeout = Duration::from_secs(parse_u64("BOWSER_TIMEOUT", &value)?);
    }
    if let Some(value) = read_env("BOWSER_STEALTH") {
        config.stealth = parse_bool("BOWSER_STEALTH", &value)?;
    }
    if let Some(value) = read_env("BOWSER_DISABLE_RUNTIME_EVENTS") {
        config.disable_runtime_events = parse_bool("BOWSER_DISABLE_RUNTIME_EVENTS", &value)?;
    }
    if let Some(value) = read_env("BOWSER_AI_ENABLED") {
        config.ai.enabled = parse_bool("BOWSER_AI_ENABLED", &value)?;
    }
    if let Some(value) = read_env("BOWSER_AI_PROVIDER") {
        config.ai.provider = parse_provider(&value)?;
    }
    if let Some(value) = read_env("BOWSER_AI_MODEL") {
        config.ai.model = value;
    }
    if let Some(value) = read_env("BOWSER_AI_API_KEY_ENV") {
        config.ai.api_key_env = value;
    }
    if let Some(value) = read_env("BOWSER_AI_ENDPOINT") {
        config.ai.endpoint = Some(value);
    }
    Ok(())
}

pub(super) fn apply_overrides(config: &mut BrowserConfig, overrides: ConfigOverrides) {
    if let Some(value) = overrides.chrome_path {
        config.chrome_path = Some(value);
    }
    if let Some(value) = overrides.chrome_args {
        config.chrome_args = value;
    }
    if let Some(value) = overrides.session_id {
        config.session.id = Some(value);
    }
    if let Some(value) = overrides.session_dir {
        config.session.dir = Some(value);
    }
    if let Some(value) = overrides.session_ttl_secs {
        config.session.idle_ttl = Duration::from_secs(value);
    }
    if let Some(value) = overrides.headless {
        config.headless = value;
    }
    if let Some(value) = overrides.persistent_profile {
        config.persistent_profile = value;
    }
    if let Some(value) = overrides.user_data_dir {
        config.user_data_dir = Some(value);
    }
    if let Some(value) = overrides.viewport {
        config.viewport = value;
    }
    if let Some(value) = overrides.timeout_secs {
        config.timeout = Duration::from_secs(value);
    }
    if let Some(value) = overrides.truncate {
        config.output.truncate = value;
    }
    if let Some(value) = overrides.include_hidden {
        config.output.include_hidden = value;
    }
    if let Some(value) = overrides.max_children {
        config.output.max_children = value;
    }
    if let Some(value) = overrides.max_list_items {
        config.output.max_list_items = value;
    }
    if let Some(value) = overrides.max_table_rows {
        config.output.max_table_rows = value;
    }
    if let Some(value) = overrides.stealth {
        config.stealth = value;
    }
    if let Some(value) = overrides.disable_runtime_events {
        config.disable_runtime_events = value;
    }
    if let Some(value) = overrides.ai_enabled {
        config.ai.enabled = value;
    }
    if let Some(value) = overrides.ai_provider {
        config.ai.provider = value;
    }
    if let Some(value) = overrides.ai_model {
        config.ai.model = value;
    }
    if let Some(value) = overrides.ai_api_key_env {
        config.ai.api_key_env = value;
    }
    if let Some(value) = overrides.ai_endpoint {
        config.ai.endpoint = Some(value);
    }
}

fn read_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}
