//! Live page URL, title, HTML, selector, and JavaScript helpers.

use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::stability;

use super::types::LivePage;

const STRICT_STABILITY_NETWORK_GRACE: Duration = Duration::from_millis(1500);
const DOM_QUIET_WINDOW: Duration = Duration::from_millis(350);

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct StabilitySample {
    ready: bool,
    signature: String,
}

impl LivePage {
    pub(super) async fn url_impl(&self) -> Result<String> {
        self.prepare().await?;
        let url = self
            .page
            .url()
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?
            .unwrap_or_default();
        if url.is_empty() {
            return self
                .evaluate_value("window.location.href".to_string())
                .await;
        }
        Ok(url)
    }

    pub(super) async fn title_impl(&self) -> Result<String> {
        self.prepare().await?;
        let title = self
            .page
            .get_title()
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?
            .unwrap_or_default();
        if title.is_empty() {
            return self.evaluate_value("document.title".to_string()).await;
        }
        Ok(title)
    }

    pub(super) async fn rendered_html_impl(&self) -> Result<String> {
        self.prepare().await?;
        let _ = self.wait_for_stable_impl(Duration::from_secs(1)).await;
        self.evaluate_value(
            "document.documentElement ? document.documentElement.outerHTML : ''".to_string(),
        )
        .await
    }

    pub(super) async fn wait_for_selector_impl(
        &self,
        selector: &str,
        timeout: Duration,
    ) -> Result<()> {
        self.prepare().await?;
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.page.find_element(selector).await.is_ok() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(Error::Timeout {
            seconds: timeout.as_secs(),
        })
    }

    pub(super) async fn wait_for_stable_impl(&self, timeout: Duration) -> Result<()> {
        self.prepare().await?;
        let start = Instant::now();
        let strict_expression = stability::sample_expression(true);
        let relaxed_expression = stability::sample_expression(false);
        let mut last_sample = None;
        let mut quiet_since = None;
        while start.elapsed() < timeout {
            let expression = if start.elapsed() < STRICT_STABILITY_NETWORK_GRACE.min(timeout) {
                strict_expression.clone()
            } else {
                relaxed_expression.clone()
            };
            let sample: Option<StabilitySample> = self.evaluate_bowser_value(expression).await.ok();
            if let Some(sample) = sample {
                let now = Instant::now();
                if sample.ready && last_sample.as_ref() == Some(&sample) {
                    let since = quiet_since.get_or_insert(now);
                    if now.duration_since(*since) >= DOM_QUIET_WINDOW {
                        return Ok(());
                    }
                } else if sample.ready {
                    quiet_since = Some(now);
                    last_sample = Some(sample);
                } else {
                    quiet_since = None;
                    last_sample = Some(sample);
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(Error::Timeout {
            seconds: timeout.as_secs(),
        })
    }

    pub(super) async fn evaluate_js_impl(&self, expression: &str) -> Result<String> {
        let value: serde_json::Value = self.evaluate_value(expression.to_string()).await?;
        serde_json::to_string_pretty(&value).map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })
    }
}
