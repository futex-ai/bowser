//! Stored session metadata and legacy migration.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::model::{MetadataRecord, PageCapture, SessionPageType};

pub(crate) const LEGACY_PAGE_ID: &str = "pg_1";

/// Persisted state for a single Bowser page within a detached session.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionPageMetadata {
    /// Stable Bowser page identifier.
    pub id: String,
    /// Stored page type, currently always `tab`.
    #[serde(rename = "type", default)]
    pub page_type: SessionPageType,
    /// Live Chrome target identifier for the page.
    pub target_id: String,
    /// Last stored preview capture for page-management fallback output.
    pub preview_capture: Option<PageCapture>,
    /// Last full capture stored for expand operations and lazy resume.
    pub full_capture: Option<PageCapture>,
    /// Metadata captured alongside the last stored capture.
    pub metadata_records: Vec<MetadataRecord>,
    /// Whether the next runtime lookup must rebuild the live page capture.
    pub requires_fresh_capture: bool,
    /// Last known live URL for the page.
    pub last_url: Option<String>,
    /// Last known live title for the page.
    pub last_title: Option<String>,
}

impl SessionPageMetadata {
    /// Creates a new detached page record.
    pub fn new(id: String, target_id: String) -> Self {
        Self {
            id,
            page_type: SessionPageType::Tab,
            target_id,
            preview_capture: None,
            full_capture: None,
            metadata_records: Vec::new(),
            requires_fresh_capture: false,
            last_url: None,
            last_title: None,
        }
    }
}

/// Persisted session metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionMetadata {
    /// Opaque Bowser session identifier.
    pub id: String,
    /// Session creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Session last-update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Chrome remote-debugging HTTP endpoint.
    pub http_url: String,
    /// Chrome DevTools websocket endpoint.
    pub websocket_url: String,
    /// Browser process identifier.
    pub pid: u32,
    /// Optional Xvfb process identifier owned by this session.
    #[serde(default)]
    pub xvfb_pid: Option<u32>,
    /// Optional Xvfb display string used by headed Chrome.
    #[serde(default)]
    pub xvfb_display: Option<String>,
    /// Browser profile directory backing this session.
    pub user_data_dir: PathBuf,
    /// Whether `user_data_dir` is an ephemeral directory generated and owned by Bowser.
    #[serde(default)]
    pub owns_user_data_dir: bool,
    /// Stable Bowser page ID currently selected for commands.
    pub selected_page_id: Option<String>,
    /// Next ordinal used when allocating a stable Bowser page ID.
    pub next_page_ordinal: u32,
    /// Persisted page inventory for the detached session.
    pub pages: Vec<SessionPageMetadata>,
}

impl SessionMetadata {
    /// Creates a new session metadata document.
    pub fn new(http_url: String, websocket_url: String, pid: u32, user_data_dir: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: format!("bsr_{}", Uuid::new_v4().simple()),
            created_at: now,
            updated_at: now,
            http_url,
            websocket_url,
            pid,
            xvfb_pid: None,
            xvfb_display: None,
            user_data_dir,
            owns_user_data_dir: false,
            selected_page_id: None,
            next_page_ordinal: 1,
            pages: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub(super) struct RawSessionMetadata {
    id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    http_url: String,
    websocket_url: String,
    pid: u32,
    #[serde(default)]
    xvfb_pid: Option<u32>,
    #[serde(default)]
    xvfb_display: Option<String>,
    user_data_dir: PathBuf,
    #[serde(default)]
    owns_user_data_dir: bool,
    #[serde(default)]
    selected_page_id: Option<String>,
    #[serde(default)]
    next_page_ordinal: Option<u32>,
    #[serde(default)]
    pages: Vec<SessionPageMetadata>,
    #[serde(default)]
    preview_capture: Option<PageCapture>,
    #[serde(default)]
    full_capture: Option<PageCapture>,
    #[serde(default)]
    metadata_records: Vec<MetadataRecord>,
    #[serde(default)]
    requires_fresh_capture: bool,
}

impl From<RawSessionMetadata> for SessionMetadata {
    fn from(raw: RawSessionMetadata) -> Self {
        let mut pages = raw.pages;
        let mut next_page_ordinal = raw.next_page_ordinal.unwrap_or(1).max(1);
        if pages.is_empty()
            && (raw.preview_capture.is_some()
                || raw.full_capture.is_some()
                || !raw.metadata_records.is_empty()
                || raw.requires_fresh_capture)
        {
            let mut legacy_page =
                SessionPageMetadata::new(LEGACY_PAGE_ID.to_string(), "legacy".to_string());
            legacy_page.preview_capture = raw.preview_capture;
            legacy_page.full_capture = raw.full_capture;
            legacy_page.metadata_records = raw.metadata_records;
            legacy_page.requires_fresh_capture = raw.requires_fresh_capture;
            legacy_page.last_url = legacy_page.url();
            legacy_page.last_title = legacy_page.title();
            pages.push(legacy_page);
            next_page_ordinal = next_page_ordinal.max(2);
        }
        next_page_ordinal = next_page_ordinal.max(
            pages
                .iter()
                .filter_map(|page| {
                    page.id
                        .strip_prefix("pg_")
                        .and_then(|suffix| suffix.parse::<u32>().ok())
                })
                .max()
                .unwrap_or(0)
                + 1,
        );
        let selected_page_id = raw.selected_page_id.or_else(|| {
            pages.first().map(|page| page.id.clone()).or_else(|| {
                if pages.is_empty() {
                    None
                } else {
                    Some(LEGACY_PAGE_ID.to_string())
                }
            })
        });
        Self {
            id: raw.id,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
            http_url: raw.http_url,
            websocket_url: raw.websocket_url,
            pid: raw.pid,
            xvfb_pid: raw.xvfb_pid,
            xvfb_display: raw.xvfb_display,
            user_data_dir: raw.user_data_dir,
            owns_user_data_dir: raw.owns_user_data_dir,
            selected_page_id,
            next_page_ordinal,
            pages,
        }
    }
}
