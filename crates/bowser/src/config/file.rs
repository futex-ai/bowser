//! Config file decoding and merge ordering.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use super::defaults::default_config_path;
use super::overrides::{apply_env, apply_overrides};
use super::types::{AiProvider, BrowserConfig, ConfigOverrides};
use super::validation::validate_config;
use crate::error::{Error, Result};
use crate::model::Viewport;

#[derive(Clone, Debug, Default, Deserialize)]
struct FileConfig {
    sessions: Option<FileSessionConfig>,
    output: Option<FileOutputConfig>,
    chrome: Option<FileChromeConfig>,
    viewport: Option<Viewport>,
    timeout: Option<u64>,
    stealth: Option<bool>,
    disable_runtime_events: Option<bool>,
    ai: Option<FileAiConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FileSessionConfig {
    dir: Option<PathBuf>,
    idle_ttl: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FileOutputConfig {
    truncate: Option<bool>,
    include_hidden: Option<bool>,
    max_children: Option<usize>,
    max_list_items: Option<usize>,
    max_table_rows: Option<usize>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FileChromeConfig {
    path: Option<PathBuf>,
    args: Option<Vec<String>>,
    headless: Option<bool>,
    persistent_profile: Option<bool>,
    user_data_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FileAiConfig {
    enabled: Option<bool>,
    provider: Option<AiProvider>,
    model: Option<String>,
    api_key_env: Option<String>,
    endpoint: Option<String>,
}

/// Loads config file, environment, and CLI overrides.
pub fn load_config(
    config_path: Option<&Path>,
    overrides: ConfigOverrides,
) -> Result<BrowserConfig> {
    let path = config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_config_path);
    let mut config = BrowserConfig::default();
    if path.exists() {
        let contents =
            std::fs::read_to_string(&path).map_err(|err| Error::io("read config", err))?;
        let file_config: FileConfig = serde_yaml::from_str(&contents).map_err(|err| {
            Error::config(format!(
                "failed to parse config file {}: {err}",
                path.display()
            ))
        })?;
        apply_file_config(&mut config, file_config);
    }
    apply_env(&mut config)?;
    apply_overrides(&mut config, overrides);
    validate_config(&config)?;
    Ok(config)
}

fn apply_file_config(config: &mut BrowserConfig, file: FileConfig) {
    if let Some(sessions) = file.sessions {
        config.session.dir = sessions.dir.or(config.session.dir.clone());
        if let Some(ttl) = sessions.idle_ttl {
            config.session.idle_ttl = Duration::from_secs(ttl);
        }
    }
    if let Some(output) = file.output {
        if let Some(value) = output.truncate {
            config.output.truncate = value;
        }
        if let Some(value) = output.include_hidden {
            config.output.include_hidden = value;
        }
        if let Some(value) = output.max_children {
            config.output.max_children = value;
        }
        if let Some(value) = output.max_list_items {
            config.output.max_list_items = value;
        }
        if let Some(value) = output.max_table_rows {
            config.output.max_table_rows = value;
        }
    }
    if let Some(chrome) = file.chrome {
        config.chrome_path = chrome.path.or(config.chrome_path.clone());
        if let Some(args) = chrome.args {
            config.chrome_args = args;
        }
        if let Some(headless) = chrome.headless {
            config.headless = headless;
        }
        if let Some(persistent_profile) = chrome.persistent_profile {
            config.persistent_profile = persistent_profile;
        }
        config.user_data_dir = chrome.user_data_dir.or(config.user_data_dir.clone());
    }
    if let Some(viewport) = file.viewport {
        config.viewport = viewport;
    }
    if let Some(timeout) = file.timeout {
        config.timeout = Duration::from_secs(timeout);
    }
    if let Some(stealth) = file.stealth {
        config.stealth = stealth;
    }
    if let Some(disable_runtime_events) = file.disable_runtime_events {
        config.disable_runtime_events = disable_runtime_events;
    }
    if let Some(ai) = file.ai {
        if let Some(enabled) = ai.enabled {
            config.ai.enabled = enabled;
        }
        if let Some(provider) = ai.provider {
            config.ai.provider = provider;
        }
        if let Some(model) = ai.model {
            config.ai.model = model;
        }
        if let Some(api_key_env) = ai.api_key_env {
            config.ai.api_key_env = api_key_env;
        }
        config.ai.endpoint = ai.endpoint.or(config.ai.endpoint.clone());
    }
}
