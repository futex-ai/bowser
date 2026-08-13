//! Fresh-session checkpoint restoration.

use std::collections::{HashMap, HashSet};

use chromiumoxide::cdp::browser_protocol::network::{CookieParam, SetCookiesParams};

use crate::checkpoint::{CheckpointStorageEntry, SessionCheckpoint};
use crate::error::{Error, Result};
use crate::page::PageEngine;

use super::super::engine::{Browser, BrowserEngine};
use super::http_origin;

impl Browser {
    /// Restores a validated checkpoint into a fresh Bowser-owned session.
    pub async fn restore(
        mut config: crate::BrowserConfig,
        checkpoint: SessionCheckpoint,
    ) -> Result<Self> {
        checkpoint.validate()?;
        if config.session.id.is_some() {
            return Err(Error::CheckpointRestore {
                session_id: None,
                reason: "restore cannot resume an existing --session".to_string(),
            });
        }
        config.persistent_profile = false;
        config.user_data_dir = None;
        let browser = Self::launch(config).await?;
        if let Err(error) = browser.restore_checkpoint_impl(&checkpoint).await {
            let session_id = browser.session_info.id.clone();
            let reason = match error {
                Error::CheckpointRestore { reason, .. } => reason,
                error => error.to_string(),
            };
            let _ = browser.close().await;
            return Err(Error::CheckpointRestore {
                session_id: Some(session_id),
                reason,
            });
        }
        Ok(browser)
    }

    async fn restore_checkpoint_impl(&self, checkpoint: &SessionCheckpoint) -> Result<()> {
        self.normalize_fresh_pages().await?;
        self.restore_cookies(checkpoint).await?;
        let origin_entries = checkpoint
            .origins
            .iter()
            .map(|origin| (origin.origin.as_str(), origin.local_storage.as_slice()))
            .collect::<HashMap<_, _>>();
        let mut restored_origins = HashSet::new();
        let mut page_ids = Vec::with_capacity(checkpoint.pages.len());
        for (index, saved_page) in checkpoint.pages.iter().enumerate() {
            let page = if index == 0 {
                let page = self.current_page().await?;
                page.navigate(&saved_page.url).await?;
                page
            } else {
                self.new_page(Some(&saved_page.url)).await?
            };
            restore_origin_storage(
                page.as_ref(),
                &saved_page.url,
                &origin_entries,
                &mut restored_origins,
            )
            .await?;
            let page_id = self.session_info().await?.selected_page_id.ok_or_else(|| {
                Error::CheckpointRestore {
                    session_id: None,
                    reason: "restored page has no Bowser page identifier".to_string(),
                }
            })?;
            page_ids.push(page_id);
        }
        let selected_page_id =
            page_ids
                .get(checkpoint.selected_page)
                .ok_or_else(|| Error::CheckpointRestore {
                    session_id: None,
                    reason: "restored selected page is unavailable".to_string(),
                })?;
        self.select_page(selected_page_id).await?;
        Ok(())
    }

    async fn normalize_fresh_pages(&self) -> Result<()> {
        let pages = self.list_pages().await?;
        let retained_page_id = pages
            .iter()
            .find(|page| page.live && page.selected)
            .or_else(|| pages.iter().find(|page| page.live))
            .map(|page| page.id.clone())
            .ok_or_else(|| Error::CheckpointRestore {
                session_id: None,
                reason: "fresh browser has no page available for restore".to_string(),
            })?;
        for page in pages {
            if page.live && page.id != retained_page_id {
                self.close_page(Some(&page.id)).await?;
            }
        }
        self.select_page(&retained_page_id).await?;
        Ok(())
    }

    async fn restore_cookies(&self, checkpoint: &SessionCheckpoint) -> Result<()> {
        if checkpoint.cookies.is_empty() {
            return Ok(());
        }
        let mut state = self.inner.lock().await;
        let (_, page) = self.attach_best_page(&mut state).await?;
        let target_id = page.target_id().as_ref().to_string();
        let page = self
            .activate_or_reacquire_page(&mut state, page, target_id)
            .await?;
        let cookies = checkpoint
            .cookies
            .iter()
            .cloned()
            .map(CookieParam::from)
            .collect::<Vec<_>>();
        page.execute(SetCookiesParams::new(cookies))
            .await
            .map_err(|source| Error::CheckpointRestore {
                session_id: None,
                reason: format!("failed to restore cookies: {source}"),
            })?;
        Ok(())
    }
}

async fn restore_origin_storage(
    page: &dyn PageEngine,
    url: &str,
    origins: &HashMap<&str, &[CheckpointStorageEntry]>,
    restored_origins: &mut HashSet<String>,
) -> Result<()> {
    let Some(origin) = http_origin(url) else {
        return Ok(());
    };
    if !restored_origins.insert(origin.clone()) {
        return Ok(());
    }
    let Some(entries) = origins.get(origin.as_str()) else {
        return Ok(());
    };
    let entries = serde_json::to_string(entries).map_err(|source| Error::CheckpointRestore {
        session_id: None,
        reason: format!("failed to encode localStorage for {origin}: {source}"),
    })?;
    let script = format!(
        "(() => {{ localStorage.clear(); for (const entry of {entries}) localStorage.setItem(entry.key, entry.value); return true; }})()"
    );
    page.evaluate_js(&script).await?;
    page.reload().await?;
    Ok(())
}
