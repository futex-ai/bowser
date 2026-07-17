//! Navigation deadlines and document readiness sampling.

use std::future::Future;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::error::{Error, Result};

use super::types::LivePage;

pub(super) const ABOUT_BLANK_URL: &str = "about:blank";
const DOCUMENT_STATE_READ_TIMEOUT: Duration = Duration::from_secs(1);
pub(super) const NAVIGATION_READY_POLL: Duration = Duration::from_millis(100);
const NAVIGATION_SAMPLE_EXPRESSION: &str = r#"
(() => ({
  url: window.location.href,
  ready: document.readyState !== 'loading',
  documentKey: String(performance.timeOrigin)
}))()
"#;

pub(super) struct NavigationDeadline {
    expires_at: Instant,
    timeout: Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RedirectReadiness {
    LoaderCorrelated,
    ChangedDocument,
}

impl NavigationDeadline {
    pub(super) fn new(timeout: Duration) -> Self {
        let now = Instant::now();
        let expires_at = match now.checked_add(timeout) {
            Some(expires_at) => expires_at,
            None => now,
        };
        Self {
            expires_at,
            timeout,
        }
    }

    pub(super) fn remaining(&self) -> Option<Duration> {
        self.expires_at
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
    }

    pub(super) fn capped(&self, timeout: Duration) -> Self {
        let now = Instant::now();
        let remaining = self.remaining().unwrap_or(Duration::ZERO).min(timeout);
        let expires_at = match now.checked_add(remaining) {
            Some(expires_at) => expires_at,
            None => now,
        };
        Self {
            expires_at,
            timeout: self.timeout,
        }
    }

    pub(super) async fn run<F, T>(&self, future: F) -> Option<T>
    where
        F: Future<Output = T>,
    {
        let remaining = self.remaining()?;
        tokio::time::timeout(remaining, future).await.ok()
    }

    pub(super) async fn run_navigation_call<F, T>(&self, future: F) -> Option<T>
    where
        F: Future<Output = T>,
    {
        self.run(future).await
    }

    pub(super) fn state_read_timeout(&self) -> Duration {
        self.remaining()
            .map(|remaining| remaining.min(DOCUMENT_STATE_READ_TIMEOUT))
            .unwrap_or(Duration::ZERO)
    }

