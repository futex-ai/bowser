//! Session-facing model types.

use std::path::PathBuf;

/// Metadata about the attached Bowser session.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionInfo {
    /// Opaque session identifier.
    pub id: String,
    /// Whether the current browser handle resumed an existing detached session.
    pub resumed: bool,
    /// Stable Bowser page ID for the selected page, when known.
    pub selected_page_id: Option<String>,
}

/// Session summary for list/info output.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionSummary {
    /// Opaque session identifier.
    pub id: String,
    /// Best available URL for summary display.
    pub url: Option<String>,
    /// Best available title for summary display.
    pub title: Option<String>,
    /// Last metadata update timestamp.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Browser process identifier.
    pub pid: u32,
    /// Browser profile directory backing the detached session.
    pub user_data_dir: PathBuf,
}

/// Stored type information for a Bowser session page.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionPageType {
    /// A top-level browser tab.
    #[default]
    Tab,
}

/// Session page summary for CLI and REPL page management.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionPageSummary {
    /// Stable Bowser page identifier.
    pub id: String,
    /// Stored page type, currently always `tab`.
    #[serde(rename = "type", default)]
    pub page_type: SessionPageType,
    /// Best available URL for the page.
    pub url: Option<String>,
    /// Best available title for the page.
    pub title: Option<String>,
    /// Whether this page is currently selected for commands.
    pub selected: bool,
    /// Whether Bowser could confirm the page still has a live Chrome target.
    pub live: bool,
}
