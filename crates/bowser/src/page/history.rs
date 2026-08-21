//! Browser-owned history traversal and reload helpers.

use chromiumoxide::cdp::browser_protocol::page::{
    GetNavigationHistoryParams, NavigateToHistoryEntryParams, ReloadParams,
};

use crate::cdp_trace;
use crate::error::{Error, Result};

use super::navigation_state::NavigationDeadline;
use super::types::LivePage;

#[derive(Clone, Copy)]
enum HistoryDirection {
    Back,
    Forward,
}

impl HistoryDirection {
    fn target_index(self, current_index: i64) -> Option<usize> {
        let index = match self {
            Self::Back => current_index.checked_sub(1)?,
            Self::Forward => current_index.checked_add(1)?,
        };
        usize::try_from(index).ok()
    }

    fn exhausted_error(self) -> Error {
        match self {
            Self::Back => Error::HistoryBackExhausted,
            Self::Forward => Error::HistoryForwardExhausted,
        }
    }
}

impl LivePage {
    pub(super) async fn back_impl(&self) -> Result<()> {
        self.history_navigation_impl(HistoryDirection::Back).await
    }

    pub(super) async fn forward_impl(&self) -> Result<()> {
        self.history_navigation_impl(HistoryDirection::Forward)
            .await
    }

    async fn history_navigation_impl(&self, direction: HistoryDirection) -> Result<()> {
        self.prepare().await?;
        let deadline = NavigationDeadline::new(self.config.timeout);
        let entry_id = self.history_entry_id(direction, &deadline).await?;
        let previous_sample = self.navigation_sample_within(&deadline).await;
        let previous_url = previous_sample.as_ref().map(|sample| sample.url.clone());
        let previous_document_key = previous_sample
            .as_ref()
            .map(|sample| sample.document_key.clone());
        self.mark_cached_document_state_stale().await?;
        self.navigate_to_history_entry(entry_id, &deadline).await?;
        let changed = self
            .wait_for_history_document_within(
                previous_url.as_deref(),
                previous_document_key.as_deref(),
                &deadline,
            )
            .await;
        if !changed {
            return Err(Error::Timeout {
                seconds: deadline.timeout_seconds(),
            });
        }
        self.clear_cached_document_state(deadline.state_read_timeout())
            .await?;
        Ok(())
    }

    async fn history_entry_id(
        &self,
        direction: HistoryDirection,
        deadline: &NavigationDeadline,
    ) -> Result<i64> {
        cdp_trace::record_method("Page.getNavigationHistory");
        let Some(result) = deadline
            .run_navigation_call(self.page.execute(GetNavigationHistoryParams::default()))
            .await
        else {
            return Err(Error::Timeout {
                seconds: deadline.timeout_seconds(),
            });
        };
        let history = match result {
            Ok(history) => history,
            Err(source) => {
                return Err(Error::HistoryRead { source });
            }
        };
        let entry_id = direction
            .target_index(history.current_index)
            .and_then(|index| history.entries.get(index))
            .map(|entry| entry.id);
        entry_id.ok_or(direction.exhausted_error())
    }

    async fn navigate_to_history_entry(
        &self,
        entry_id: i64,
        deadline: &NavigationDeadline,
    ) -> Result<()> {
        cdp_trace::record_method("Page.navigateToHistoryEntry");
        let Some(result) = deadline
            .run_navigation_call(
                self.page
                    .execute(NavigateToHistoryEntryParams::new(entry_id)),
            )
            .await
        else {
            return Err(Error::Timeout {
                seconds: deadline.timeout_seconds(),
            });
        };
        if let Err(source) = result {
            return Err(Error::HistoryNavigation { source });
        }
        Ok(())
    }

    pub(super) async fn reload_impl(&self) -> Result<()> {
        self.prepare().await?;
        let deadline = NavigationDeadline::new(self.config.timeout);
        let previous_sample = self.navigation_sample_within(&deadline).await;
        let previous_url = previous_sample.as_ref().map(|sample| sample.url.clone());
        let previous_document_key = previous_sample
            .as_ref()
            .map(|sample| sample.document_key.clone());
        let previous_loader_id = if previous_sample.is_none() {
            self.main_frame_loader_id_within(&deadline).await
        } else {
            None
        };
        if deadline.remaining().is_none() {
            return Err(Error::Timeout {
                seconds: deadline.timeout_seconds(),
            });
        }
        self.mark_cached_document_state_stale().await?;
        cdp_trace::record_method("Page.reload");
        let reload_result = deadline
            .run_navigation_call(self.page.execute(ReloadParams::default()))
            .await;
        match reload_result {
            Some(Ok(_)) => {}
            Some(Err(err)) => {
                return Err(Error::Navigation {
                    url: format!("reload failed: {err}"),
                });
            }
            None => {
                return Err(Error::Timeout {
                    seconds: self.config.timeout.as_secs(),
                });
            }
        }
        let reloaded = if previous_sample.is_some() {
            self.wait_for_changed_document_within(
                previous_url.as_deref(),
                previous_document_key.as_deref(),
                &deadline,
            )
            .await
        } else if let Some(previous_loader_id) = previous_loader_id.as_deref() {
            self.wait_for_ready_document_with_loader_change_within(previous_loader_id, &deadline)
                .await
        } else {
            false
        };
        if !reloaded {
            return Err(Error::Timeout {
                seconds: self.config.timeout.as_secs(),
            });
        }
        self.clear_cached_document_state(deadline.state_read_timeout())
            .await?;
        Ok(())
    }
}
