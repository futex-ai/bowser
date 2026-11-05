//! Ephemeral favicon-enriched live page inventory.

use std::sync::Arc;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::favicon::acquisition::{DefaultFaviconAcquirer, FaviconAcquirer, FaviconResourceReader};
use crate::favicon::chromium::ChromiumFaviconReader;
use crate::favicon::scheduler::{DeadlineFaviconScheduler, FaviconFuture, FaviconScheduler};
use crate::favicon::transport::loader::{BrowserResourceLoader, ChromiumResourceLoader};
use crate::model::LiveSessionPage;

use super::engine::Browser;

const INVENTORY_BUDGET: Duration = Duration::from_secs(1);
const INVENTORY_CONCURRENCY: usize = 4;

impl Browser {
    pub(super) async fn live_session_pages_impl(&self) -> Result<Vec<LiveSessionPage>> {
        let mut state = self.inner.lock().await;
        let live_pages = self.sync_live_pages(&mut state).await?;
        if live_pages.is_empty() {
            return Err(Error::BrowserDisconnected);
        }
        let selected_page_id = state.metadata.selected_page_id.clone();
        let deadline = tokio::time::Instant::now() + INVENTORY_BUDGET;
        let acquirer: Arc<dyn FaviconAcquirer> = Arc::new(DefaultFaviconAcquirer);
        let scheduler: &dyn FaviconScheduler = &DeadlineFaviconScheduler;
        let mut pages = Vec::new();
        let mut favicon_futures = Vec::new();
        for live_page in live_pages {
            let Some(metadata) = state.metadata.page_by_target_id(&live_page.target_id) else {
                continue;
            };
            let expected_url = live_page.url;
            pages.push(LiveSessionPage {
                page_id: metadata.id.clone(),
                url: expected_url.clone(),
                title: (!live_page.title.is_empty()).then_some(live_page.title),
                selected: selected_page_id.as_deref() == Some(metadata.id.as_str()),
                favicon: None,
            });
            let loader: Arc<dyn BrowserResourceLoader> = Arc::new(ChromiumResourceLoader::new(
                live_page.page.clone(),
                deadline,
            ));
            let reader: Arc<dyn FaviconResourceReader> = Arc::new(ChromiumFaviconReader::new(
                live_page.page,
                expected_url,
                deadline,
                loader,
            ));
            let acquirer = Arc::clone(&acquirer);
            let future: FaviconFuture =
                Box::pin(async move { acquirer.acquire(reader, deadline).await });
            favicon_futures.push(future);
        }
        let favicons = scheduler
            .collect(favicon_futures, deadline, INVENTORY_CONCURRENCY)
            .await;
        for (page, favicon) in pages.iter_mut().zip(favicons) {
            page.favicon = favicon;
        }
        Ok(pages)
    }
}
