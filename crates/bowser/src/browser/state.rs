//! Browser page inventory state and metadata synchronization.

use std::{collections::HashSet, time::Duration};

use chromiumoxide::browser::Browser as ChromiumBrowser;
use futures::future::join_all;

use crate::{
    error::{Error, Result},
    session::SessionMetadata,
};

use super::{
    engine::Browser,
    page_selection::best_live_page_id,
    page_state::{live_page_focused, live_page_title, live_page_url},
};

const PAGE_DISCOVERY_ATTEMPTS: usize = 20;
const PAGE_DISCOVERY_RETRY_DELAY: Duration = Duration::from_millis(100);

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
    pub(super) focused: bool,
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
        let target_infos = state
            .browser
            .fetch_targets()
            .await
            .map_err(|err| Error::cdp(format!("failed to fetch targets: {err}")))?;
        let fetched_page_target_ids = target_infos
            .into_iter()
            .filter(|target| target.r#type == "page")
            .map(|target| target.target_id.as_ref().to_string())
            .collect::<Vec<_>>();
        let fetched_page_targets = fetched_page_target_ids
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let mut attempt = 0;
        let pages = loop {
            let pages = state
                .browser
                .pages()
                .await
                .map_err(|err| Error::cdp(format!("failed to list pages: {err}")))?;
            let ready_target_ids = pages
                .iter()
                .map(|page| page.target_id().as_ref().to_string())
                .collect::<HashSet<_>>();
            if fetched_page_targets.is_subset(&ready_target_ids) {
                break pages;
            }
            attempt += 1;
            if attempt == PAGE_DISCOVERY_ATTEMPTS {
                break pages;
            }
            tokio::time::sleep(PAGE_DISCOVERY_RETRY_DELAY).await;
        };
        let pages = pages
            .into_iter()
            .filter(|page| fetched_page_targets.contains(page.target_id().as_ref()))
            .collect();
        let mut live_pages = collect_live_pages(state, pages).await;
        self.add_fetched_live_pages(state, &fetched_page_target_ids, &mut live_pages)
            .await?;
        Ok(live_pages)
    }

    async fn add_fetched_live_pages(
        &self,
        state: &mut BrowserState,
        fetched_page_target_ids: &[String],
        live_pages: &mut Vec<LiveBrowserPage>,
    ) -> Result<()> {
        let mut live_target_ids: HashSet<String> = live_pages
            .iter()
            .map(|page| page.target_id.clone())
            .collect();
        for target_id in fetched_page_target_ids {
            if live_target_ids.contains(target_id) {
                continue;
            }
            if let Ok(page) = state.browser.get_page(target_id.clone().into()).await {
                live_target_ids.insert(target_id.clone());
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
        let selected_is_live =
            state
                .metadata
                .selected_page_id
                .as_deref()
                .is_some_and(|selected_page_id| {
                    live_pages.iter().any(|page| {
                        state
                            .metadata
                            .page_by_target_id(&page.target_id)
                            .is_some_and(|record| record.id == selected_page_id)
                    })
                });
        let focused_page_ids = live_pages
            .iter()
            .filter(|page| page.focused)
            .filter_map(|page| {
                state
                    .metadata
                    .page_by_target_id(&page.target_id)
                    .map(|record| record.id.clone())
            })
            .collect::<Vec<_>>();
        let selected_page_id = if let [focused_page_id] = focused_page_ids.as_slice() {
            Some(focused_page_id.clone())
        } else if !selected_is_live {
            best_live_page_id(&state.metadata, &live_pages)
        } else {
            None
        };
        if let Some(page_id) = selected_page_id
            && state.metadata.selected_page_id.as_ref() != Some(&page_id)
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
    join_all(
        pages
            .into_iter()
            .map(|page| build_live_browser_page(&state.metadata, page)),
    )
    .await
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
    let (url, title, focused) = tokio::join!(
        live_page_url(&page, fallback_url),
        live_page_title(&page, fallback_title),
        live_page_focused(&page)
    );
    LiveBrowserPage {
        target_id,
        page,
        url,
        title,
        focused,
    }
}
