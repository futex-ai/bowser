//! `PageEngine` implementation for `LivePage`.

use std::time::Duration;

use async_trait::async_trait;

use crate::error::Result;
use std::path::Path;

use crate::model::{
    DownloadResult, Element, ImageDescription, MetadataRecord, PageCapture, ScrollTarget,
};

use super::types::{LivePage, PageEngine};

#[async_trait]
impl PageEngine for LivePage {
    async fn navigate(&self, url: &str) -> Result<()> {
        self.navigate_impl(url).await
    }

    async fn back(&self) -> Result<()> {
        self.back_impl().await
    }

    async fn forward(&self) -> Result<()> {
        self.forward_impl().await
    }

    async fn reload(&self) -> Result<()> {
        self.reload_impl().await
    }

    async fn url(&self) -> Result<String> {
        self.url_impl().await
    }

    async fn title(&self) -> Result<String> {
        self.title_impl().await
    }

    async fn rendered_html(&self) -> Result<String> {
        self.rendered_html_impl().await
    }

    async fn capture(&self) -> Result<PageCapture> {
        self.capture_impl().await
    }

    async fn capture_subtree(&self, element_id: u32) -> Result<Element> {
        self.capture_subtree_impl(element_id).await
    }

    async fn expand(&self, element_id: u32) -> Result<Element> {
        self.expand_impl(element_id).await
    }

    async fn metadata(&self, element_id: u32) -> Result<MetadataRecord> {
        self.metadata_impl(element_id).await
    }

    async fn describe(&self, element_id: u32) -> Result<ImageDescription> {
        self.describe_impl(element_id).await
    }

    async fn click(&self, element_id: u32) -> Result<()> {
        self.click_impl(element_id).await
    }

    async fn press_keys(&self, keys: &[String]) -> Result<()> {
        self.press_keys_impl(keys).await
    }

    async fn type_text(&self, element_id: u32, text: &str) -> Result<()> {
        self.type_text_impl(element_id, text).await
    }

    async fn clear(&self, element_id: u32) -> Result<()> {
        self.clear_impl(element_id).await
    }

    async fn select_option(&self, element_id: u32, value: &str) -> Result<()> {
        self.select_option_impl(element_id, value).await
    }

    async fn submit(&self, element_id: u32) -> Result<()> {
        self.submit_impl(element_id).await
    }

    async fn scroll(&self, target: ScrollTarget) -> Result<()> {
        self.scroll_impl(target).await
    }

    async fn screenshot(&self) -> Result<Vec<u8>> {
        self.screenshot_impl().await
    }

    async fn screenshot_element(&self, element_id: u32) -> Result<Vec<u8>> {
        self.screenshot_element_impl(element_id).await
    }

    async fn download(&self, url: &str, destination: &Path) -> Result<DownloadResult> {
        self.download_impl(url, destination).await
    }

    async fn wait_for_selector(&self, selector: &str, timeout: Duration) -> Result<()> {
        self.wait_for_selector_impl(selector, timeout).await
    }

    async fn wait_for_stable(&self, timeout: Duration) -> Result<()> {
        self.wait_for_stable_impl(timeout).await
    }

    async fn evaluate_js(&self, expression: &str) -> Result<String> {
        self.evaluate_js_impl(expression).await
    }
}
