//! Public Bowser configuration data types.

use std::path::PathBuf;
use std::time::Duration;

use crate::model::Viewport;

/// Detached-session configuration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionConfig {
    pub id: Option<String>,
    pub dir: Option<PathBuf>,
    #[serde(with = "duration_seconds")]
    pub idle_ttl: Duration,
}

/// Output preview configuration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OutputConfig {
    pub truncate: bool,
    pub include_hidden: bool,
    pub max_children: usize,
    pub max_list_items: usize,
    pub max_table_rows: usize,
}

/// AI provider.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiProvider {
    Anthropic,
    OpenAi,
    Ollama,
}

/// AI configuration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AiConfig {
    pub enabled: bool,
    pub provider: AiProvider,
    pub model: String,
    pub api_key_env: String,
    pub endpoint: Option<String>,
}

/// Main browser configuration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BrowserConfig {
    pub chrome_path: Option<PathBuf>,
    pub chrome_args: Vec<String>,
    pub session: SessionConfig,
    pub output: OutputConfig,
    pub headless: bool,
    pub persistent_profile: bool,
    pub user_data_dir: Option<PathBuf>,
    pub viewport: Viewport,
    #[serde(with = "duration_seconds")]
    pub timeout: Duration,
    pub stealth: bool,
    pub disable_runtime_events: bool,
    pub ai: AiConfig,
}

/// CLI override bag.
#[derive(Clone, Debug, Default)]
pub struct ConfigOverrides {
    pub chrome_path: Option<PathBuf>,
    pub chrome_args: Option<Vec<String>>,
    pub session_id: Option<String>,
    pub session_dir: Option<PathBuf>,
    pub session_ttl_secs: Option<u64>,
    pub headless: Option<bool>,
    pub persistent_profile: Option<bool>,
    pub user_data_dir: Option<PathBuf>,
    pub viewport: Option<Viewport>,
    pub timeout_secs: Option<u64>,
    pub truncate: Option<bool>,
    pub include_hidden: Option<bool>,
    pub max_children: Option<usize>,
    pub max_list_items: Option<usize>,
    pub max_table_rows: Option<usize>,
    pub stealth: Option<bool>,
    pub disable_runtime_events: Option<bool>,
    pub ai_enabled: Option<bool>,
    pub ai_provider: Option<AiProvider>,
    pub ai_model: Option<String>,
    pub ai_api_key_env: Option<String>,
    pub ai_endpoint: Option<String>,
}

mod duration_seconds {
    //! Duration serde helpers for config files.

    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Duration::from_secs(u64::deserialize(deserializer)?))
    }
}
