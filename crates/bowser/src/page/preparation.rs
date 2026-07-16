//! Live-page preparation before navigation, capture, and interaction.

use std::time::Duration;

use crate::error::{Error, Result, is_target_session_missing};
use crate::stealth_features::StealthFeatures;
use crate::{browser_identity, cdp_trace};

use super::types::LivePage;

const TARGET_SESSION_RETRY_ATTEMPTS: usize = 10;
const TARGET_SESSION_RETRY_DELAY: Duration = Duration::from_millis(200);

impl LivePage {
    pub(super) async fn prepare(&self) -> Result<()> {
        {
            let state = self.state.lock().await;
            if state.prepared {
                return Ok(());
            }
        }
        self.wait_for_target_session().await?;
        if self.config.stealth {
            let user_agent =
                self.page
                    .user_agent()
                    .await
                    .map_err(|err| Error::StealthInjection {
                        reason: format!("failed to query user agent: {err}"),
                    })?;
            let cleaned_user_agent =
                browser_identity::browser_product_user_agent(&self.page, &user_agent).await;
            let profile = browser_identity::desktop_profile(cleaned_user_agent);
            browser_identity::apply_profile_to_page(&self.page, &profile).await?;
            self.install_new_document_script(
                "stealth identity",
                browser_identity::profile_init_script(&profile)?,
            )
            .await?;
        }
        if StealthFeatures::from_config(&self.config).runtime_disable() {
            self.disable_runtime_events().await;
        }
        self.state.lock().await.prepared = true;
        Ok(())
    }

    async fn install_new_document_script(&self, label: &str, script: String) -> Result<()> {
        let mut installed = false;
        let mut last_reason = None;
        for attempt in 0..TARGET_SESSION_RETRY_ATTEMPTS {
            cdp_trace::record_method("Page.addScriptToEvaluateOnNewDocument");
            match self.page.evaluate_on_new_document(script.clone()).await {
                Ok(_) => {
                    installed = true;
                    break;
                }
                Err(err) => {
                    let reason = err.to_string();
                    if !is_target_session_missing(&reason) {
                        return Err(Error::StealthInjection {
                            reason: format!("failed to install {label} script: {reason}"),
                        });
                    }
                    last_reason = Some(reason);
                    let _ = self.page.activate().await;
                    let _ = self.wait_for_target_session().await;
                    if attempt + 1 < TARGET_SESSION_RETRY_ATTEMPTS {
                        tokio::time::sleep(TARGET_SESSION_RETRY_DELAY).await;
                    }
                }
            }
        }
        if !installed {
            return Err(Error::StealthInjection {
                reason: format!(
                    "failed to install {label} script: {}",
                    last_reason.unwrap_or_else(|| "unknown target session error".to_string())
                ),
            });
        }
        self.evaluate_preparation_script(&script).await;
        Ok(())
    }

    async fn evaluate_preparation_script(&self, script: &str) {
        let _ = self
            .evaluate_raw_value::<serde_json::Value>(script.to_string())
            .await;
    }

    async fn wait_for_target_session(&self) -> Result<()> {
        let mut last_reason = None;
        for attempt in 0..TARGET_SESSION_RETRY_ATTEMPTS {
            cdp_trace::record_method("Runtime.evaluate");
            match self.page.evaluate("1").await {
                Ok(_) => return Ok(()),
                Err(err) => {
                    let reason = err.to_string();
                    if !is_target_session_missing(&reason) {
                        return Err(Error::JsEvaluation {
                            reason: format!("failed to attach target session: {reason}"),
                        });
                    }
                    last_reason = Some(reason);
                    let _ = self.page.activate().await;
                    if attempt + 1 < TARGET_SESSION_RETRY_ATTEMPTS {
                        tokio::time::sleep(TARGET_SESSION_RETRY_DELAY).await;
                    }
                }
            }
        }
        Err(Error::JsEvaluation {
            reason: format!(
                "failed to attach target session: {}",
                last_reason.unwrap_or_else(|| "unknown target session error".to_string())
            ),
        })
    }

    async fn disable_runtime_events(&self) {
        cdp_trace::record_method("Runtime.disable");
        let _ = self.page.disable_runtime().await;
    }
}
