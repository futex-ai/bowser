//! Page runtime types.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chromiumoxide::Page as ChromiumPage;
use chromiumoxide::cdp::browser_protocol::dom::BackendNodeId;
use chromiumoxide::cdp::browser_protocol::page::FrameId;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::ai::ImageSummarizer;
use crate::config::BrowserConfig;
use crate::error::Result;
use crate::model::{
    DownloadResult, Element, ElementBounds, ElementVisibility, ImageDescription, MetadataRecord,
    PageCapture, ScrollTarget,
};
use crate::mouse::CursorPosition;
use crate::session::SessionStore;

/// Page trait boundary.
#[async_trait]
pub trait PageEngine: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn back(&self) -> Result<()>;
    async fn forward(&self) -> Result<()>;
    async fn reload(&self) -> Result<()>;
    async fn url(&self) -> Result<String>;
    async fn title(&self) -> Result<String>;
    async fn rendered_html(&self) -> Result<String>;
    async fn capture(&self) -> Result<PageCapture>;
    async fn capture_subtree(&self, element_id: u32) -> Result<Element>;
    async fn expand(&self, element_id: u32) -> Result<Element>;
    async fn metadata(&self, element_id: u32) -> Result<MetadataRecord>;
    async fn describe(&self, element_id: u32) -> Result<ImageDescription>;
    async fn click(&self, element_id: u32) -> Result<()>;
    async fn press_keys(&self, keys: &[String]) -> Result<()>;
    async fn type_text(&self, element_id: u32, text: &str) -> Result<()>;
    async fn clear(&self, element_id: u32) -> Result<()>;
    async fn select_option(&self, element_id: u32, value: &str) -> Result<()>;
    async fn submit(&self, element_id: u32) -> Result<()>;
    async fn scroll(&self, target: ScrollTarget) -> Result<()>;
    async fn screenshot(&self) -> Result<Vec<u8>>;
    async fn screenshot_element(&self, element_id: u32) -> Result<Vec<u8>>;
    async fn download(&self, url: &str, destination: &Path) -> Result<DownloadResult>;
    async fn wait_for_selector(&self, selector: &str, timeout: Duration) -> Result<()>;
    async fn wait_for_stable(&self, timeout: Duration) -> Result<()>;
    async fn evaluate_js(&self, expression: &str) -> Result<String>;
}

/// Live page implementation.
pub struct LivePage {
    pub(super) page: ChromiumPage,
    pub(super) config: BrowserConfig,
    pub(super) session_id: String,
    pub(super) page_id: String,
    pub(super) resumed: bool,
    pub(super) store: Arc<dyn SessionStore>,
    pub(super) summarizer: Option<Arc<dyn ImageSummarizer>>,
    pub(super) state: Mutex<PageState>,
}

pub(crate) struct LivePageSession {
    pub(crate) session_id: String,
    pub(crate) page_id: String,
    pub(crate) resumed: bool,
}

#[derive(Debug)]
pub(super) struct PageState {
    pub(super) preview_capture: Option<PageCapture>,
    pub(super) full_capture: Option<PageCapture>,
    pub(super) metadata_records: Vec<MetadataRecord>,
    pub(super) requires_fresh_capture: bool,
    pub(super) capture_in_progress: bool,
    pub(super) prepared: bool,
    pub(super) cursor_position: Option<CursorPosition>,
    pub(super) live_element_contexts: HashMap<u32, LiveElementContext>,
    pub(super) backend_node_ids: HashMap<u32, BackendNodeId>,
}

#[derive(Clone, Debug)]
pub(super) enum LiveElementContext {
    /// Element ID owned by a target-backed iframe's isolated capture world.
    Remote {
        frame_id: FrameId,
        websocket_url: String,
        element_id: u32,
        owner_element_id: Option<u32>,
    },
}

#[derive(Debug, Deserialize)]
pub(super) struct ActionResponse {
    pub(super) ok: bool,
    pub(super) kind: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct Bounds {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct ElementRuntimeState {
    pub(super) present: bool,
    pub(super) in_viewport: bool,
    pub(super) obscured: bool,
    pub(super) enabled: bool,
    pub(super) visible: bool,
    pub(super) clickable: bool,
    pub(super) focused: bool,
    pub(super) bounds: Option<ElementBounds>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) struct InteractionTarget {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) viewport_width: f64,
    pub(super) viewport_height: f64,
    pub(super) visible: bool,
    pub(super) pointer_enabled: bool,
    pub(super) enabled: bool,
    #[serde(default)]
    pub(super) focus_after_click: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct ViewportMetrics {
    pub(super) width: f64,
    pub(super) height: f64,
}

#[derive(Debug, Deserialize)]
pub(super) struct SubmitPlan {
    pub(super) ok: bool,
    pub(super) kind: Option<String>,
    pub(super) strategy: Option<String>,
}

impl ElementRuntimeState {
    pub(super) fn visibility(self) -> ElementVisibility {
        ElementVisibility {
            present: self.present,
            in_viewport: self.in_viewport,
            obscured: self.obscured,
            enabled: self.enabled,
            visible: self.visible,
            clickable: self.clickable,
        }
    }
}
