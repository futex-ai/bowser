//! Browser page inventory state and metadata synchronization.

use std::collections::HashSet;

use chromiumoxide::browser::Browser as ChromiumBrowser;

use crate::{
    error::{Error, Result},
    session::SessionMetadata,
};

use super::{
    engine::Browser,
    page_state::{live_page_title, live_page_url},
};

#[derive(Debug)]
pub(super) struct BrowserState {
    pub(super) browser: ChromiumBrowser,
    pub(super) handler_task: tokio::task::JoinHandle<()>,
    pub(super) metadata: SessionMetadata,
}

#[derive(Clone)]
pub(super) struct LiveBrowserPage {
    pub(super) page: chromiumoxide::Page,
    pub(super) target_id: String,
    pub(super) url: String,
    pub(super) title: String,
}

impl Browser {
    pub(super) async fn refresh_metadata(&self, state: &mut BrowserState) -> Result<()> {
        state.metadata = self
            .store
            .load(&state.metadata.id)
            .await
            .unwrap_or_else(|_| state.metadata.clone());
        Ok(())
    }

    pub(super) async fn list_live_pages(
        &self,
        state: &mut BrowserState,
    ) -> Result<Vec<LiveBrowserPage>> {
        state
            .browser
            .fetch_targets()
            .await
            .map_err(|err| Error::cdp(format!("failed to fetch targets: {err}")))?;
        for attempt in 0..20 {
            let pages = state
                .browser
                .pages()
                .await
                .map_err(|err| Error::cdp(format!("failed to list pages: {err}")))?;
            if !pages.is_empty() || attempt == 19 {
                let mut live_pages = collect_live_pages(state, pages).await;
                self.add_known_live_pages(state, &mut live_pages).await?;
                return Ok(live_pages);
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let mut live_pages = Vec::new();
        let target_ids: Vec<String> = state
            .metadata
            .pages
            .iter()
            .map(|page| page.target_id.clone())
            .filter(|target_id| !target_id.is_empty() && target_id != "legacy")
            .collect();
        for target_id in target_ids {
            if let Some(page) = self.lookup_page_by_target_id(state, &target_id).await? {
                live_pages.push(build_live_browser_page(&state.metadata, page).await);
            }
        }
        Ok(live_pages)
    }

    async fn add_known_live_pages(
        &self,
        state: &mut BrowserState,
        live_pages: &mut Vec<LiveBrowserPage>,
    ) -> Result<()> {
        let mut live_target_ids: HashSet<String> = live_pages
            .iter()
            .map(|page| page.target_id.clone())
            .collect();
        let known_target_ids: Vec<String> = state
            .metadata
            .pages
            .iter()
            .map(|page| page.target_id.clone())
            .filter(|target_id| !target_id.is_empty() && target_id != "legacy")
            .collect();
        for target_id in known_target_ids {
            if live_target_ids.contains(&target_id) {
                continue;
            }
            if let Some(page) = self.lookup_page_by_target_id(state, &target_id).await? {
                live_target_ids.insert(target_id);
                live_pages.push(build_live_browser_page(&state.metadata, page).await);
            }
        }
        Ok(())
    }

    pub(super) async fn lookup_page_by_target_id(
        &self,
        state: &mut BrowserState,
        target_id: &str,
    ) -> Result<Option<chromiumoxide::Page>> {
        state
            .browser
            .fetch_targets()
            .await
            .map_err(|err| Error::cdp(format!("failed to fetch targets: {err}")))?;
        for attempt in 0..50 {
            if let Some(page) = page_from_current_pages(state, target_id).await? {
                return Ok(Some(page));
            }
            match state.browser.get_page(target_id.to_string().into()).await {
                Ok(page) => return Ok(Some(page)),
                Err(_) if attempt < 49 => {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                Err(_) => return Ok(None),
            }
        }
        Ok(None)
    }

    pub(super) async fn sync_live_pages(
        &self,
        state: &mut BrowserState,
    ) -> Result<Vec<LiveBrowserPage>> {
        self.refresh_metadata(state).await?;
        let live_pages = self.list_live_pages(state).await?;
        let mut changed = false;
        let mut matched_page_ids = HashSet::new();
        for live_page in &live_pages {
            let page = if let Some(index) = state
                .metadata
                .pages
                .iter()
                .position(|page| page.target_id == live_page.target_id)
            {
                &mut state.metadata.pages[index]
            } else if let Some(index) = state.metadata.pages.iter().position(|page| {
                !matched_page_ids.contains(&page.id)
                    && page.has_non_blank_url()
                    && page.url().as_deref() == Some(live_page.url.as_str())
            }) {
                state.metadata.pages[index].target_id = live_page.target_id.clone();
                changed = true;
                &mut state.metadata.pages[index]
            } else {
                changed = true;
                state.metadata.ensure_page_for_target(&live_page.target_id)
            };
            if page.last_url.as_deref() != Some(live_page.url.as_str())
                || page.last_title.as_deref() != Some(live_page.title.as_str())
            {
                page.update_live_state(live_page.url.clone(), live_page.title.clone());
                changed = true;
            }
            matched_page_ids.insert(page.id.clone());
        }
        if state.metadata.selected_page_id.is_none()
            && let Some(page_id) = best_live_page_id(&state.metadata, &live_pages)
        {
            state.metadata.selected_page_id = Some(page_id);
            changed = true;
        }
        if changed {
            state.metadata.updated_at = chrono::Utc::now();
            self.store.save(&state.metadata).await?;
        }
        Ok(live_pages)
    }
}

async fn page_from_current_pages(
    state: &mut BrowserState,
    target_id: &str,
) -> Result<Option<chromiumoxide::Page>> {
    let pages = state
        .browser
        .pages()
        .await
        .map_err(|err| Error::cdp(format!("failed to list pages: {err}")))?;
    Ok(pages
        .into_iter()
        .find(|page| page.target_id().as_ref() == target_id))
}

async fn collect_live_pages(
    state: &BrowserState,
    pages: Vec<chromiumoxide::Page>,
) -> Vec<LiveBrowserPage> {
    let mut live_pages = Vec::with_capacity(pages.len());
    for page in pages {
        live_pages.push(build_live_browser_page(&state.metadata, page).await);
    }
    live_pages
}

async fn build_live_browser_page(
    metadata: &SessionMetadata,
    page: chromiumoxide::Page,
) -> LiveBrowserPage {
    let target_id = page.target_id().as_ref().to_string();
    let fallback_url = metadata
        .page_by_target_id(&target_id)
        .and_then(|record| record.url());
    let fallback_title = metadata
        .page_by_target_id(&target_id)
        .and_then(|record| record.title());
    let (url, title) = tokio::join!(
        live_page_url(&page, fallback_url),
        live_page_title(&page, fallback_title)
    );
    LiveBrowserPage {
        target_id,
        page,
        url,
        title,
    }
}

pub(super) fn best_live_page_id(
    metadata: &SessionMetadata,
    live_pages: &[LiveBrowserPage],
) -> Option<String> {
    let live_page_id = |page: &LiveBrowserPage| {
        metadata
            .page_by_target_id(&page.target_id)
            .map(|record| record.id.clone())
    };
    if let Some(selected_page_id) = metadata.selected_page_id.as_deref()
        && live_pages.iter().any(|page| {
            metadata
                .page_by_target_id(&page.target_id)
                .is_some_and(|record| record.id == selected_page_id)
        })
    {
        return Some(selected_page_id.to_string());
    }
    live_pages
        .iter()
        .find(|page| !page.url.is_empty() && page.url != "about:blank")
        .and_then(live_page_id)
        .or_else(|| live_pages.first().and_then(live_page_id))
}

pub(super) fn take_live_page_by_id(
    metadata: &SessionMetadata,
    live_pages: &mut Vec<LiveBrowserPage>,
    page_id: &str,
) -> Option<LiveBrowserPage> {
    let index = live_pages.iter().position(|page| {
        metadata
            .page_by_target_id(&page.target_id)
            .is_some_and(|record| record.id == page_id)
    })?;
    Some(live_pages.remove(index))
}

pub(super) fn page_target_id(
    metadata: &SessionMetadata,
    page_id: &str,
    page: &chromiumoxide::Page,
) -> String {
    metadata
        .page_by_id(page_id)
        .map(|record| record.target_id.clone())
        .filter(|target_id| !target_id.is_empty() && target_id != "legacy")
        .unwrap_or_else(|| page.target_id().as_ref().to_string())
}