    pub(super) fn timeout_seconds(&self) -> u64 {
        self.timeout.as_secs()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(super) struct NavigationSample {
    pub(super) url: String,
    pub(super) ready: bool,
    #[serde(rename = "documentKey")]
    pub(super) document_key: String,
}

impl LivePage {
    pub(super) async fn evaluate_side_effect_within(
        &self,
        expression: String,
        deadline: &NavigationDeadline,
    ) -> Result<()> {
        let Some(result) = deadline.run(self.evaluate_side_effect(expression)).await else {
            return Err(Error::Timeout {
                seconds: deadline.timeout_seconds(),
            });
        };
        result
    }

    pub(super) async fn navigation_sample_within(
        &self,
        deadline: &NavigationDeadline,
    ) -> Option<NavigationSample> {
        let timeout = deadline.state_read_timeout();
        if timeout.is_zero() {
            return None;
        }
        tokio::time::timeout(
            timeout,
            self.evaluate_raw_value(NAVIGATION_SAMPLE_EXPRESSION.to_string()),
        )
        .await
        .ok()?
        .ok()
    }

    pub(super) async fn finish_assigned_document_within(
        &self,
        requested_url: &str,
        previous_url: Option<&str>,
        previous_document_key: Option<&str>,
        accepted_loader_id: Option<&str>,
        redirect_readiness: RedirectReadiness,
        deadline: &NavigationDeadline,
    ) -> Result<bool> {
        if !self
            .wait_for_assigned_document_within(
                requested_url,
                previous_url,
                previous_document_key,
                accepted_loader_id,
                redirect_readiness,
                deadline,
            )
            .await
        {
            return Ok(false);
        }
        self.clear_cached_document_state(deadline.state_read_timeout())
            .await?;
        Ok(true)
    }

    pub(super) async fn wait_for_changed_document_within(
        &self,
        previous_url: Option<&str>,
        previous_document_key: Option<&str>,
        deadline: &NavigationDeadline,
    ) -> bool {
        while let Some(remaining) = deadline.remaining() {
            if self
                .navigation_sample_within(deadline)
                .await
                .as_ref()
                .is_some_and(|sample| {
                    changed_document_ready(sample, previous_url, previous_document_key)
                })
            {
                return true;
            }
            tokio::time::sleep(NAVIGATION_READY_POLL.min(remaining)).await;
        }
        false
    }

    pub(super) async fn wait_for_history_document_within(
        &self,
        previous_url: Option<&str>,
        previous_document_key: Option<&str>,
        deadline: &NavigationDeadline,
    ) {
        let _ = self
            .wait_for_changed_document_within(previous_url, previous_document_key, deadline)
            .await;
    }

    async fn wait_for_assigned_document_within(
        &self,
        requested_url: &str,
        previous_url: Option<&str>,
        previous_document_key: Option<&str>,
        accepted_loader_id: Option<&str>,
        redirect_readiness: RedirectReadiness,
        deadline: &NavigationDeadline,
    ) -> bool {
        while let Some(remaining) = deadline.remaining() {
            if let Some(sample) = self.navigation_sample_within(deadline).await {
                if requested_document_ready(
                    &sample,
                    requested_url,
                    previous_url,
                    previous_document_key,
                ) {
                    return true;
                }
                let redirected_ready = redirected_document_ready(
                    &sample,
                    requested_url,
                    previous_url,
                    previous_document_key,
                );
                if redirected_ready
                    && matches!(redirect_readiness, RedirectReadiness::ChangedDocument)
                {
                    return true;
                }
                if matches!(redirect_readiness, RedirectReadiness::LoaderCorrelated)
                    && (redirected_ready
                        || loader_correlated_redirect_ready(&sample, requested_url))
                    && let Some(accepted_loader_id) = accepted_loader_id
                {
                    let current_loader_id = self.main_frame_loader_id_within(deadline).await;
                    if current_loader_id.as_deref() == Some(accepted_loader_id) {
                        return true;
                    }
                }
            }
            tokio::time::sleep(NAVIGATION_READY_POLL.min(remaining)).await;
        }
        false
    }
}

pub(super) fn requested_document_ready(
    sample: &NavigationSample,
    requested_url: &str,
    previous_url: Option<&str>,
    previous_document_key: Option<&str>,
) -> bool {
    if !sample.ready {
        return false;
    }
    if sample.url == requested_url {
        return previous_url != Some(requested_url)
            || document_changed(sample, previous_document_key);
    }
    false
}

pub(super) fn redirected_document_ready(
    sample: &NavigationSample,
    requested_url: &str,
    previous_url: Option<&str>,
    previous_document_key: Option<&str>,
) -> bool {
    if !sample.ready || sample.url == requested_url || sample.url == ABOUT_BLANK_URL {
        return false;
    }
    previous_url.is_some_and(|previous| sample.url != previous)
        || document_changed(sample, previous_document_key)
}

pub(super) fn loader_correlated_redirect_ready(
    sample: &NavigationSample,
    requested_url: &str,
) -> bool {
    sample.ready && sample.url != requested_url && sample.url != ABOUT_BLANK_URL
}

pub(super) fn changed_document_ready(
    sample: &NavigationSample,
    previous_url: Option<&str>,
    previous_document_key: Option<&str>,
) -> bool {
    if !sample.ready {
        return false;
    }
    let has_previous_sample = previous_url.is_some() || previous_document_key.is_some();
    if !has_previous_sample {
        return false;
    }
    if sample.url == ABOUT_BLANK_URL {
        return previous_url == Some(ABOUT_BLANK_URL);
    }
    previous_url.is_some_and(|previous| sample.url != previous)
        || document_changed(sample, previous_document_key)
}

fn document_changed(sample: &NavigationSample, previous_document_key: Option<&str>) -> bool {
    previous_document_key.is_some_and(|previous| sample.document_key != previous)
}

#[cfg(test)]
#[path = "_tests_/navigation_state_tests.rs"]
mod navigation_state_tests;
