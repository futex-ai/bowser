//! Page attach, selection, and close operations for the live browser.

use std::{collections::HashSet, sync::Arc};

use crate::{
    ai::build_image_summarizer,
    error::{Error, Result},
    page::{LivePage, LivePageSession, PageEngine},
    session::SessionMetadata,
};

use super::{
    engine::Browser,
    page_selection::{best_live_page_id, page_target_id, take_live_page_by_id},
};

impl Browser {
    pub(super) async fn current_page_impl(&self) -> Result<Box<dyn PageEngine>> {
        let mut state = self.inner.lock().await;
        let (page_id, page) = self.attach_best_page(&mut state).await?;
        let target_id = page_target_id(&state.metadata, &page_id, &page);
        let page = self
            .activate_or_reacquire_page(&mut state, page, target_id)
            .await?;
        self.build_live_page(page, page_id, &state.metadata)
    }

    pub(super) async fn list_pages_impl(&self) -> Result<Vec<crate::model::SessionPageSummary>> {
        let mut state = self.inner.lock().await;
        let live_pages = self.sync_live_pages(&mut state).await?;
        let live_page_ids: HashSet<String> = live_pages
            .iter()
            .filter_map(|page| {
                state
                    .metadata
                    .page_by_target_id(&page.target_id)
                    .map(|record| record.id.clone())
            })
            .collect();
        Ok(state.metadata.page_summaries(&live_page_ids))
    }

    pub(super) async fn live_session_metadata_impl(&self) -> Result<SessionMetadata> {
        let mut state = self.inner.lock().await;
        let live_pages = self.sync_live_pages(&mut state).await?;
        if live_pages.is_empty() {
            return Err(Error::BrowserDisconnected);
        }
        let live_target_ids: HashSet<String> = live_pages
            .iter()
            .map(|page| page.target_id.clone())
            .collect();
        let mut metadata = state.metadata.clone();
        metadata
            .pages
            .retain(|page| live_target_ids.contains(&page.target_id));
        Ok(metadata)
    }

    pub(super) async fn select_page_impl(&self, page_id: &str) -> Result<Box<dyn PageEngine>> {
        let mut state = self.inner.lock().await;
        let (page_id, page) = self.attach_specific_page(&mut state, page_id).await?;
        let target_id = page_target_id(&state.metadata, &page_id, &page);
        let page = self
            .activate_or_reacquire_page(&mut state, page, target_id)
            .await?;
        self.build_live_page(page, page_id, &state.metadata)
    }

    pub(super) async fn new_page_impl(&self, url: Option<&str>) -> Result<Box<dyn PageEngine>> {
        let mut state = self.inner.lock().await;
        self.refresh_metadata(&mut state).await?;
        let destination = url.unwrap_or("about:blank");
        let page = state
            .browser
            .new_page(destination)
            .await
            .map_err(|err| Error::cdp(format!("failed to create page: {err}")))?;
        super::session::ensure_page_activated(&page).await?;
        let target_id = page.target_id().as_ref().to_string();
        let (current_url, current_title) = tokio::join!(
            super::page_state::live_page_url(&page, None),
            super::page_state::live_page_title(&page, None)
        );
        let page_id = {
            let page_state = state.metadata.ensure_page_for_target(&target_id);
            page_state.update_live_state(current_url, current_title);
            page_state.id.clone()
        };
        state.metadata.selected_page_id = Some(page_id.clone());
        state.metadata.updated_at = chrono::Utc::now();
        self.store.save(&state.metadata).await?;
        self.build_live_page(page, page_id, &state.metadata)
    }

    pub(super) async fn close_page_impl(
        &self,
        page_id: Option<&str>,
    ) -> Result<Box<dyn PageEngine>> {
        let mut state = self.inner.lock().await;
        let mut live_pages = self.sync_live_pages(&mut state).await?;
        let current_page_id = if let Some(page_id) = page_id {
            if state.metadata.page_by_id(page_id).is_none() {
                return Err(Error::SessionPageNotFound {
                    page_id: page_id.to_string(),
                });
            }
            page_id.to_string()
        } else {
            best_live_page_id(&state.metadata, &live_pages).ok_or(Error::BrowserDisconnected)?
        };
        let page = take_live_page_by_id(&state.metadata, &mut live_pages, &current_page_id)
            .ok_or_else(|| Error::SessionPageNotLive {
                page_id: current_page_id.clone(),
            })?;
        let closed_target_id = page.target_id.clone();
        self.close_live_page_target(&mut state, page.page, &current_page_id, &closed_target_id)
            .await?;
        state.metadata.remove_page(&current_page_id);
        if state.metadata.selected_page_id.as_deref() == Some(current_page_id.as_str()) {
            state.metadata.selected_page_id = None;
        }
        state.metadata.updated_at = chrono::Utc::now();
        self.store.save(&state.metadata).await?;
        let mut live_pages = self.sync_live_pages(&mut state).await?;
        if live_pages.is_empty() {
            let replacement = state
                .browser
                .new_page("about:blank")
                .await
                .map_err(|err| Error::cdp(format!("failed to create page: {err}")))?;
            let target_id = replacement.target_id().as_ref().to_string();
            let page_state = state.metadata.ensure_page_for_target(&target_id);
            page_state.update_live_state("about:blank".to_string(), String::new());
            live_pages = self.sync_live_pages(&mut state).await?;
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
        let selected_page_id = if state.metadata.selected_page_id.as_deref()
            == Some(current_page_id.as_str())
            || !selected_is_live
        {
            best_live_page_id(&state.metadata, &live_pages).ok_or(Error::BrowserDisconnected)?
        } else {
            state
                .metadata
                .selected_page_id
                .clone()
                .ok_or(Error::BrowserDisconnected)?
        };
        let page = take_live_page_by_id(&state.metadata, &mut live_pages, &selected_page_id)
            .ok_or(Error::BrowserDisconnected)?;
        state.metadata.selected_page_id = Some(selected_page_id.clone());
        state.metadata.updated_at = chrono::Utc::now();
        self.store.save(&state.metadata).await?;
        let page = self
            .activate_or_reacquire_page(&mut state, page.page, page.target_id)
            .await?;
        self.build_live_page(page, selected_page_id, &state.metadata)
    }

    fn build_live_page(
        &self,
        page: chromiumoxide::Page,
        page_id: String,
        metadata: &SessionMetadata,
    ) -> Result<Box<dyn PageEngine>> {
        let page_metadata =
            metadata
                .page_by_id(&page_id)
                .cloned()
                .ok_or_else(|| Error::SessionPageNotFound {
                    page_id: page_id.clone(),
                })?;
        Ok(Box::new(LivePage::new(
            page,
            self.config.clone(),
            LivePageSession {
                session_id: self.session_info.id.clone(),
                page_id,
                resumed: self.session_info.resumed,
            },
            Arc::clone(&self.store),
            page_metadata,
            build_image_summarizer(&self.config.ai),
        )))
    }
}
