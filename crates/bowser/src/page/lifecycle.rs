//! Live-page lifecycle and capture-state helpers.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Page as ChromiumPage;

use crate::ai::ImageSummarizer;
use crate::capture::{
    ImageKey, known_image_descriptions, restore_image_descriptions, set_images_describable,
    truncate_capture,
};
use crate::config::BrowserConfig;
use crate::error::{Error, Result};
use crate::model::{PageCapture, collect_metadata_records};
use crate::session::{SessionPageMetadata, SessionStore};
use crate::stealth_features::StealthFeatures;

use super::types::{LivePage, LivePageSession, PageState};

const LIVE_DOCUMENT_STATE_TIMEOUT: Duration = Duration::from_secs(1);

impl LivePage {
    /// Creates a new live page wrapper.
    pub fn new(
        page: ChromiumPage,
        config: BrowserConfig,
        session: LivePageSession,
        store: Arc<dyn SessionStore>,
        metadata: SessionPageMetadata,
        summarizer: Option<Arc<dyn ImageSummarizer>>,
    ) -> Self {
        Self {
            page,
            config,
            session_id: session.session_id,
            page_id: session.page_id,
            resumed: session.resumed,
            store,
            summarizer,
            state: tokio::sync::Mutex::new(PageState {
                preview_capture: metadata.preview_capture,
                full_capture: metadata.full_capture,
                metadata_records: metadata.metadata_records,
                requires_fresh_capture: metadata.requires_fresh_capture,
                capture_in_progress: false,
                prepared: false,
                cursor_position: None,
                live_element_contexts: HashMap::new(),
                backend_node_ids: HashMap::new(),
            }),
        }
    }

    pub(super) async fn persist_state(&self, requires_fresh_capture: bool) -> Result<()> {
        let state = self.state.lock().await;
        let mut metadata = self.store.load(&self.session_id).await?;
        let page =
            metadata
                .page_by_id_mut(&self.page_id)
                .ok_or_else(|| Error::SessionPageNotFound {
                    page_id: self.page_id.clone(),
                })?;
        page.preview_capture = state.preview_capture.clone();
        page.full_capture = state.full_capture.clone();
        page.metadata_records = state.metadata_records.clone();
        page.requires_fresh_capture = requires_fresh_capture;
        page.last_url = page.url();
        page.last_title = page.title();
        metadata.updated_at = chrono::Utc::now();
        self.store.save(&metadata).await
    }

    pub(super) async fn clear_cached_document_state(&self, timeout: Duration) -> Result<()> {
        let timeout = timeout.min(LIVE_DOCUMENT_STATE_TIMEOUT);
        let (live_url, live_title) = tokio::join!(
            self.live_url_within(timeout),
            self.live_title_within(timeout)
        );
        self.write_stale_document_state(live_url, live_title).await
    }

    pub(super) async fn mark_cached_document_state_stale(&self) -> Result<()> {
        self.write_stale_document_state(None, None).await
    }

    async fn write_stale_document_state(
        &self,
        live_url: Option<String>,
        live_title: Option<String>,
    ) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            state.preview_capture = None;
            state.full_capture = None;
            state.metadata_records.clear();
            state.requires_fresh_capture = true;
            state.live_element_contexts.clear();
            state.backend_node_ids.clear();
        }
        let mut metadata = self.store.load(&self.session_id).await?;
        let page =
            metadata
                .page_by_id_mut(&self.page_id)
                .ok_or_else(|| Error::SessionPageNotFound {
                    page_id: self.page_id.clone(),
                })?;
        page.preview_capture = None;
        page.full_capture = None;
        page.metadata_records.clear();
        page.requires_fresh_capture = true;
        if live_url.is_some() {
            page.last_url = live_url;
        }
        if live_title.is_some() {
            page.last_title = live_title;
        }
        metadata.updated_at = chrono::Utc::now();
        self.store.save(&metadata).await
    }

    async fn live_url_within(&self, timeout: Duration) -> Option<String> {
        if timeout.is_zero() {
            return None;
        }
        tokio::time::timeout(timeout, self.page.url())
            .await
            .ok()?
            .ok()
            .flatten()
    }

    async fn live_title_within(&self, timeout: Duration) -> Option<String> {
        if timeout.is_zero() {
            return None;
        }
        tokio::time::timeout(timeout, self.page.get_title())
            .await
            .ok()?
            .ok()
            .flatten()
    }

    pub(super) async fn capture_snapshot(&self, wait_for_stable: bool) -> Result<PageCapture> {
        self.prepare().await?;
        {
            let mut state = self.state.lock().await;
            state.capture_in_progress = true;
        }
        let capture_result = async {
            let features = StealthFeatures::from_config(&self.config);
            let cached_descriptions = self.cached_image_descriptions().await;
            if wait_for_stable && !features.skip_runtime_stability() {
                let _ = self.wait_for_stable_impl(Duration::from_secs(5)).await;
            }
            let mut full = if features.accessibility_capture() {
                self.capture_accessibility_snapshot().await?
            } else {
                self.capture_frame_tree().await?
            };
            restore_image_descriptions(&mut full, &mut [], &cached_descriptions);
            set_images_describable(&mut full, &mut [], self.summarizer.is_some());
            let metadata = collect_metadata_records(&full);
            let preview = truncate_capture(&full, &self.config.output);
            let mut state = self.state.lock().await;
            state.preview_capture = Some(preview.clone());
            state.full_capture = Some(full);
            state.metadata_records = metadata;
            state.requires_fresh_capture = false;
            drop(state);
            self.persist_state(false).await?;
            Ok(preview)
        }
        .await;
        self.state.lock().await.capture_in_progress = false;
        capture_result
    }

    pub(super) async fn ensure_preview_capture(&self) -> Result<PageCapture> {
        let capture = {
            let state = self.state.lock().await;
            if state.requires_fresh_capture || state.preview_capture.is_none() {
                None
            } else {
                state.preview_capture.clone()
            }
        };
        if let Some(capture) = capture {
            Ok(capture)
        } else {
            self.capture_impl().await
        }
    }

    pub(super) async fn ensure_live_ids(&self, element_id: u32) -> Result<()> {
        self.prepare().await?;
        let needs_capture = self.needs_live_id_refresh(element_id).await;
        if needs_capture {
            let _ = self.capture_snapshot(false).await?;
        }
        Ok(())
    }

    async fn needs_live_id_refresh(&self, element_id: u32) -> bool {
        {
            let state = self.state.lock().await;
            if should_refresh_live_ids(state.requires_fresh_capture, state.capture_in_progress) {
                return true;
            }
            if state.live_element_contexts.contains_key(&element_id) {
                return false;
            }
            if state.backend_node_ids.contains_key(&element_id) {
                return false;
            }
        }
        self.evaluate_bowser_value::<bool>(format!(
            r#"
(() => {{
  const map = window.__bowserElements;
  if (!(map instanceof Map)) return false;
  const el = map.get({element_id});
  return !!el && el.isConnected;
}})()
"#
        ))
        .await
        .map(|available| !available)
        .unwrap_or(true)
    }

    async fn cached_image_descriptions(&self) -> std::collections::HashMap<ImageKey, String> {
        let state = self.state.lock().await;
        state
            .full_capture
            .as_ref()
            .or(state.preview_capture.as_ref())
            .map(known_image_descriptions)
            .unwrap_or_default()
    }
}

pub(crate) fn should_refresh_live_ids(
    requires_fresh_capture: bool,
    capture_in_progress: bool,
) -> bool {
    requires_fresh_capture && !capture_in_progress
}
