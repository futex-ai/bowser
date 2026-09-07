//! Page target closing and reacquisition helpers.

use std::time::Duration;

use chromiumoxide::{
    browser::Browser as ChromiumBrowser,
    cdp::browser_protocol::target::{CloseTargetParams, GetTargetsParams},
};

use crate::error::{Error, Result, is_target_session_missing};

use super::{engine::Browser, session::ensure_page_activated, state::BrowserState};

const PAGE_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

impl Browser {
    pub(super) async fn close_live_page_target(
        &self,
        state: &mut BrowserState,
        page: chromiumoxide::Page,
        page_id: &str,
        target_id: &str,
    ) -> Result<()> {
        let _ = tokio::time::timeout(PAGE_CLOSE_TIMEOUT, page.close()).await;
        if self.wait_for_target_to_close(state, target_id).await? {
            return Ok(());
        }
        let force_close_result = self
            .force_close_target(&state.browser, page_id, target_id)
            .await;
        if self.wait_for_target_to_close(state, target_id).await? {
            return Ok(());
        }
        force_close_result?;
        Err(Error::PageCloseTimeout {
            page_id: page_id.to_string(),
            seconds: PAGE_CLOSE_TIMEOUT.as_secs(),
        })
    }

    pub(super) async fn activate_or_reacquire_page(
        &self,
        state: &mut BrowserState,
        page: chromiumoxide::Page,
        target_id: String,
    ) -> Result<chromiumoxide::Page> {
        match ensure_page_activated(&page).await {
            Ok(()) => Ok(page),
            Err(Error::PageTargetSessionInvalid { .. }) => {
                let page = self
                    .lookup_page_by_target_id(state, &target_id)
                    .await?
                    .ok_or(Error::BrowserDisconnected)?;
                ensure_page_activated(&page).await?;
                Ok(page)
            }
            Err(err) => Err(err),
        }
    }

    async fn force_close_target(
        &self,
        browser: &ChromiumBrowser,
        page_id: &str,
        target_id: &str,
    ) -> Result<()> {
        match tokio::time::timeout(
            PAGE_CLOSE_TIMEOUT,
            browser.execute(CloseTargetParams::new(target_id.to_string())),
        )
        .await
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) => {
                let reason = err.to_string();
                if is_target_session_missing(&reason) {
                    Ok(())
                } else {
                    Err(Error::PageClose {
                        page_id: page_id.to_string(),
                        reason,
                    })
                }
            }
            Err(_) => Err(Error::PageCloseTimeout {
                page_id: page_id.to_string(),
                seconds: PAGE_CLOSE_TIMEOUT.as_secs(),
            }),
        }
    }

    async fn wait_for_target_to_close(
        &self,
        state: &mut BrowserState,
        target_id: &str,
    ) -> Result<bool> {
        for attempt in 0..20 {
            state
                .browser
                .execute(GetTargetsParams::default())
                .await
                .map_err(|err| Error::cdp(format!("failed to fetch targets: {err}")))?;
            let pages = state
                .browser
                .pages()
                .await
                .map_err(|err| Error::cdp(format!("failed to list pages: {err}")))?;
            if !pages
                .iter()
                .any(|page| page.target_id().as_ref() == target_id)
            {
                return Ok(true);
            }
            if attempt < 19 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        Ok(false)
    }
}
