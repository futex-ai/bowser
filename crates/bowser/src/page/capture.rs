//! Capture and screenshot helpers.

use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, Viewport as ClipViewport,
};
use chromiumoxide::page::ScreenshotParams;

use crate::error::{Error, Result};
use crate::expand::{expand_element, find_capture_element};
use crate::model::{Element, PageCapture};

use super::types::{LiveElementContext, LivePage};

impl LivePage {
    pub(super) async fn capture_impl(&self) -> Result<PageCapture> {
        self.capture_snapshot(true).await
    }

    pub(super) async fn capture_subtree_impl(&self, element_id: u32) -> Result<Element> {
        let capture = self.ensure_preview_capture().await?;
        find_capture_element(&capture, element_id).ok_or(Error::ElementNotFound { element_id })
    }

    pub(super) async fn expand_impl(&self, element_id: u32) -> Result<Element> {
        let capture = {
            let state = self.state.lock().await;
            state
                .full_capture
                .clone()
                .or_else(|| state.preview_capture.clone())
        }
        .ok_or(Error::ExpandNotFound { element_id })?;
        let element =
            expand_element(&capture, element_id).ok_or(Error::ExpandNotFound { element_id })?;
        if element.is_expandable()
            || matches!(
                element,
                Element::Table { .. }
                    | Element::List { .. }
                    | Element::Nav { .. }
                    | Element::Form { .. }
                    | Element::Section { .. }
                    | Element::Iframe { .. }
            )
        {
            Ok(element)
        } else {
            Err(Error::ExpandNotFound { element_id })
        }
    }

    pub(super) async fn screenshot_impl(&self) -> Result<Vec<u8>> {
        self.prepare().await?;
        let metrics = self
            .page
            .layout_metrics()
            .await
            .map_err(|err| Error::Screenshot {
                reason: err.to_string(),
            })?;
        self.page
            .screenshot(
                ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .clip(ClipViewport {
                        x: 0.0,
                        y: 0.0,
                        width: metrics.css_content_size.width,
                        height: metrics.css_content_size.height,
                        scale: 1.0,
                    })
                    .from_surface(true)
                    .capture_beyond_viewport(true)
                    .build(),
            )
            .await
            .map_err(|err| Error::Screenshot {
                reason: err.to_string(),
            })
    }

    pub(super) async fn screenshot_element_impl(&self, element_id: u32) -> Result<Vec<u8>> {
        self.ensure_live_ids(element_id).await?;
        self.screenshot_element_from_live_ids(element_id).await
    }

    pub(super) async fn screenshot_element_from_live_ids(
        &self,
        element_id: u32,
    ) -> Result<Vec<u8>> {
        if let Some(context @ LiveElementContext::Remote { .. }) =
            self.live_element_context(element_id).await
        {
            return self.screenshot_frame_element(context, element_id).await;
        }
        let bounds = self.element_bounds(element_id).await?;
        self.screenshot_clip(bounds).await
    }

    pub(super) async fn screenshot_clip(&self, bounds: super::types::Bounds) -> Result<Vec<u8>> {
        self.page
            .screenshot(
                ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .clip(ClipViewport {
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: bounds.height,
                        scale: 1.0,
                    })
                    .capture_beyond_viewport(true)
                    .build(),
            )
            .await
            .map_err(|err| Error::Screenshot {
                reason: err.to_string(),
            })
    }
}
