//! Navigation helpers.

use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::page::{NavigateParams, ReloadParams};

use crate::cdp_trace;
use crate::error::{Error, Result};

use super::navigation_state::{NavigationDeadline, RedirectReadiness};
use super::types::LivePage;

const LOCATION_ASSIGNMENT_READY_TIMEOUT: Duration = Duration::from_secs(2);

enum BrowserNavigationAttempt {
    Accepted { loader_id: Option<String> },
    NotAccepted,
}

impl LivePage {
    pub(super) async fn navigate_impl(&self, url: &str) -> Result<()> {
        self.prepare().await?;
        let deadline = NavigationDeadline::new(self.config.timeout);
        let previous_sample = self.navigation_sample_within(&deadline).await;
        let previous_url = previous_sample.as_ref().map(|sample| sample.url.clone());
        let previous_document_key = previous_sample
            .as_ref()
            .map(|sample| sample.document_key.clone());
        if deadline.remaining().is_none() {
            return Err(Error::Timeout {
                seconds: deadline.timeout_seconds(),
            });
        }
        self.mark_cached_document_state_stale().await?;
        if self.resumed
            && self
                .location_assignment_navigation(
                    url,
                    previous_url.as_deref(),
                    previous_document_key.as_deref(),
                    &deadline,
                )
                .await?
        {
            return Ok(());
        }
        cdp_trace::record_method("Page.navigate");
        if let BrowserNavigationAttempt::Accepted { loader_id } =
            self.browser_navigation_attempt(url, &deadline).await
            && self
                .finish_assigned_document_within(
                    url,
                    previous_url.as_deref(),
                    previous_document_key.as_deref(),
                    loader_id.as_deref(),
                    RedirectReadiness::LoaderCorrelated,
                    &deadline,
                )
                .await?
        {
            return Ok(());
        }
        if !self.resumed
            && self
                .location_assignment_navigation(
                    url,
                    previous_url.as_deref(),
                    previous_document_key.as_deref(),
                    &deadline,
                )
                .await?
        {
            return Ok(());
        }
        self.clear_cached_document_state(deadline.state_read_timeout())
            .await?;
        Err(Error::Navigation {
            url: url.to_string(),
        })
    }

    pub(super) async fn back_impl(&self) -> Result<()> {
        self.prepare().await?;
        let deadline = NavigationDeadline::new(self.config.timeout);
        let previous_sample = self.navigation_sample_within(&deadline).await;
        let previous_url = previous_sample.as_ref().map(|sample| sample.url.clone());
        let previous_document_key = previous_sample
            .as_ref()
            .map(|sample| sample.document_key.clone());
        self.mark_cached_document_state_stale().await?;
        let _ = self
            .evaluate_side_effect_within("history.back()".to_string(), &deadline)
            .await;
        self.wait_for_history_document_within(
            previous_url.as_deref(),
            previous_document_key.as_deref(),
            &deadline,
        )
        .await;
        self.clear_cached_document_state(deadline.state_read_timeout())
            .await?;
        Ok(())
    }

    pub(super) async fn forward_impl(&self) -> Result<()> {
        self.prepare().await?;
        let deadline = NavigationDeadline::new(self.config.timeout);
        let previous_sample = self.navigation_sample_within(&deadline).await;
        let previous_url = previous_sample.as_ref().map(|sample| sample.url.clone());
        let previous_document_key = previous_sample
            .as_ref()
            .map(|sample| sample.document_key.clone());
        self.mark_cached_document_state_stale().await?;
        let _ = self
            .evaluate_side_effect_within("history.forward()".to_string(), &deadline)
            .await;
        self.wait_for_history_document_within(
            previous_url.as_deref(),
            previous_document_key.as_deref(),
            &deadline,
        )
        .await;
        self.clear_cached_document_state(deadline.state_read_timeout())
            .await?;
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
        if reloaded {
            self.clear_cached_document_state(deadline.state_read_timeout())
                .await?;
            Ok(())
        } else {
            Err(Error::Timeout {
                seconds: self.config.timeout.as_secs(),
            })
        }
    }

    async fn browser_navigation_attempt(
        &self,
        url: &str,
        deadline: &NavigationDeadline,
    ) -> BrowserNavigationAttempt {
        let Some(result) = deadline
            .run_navigation_call(self.page.execute(NavigateParams::new(url)))
            .await
        else {
            return BrowserNavigationAttempt::NotAccepted;
        };
        let Ok(result) = result else {
            return BrowserNavigationAttempt::NotAccepted;
        };
        if result.error_text.is_some() {
            return BrowserNavigationAttempt::NotAccepted;
        }
        BrowserNavigationAttempt::Accepted {
            loader_id: result
                .loader_id
                .as_ref()
                .map(|loader_id| loader_id.as_ref().to_string()),
        }
    }

    async fn location_assignment_navigation(
        &self,
        url: &str,
        previous_url: Option<&str>,
        previous_document_key: Option<&str>,
        deadline: &NavigationDeadline,
    ) -> Result<bool> {
        let destination = serde_json::to_string(url).map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })?;
        let location_deadline = deadline.capped(LOCATION_ASSIGNMENT_READY_TIMEOUT);
        cdp_trace::record_method("Page.locationAssign");
        if self
            .evaluate_side_effect_within(
                format!("window.location.href = {destination};"),
                &location_deadline,
            )
            .await
            .is_err()
        {
            return Ok(false);
        }
        self.finish_assigned_document_within(
            url,
            previous_url,
            previous_document_key,
            None,
            RedirectReadiness::ChangedDocument,
            deadline,
        )
        .await
    }
}
