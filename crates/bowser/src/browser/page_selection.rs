//! Page selection and attachment helpers.

use crate::error::{Error, Result};

use super::{
    engine::Browser,
    state::{BrowserState, best_live_page_id, take_live_page_by_id},
};

impl Browser {
    pub(super) async fn create_blank_page(
        &self,
        state: &mut BrowserState,
    ) -> Result<(String, chromiumoxide::Page)> {
        let unmanaged_blank_pages = self
            .list_live_pages(state)
            .await?
            .into_iter()
            .filter(|page| {
                (page.url.is_empty() || page.url == "about:blank")
                    && state.metadata.page_by_target_id(&page.target_id).is_none()
            })
            .collect::<Vec<_>>();
        let page = state
            .browser
            .new_page("about:blank")
            .await
            .map_err(|err| Error::cdp(format!("failed to create page: {err}")))?;
        let target_id = page.target_id().as_ref().to_string();
        for unmanaged_page in unmanaged_blank_pages {
            if let Err(error) = self
                .close_live_page_target(
                    state,
                    unmanaged_page.page,
                    "Chrome startup page",
                    &unmanaged_page.target_id,
                )
                .await
            {
                let _ = self
                    .close_live_page_target(
                        state,
                        page.clone(),
                        "unfinished Bowser page",
                        &target_id,
                    )
                    .await;
                return Err(error);
            }
        }
        let page_id = {
            let page_state = state.metadata.ensure_page_for_target(&target_id);
            page_state.update_live_state("about:blank".to_string(), String::new());
            page_state.id.clone()
        };
        state.metadata.selected_page_id = Some(page_id.clone());
        state.metadata.updated_at = chrono::Utc::now();
        self.store.save(&state.metadata).await?;
        Ok((page_id, page))
    }

    pub(super) async fn attach_best_page(
        &self,
        state: &mut BrowserState,
    ) -> Result<(String, chromiumoxide::Page)> {
        self.refresh_metadata(state).await?;
        if let Some(selected_page_id) = state.metadata.selected_page_id.clone()
            && let Some(target_id) = state
                .metadata
                .page_by_id(&selected_page_id)
                .map(|page| page.target_id.clone())
                .filter(|target_id| !target_id.is_empty() && target_id != "legacy")
            && let Some(page) = self.lookup_page_by_target_id(state, &target_id).await?
        {
            return Ok((selected_page_id, page));
        }
        if state.metadata.pages.is_empty() {
            return self.create_blank_page(state).await;
        }
        let mut live_pages = self.sync_live_pages(state).await?;
        if live_pages.is_empty() {
            return self.create_blank_page(state).await;
        }
        let page_id =
            best_live_page_id(&state.metadata, &live_pages).ok_or(Error::BrowserDisconnected)?;
        let page = take_live_page_by_id(&state.metadata, &mut live_pages, &page_id)
            .ok_or(Error::BrowserDisconnected)?;
        if state.metadata.selected_page_id.as_deref() != Some(page_id.as_str()) {
            state.metadata.selected_page_id = Some(page_id.clone());
            state.metadata.updated_at = chrono::Utc::now();
            self.store.save(&state.metadata).await?;
        }
        Ok((page_id, page.page))
    }

    pub(super) async fn attach_specific_page(
        &self,
        state: &mut BrowserState,
        page_id: &str,
    ) -> Result<(String, chromiumoxide::Page)> {
        self.refresh_metadata(state).await?;
        let Some(page_record) = state.metadata.page_by_id(page_id).cloned() else {
            return Err(Error::SessionPageNotFound {
                page_id: page_id.to_string(),
            });
        };
        if !page_record.target_id.is_empty()
            && page_record.target_id != "legacy"
            && let Some(page) = self
                .lookup_page_by_target_id(state, &page_record.target_id)
                .await?
        {
            if state.metadata.selected_page_id.as_deref() != Some(page_id) {
                state.metadata.selected_page_id = Some(page_id.to_string());
                state.metadata.updated_at = chrono::Utc::now();
                self.store.save(&state.metadata).await?;
            }
            return Ok((page_id.to_string(), page));
        }
        let mut live_pages = self.sync_live_pages(state).await?;
        let page =
            take_live_page_by_id(&state.metadata, &mut live_pages, page_id).ok_or_else(|| {
                Error::SessionPageNotLive {
                    page_id: page_id.to_string(),
                }
            })?;
        if state.metadata.selected_page_id.as_deref() != Some(page_id) {
            state.metadata.selected_page_id = Some(page_id.to_string());
            state.metadata.updated_at = chrono::Utc::now();
            self.store.save(&state.metadata).await?;
        }
        Ok((page_id.to_string(), page.page))
    }
}
