//! Live checkpoint capture.

use std::collections::HashSet;

use chromiumoxide::cdp::browser_protocol::storage::GetCookiesParams;
use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;

use crate::checkpoint::{
    CHECKPOINT_VERSION, CheckpointCookie, CheckpointOrigin, CheckpointPage, CheckpointStorageEntry,
    SessionCheckpoint,
};
use crate::error::{Error, Result};

use super::super::engine::Browser;
use super::super::page_state::live_page_url;
use super::super::state::{LiveBrowserPage, best_live_page_id};
use super::http_origin;

const LOCAL_STORAGE_SCRIPT: &str =
    "Object.entries(window.localStorage).map(([key, value]) => ({ key, value }))";

impl Browser {
    pub(in crate::browser) async fn export_checkpoint_impl(&self) -> Result<SessionCheckpoint> {
        let mut state = self.inner.lock().await;
        self.refresh_metadata(&mut state).await?;
        let live_pages = checkpoint_pages(&mut state).await?;
        let selected_page_id =
            best_live_page_id(&state.metadata, &live_pages).ok_or_else(|| {
                Error::CheckpointCapture {
                    reason: "session has no selected live page".to_string(),
                }
            })?;
        let selected_page = live_pages
            .iter()
            .position(|page| {
                state
                    .metadata
                    .page_by_target_id(&page.target_id)
                    .is_some_and(|record| record.id == selected_page_id)
            })
            .ok_or_else(|| Error::CheckpointCapture {
                reason: "selected page is not in the live page inventory".to_string(),
            })?;
        let cookies = state
            .browser
            .execute(GetCookiesParams::default())
            .await
            .map_err(|source| Error::CheckpointCapture {
                reason: format!("failed to read browser cookies: {source}"),
            })?
            .result
            .cookies
            .into_iter()
            .map(CheckpointCookie::from)
            .collect();
        let origins = capture_origins(&live_pages).await?;
        let pages = live_pages
            .into_iter()
            .map(|page| CheckpointPage { url: page.url })
            .collect();
        let checkpoint = SessionCheckpoint {
            checkpoint: CHECKPOINT_VERSION,
            created_at: chrono::Utc::now(),
            cookies,
            origins,
            pages,
            selected_page,
        };
        checkpoint.validate()?;
        Ok(checkpoint)
    }
}

async fn checkpoint_pages(
    state: &mut super::super::state::BrowserState,
) -> Result<Vec<super::super::state::LiveBrowserPage>> {
    let records = state.metadata.pages.clone();
    let mut pages = Vec::with_capacity(records.len());
    for record in records {
        if record.target_id.is_empty() || record.target_id == "legacy" {
            continue;
        }
        let page = lookup_checkpoint_page(state, &record.target_id, &record.id).await?;
        let fallback_url = record.url();
        let url = live_page_url(&page, fallback_url).await;
        pages.push(super::super::state::LiveBrowserPage {
            target_id: record.target_id,
            page,
            url,
            title: String::new(),
        });
    }
    if pages.is_empty() {
        return Err(Error::CheckpointCapture {
            reason: "session has no live pages".to_string(),
        });
    }
    Ok(pages)
}

async fn lookup_checkpoint_page(
    state: &mut super::super::state::BrowserState,
    target_id: &str,
    page_id: &str,
) -> Result<chromiumoxide::Page> {
    if let Ok(page) = state.browser.get_page(target_id.to_string().into()).await {
        return Ok(page);
    }
    state
        .browser
        .fetch_targets()
        .await
        .map_err(|source| Error::CheckpointCapture {
            reason: format!("failed to refresh checkpoint targets: {source}"),
        })?;
    for attempt in 0..50 {
        if let Ok(page) = state.browser.get_page(target_id.to_string().into()).await {
            return Ok(page);
        }
        if attempt < 49 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(Error::CheckpointCapture {
        reason: format!("failed to attach checkpoint page {page_id}"),
    })
}

async fn capture_origins(pages: &[LiveBrowserPage]) -> Result<Vec<CheckpointOrigin>> {
    let mut seen = HashSet::new();
    let mut origins = Vec::new();
    for page in pages {
        let Some(origin) = http_origin(&page.url) else {
            continue;
        };
        if !seen.insert(origin.clone()) {
            continue;
        }
        let local_storage = evaluate_local_storage(&page.page).await?;
        origins.push(CheckpointOrigin {
            origin,
            local_storage,
        });
    }
    Ok(origins)
}

async fn evaluate_local_storage(page: &chromiumoxide::Page) -> Result<Vec<CheckpointStorageEntry>> {
    let mut params = EvaluateParams::new(LOCAL_STORAGE_SCRIPT);
    params.await_promise = Some(true);
    params.return_by_value = Some(true);
    let result = page
        .execute(params)
        .await
        .map_err(|source| Error::CheckpointCapture {
            reason: format!("failed to read localStorage: {source}"),
        })?
        .result;
    if let Some(exception) = result.exception_details {
        return Err(Error::CheckpointCapture {
            reason: format!("failed to read localStorage: {exception:?}"),
        });
    }
    serde_json::from_value(
        result
            .result
            .value
            .unwrap_or(serde_json::Value::Array(Vec::new())),
    )
    .map_err(|source| Error::CheckpointCapture {
        reason: format!("failed to decode localStorage: {source}"),
    })
}
