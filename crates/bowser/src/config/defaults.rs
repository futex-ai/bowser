//! Default Bowser configuration values.

use std::path::PathBuf;
use std::time::Duration;

use crate::config::types::{AiConfig, AiProvider, BrowserConfig, OutputConfig, SessionConfig};
use crate::model::Viewport;

/// Returns the default config path.
pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bowser")
        .join("config.yaml")
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            chrome_path: None,
            chrome_args: Vec::new(),
            session: SessionConfig {
                id: None,
                dir: None,
                idle_ttl: Duration::from_secs(1800),
            },
            output: OutputConfig {
                truncate: true,
                include_hidden: false,
                max_children: 50,
                max_list_items: 20,
                max_table_rows: 20,
            },
            headless: true,
            persistent_profile: false,
            user_data_dir: None,
            viewport: Viewport::default(),
            timeout: Duration::from_secs(30),
            stealth: true,
            disable_runtime_events: false,
            ai: AiConfig {
                enabled: true,
                provider: AiProvider::Anthropic,
                model: "claude-sonnet-4-20250514".to_string(),
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
                endpoint: None,
            },
        }
    }
}
