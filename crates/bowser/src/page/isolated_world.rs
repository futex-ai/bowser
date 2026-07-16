//! Isolated DevTools world helpers for Bowser-owned page bookkeeping.

use std::time::Duration;

use chromiumoxide::Page as ChromiumPage;
use chromiumoxide::cdp::browser_protocol::page::{CreateIsolatedWorldParams, FrameId};
use chromiumoxide::cdp::js_protocol::runtime::{EvaluateParams, ExecutionContextId};
use serde::Deserialize;
use tokio::time::sleep;

use crate::cdp_trace;
use crate::error::{Error, Result};

use super::types::LivePage;

const BOWSER_WORLD_NAME: &str = "__bowser_capture__";
const MAIN_FRAME_RETRY_ATTEMPTS: usize = 20;
const MAIN_FRAME_RETRY_DELAY: Duration = Duration::from_millis(25);

impl LivePage {
    pub(super) async fn evaluate_bowser_value<T: for<'de> Deserialize<'de>>(
        &self,
        expression: String,
    ) -> Result<T> {
        let frame_id = self.main_frame_id().await?;
        evaluate_bowser_world_value_for_page(&self.page, &frame_id, expression).await
    }

    async fn main_frame_id(&self) -> Result<FrameId> {
        let mut last_error = None;
        for attempt in 0..MAIN_FRAME_RETRY_ATTEMPTS {
            match self.page.mainframe().await {
                Ok(Some(frame_id)) => return Ok(frame_id),
                Ok(None) => {
                    last_error = Some("main frame unavailable".to_string());
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                }
            }
            if attempt + 1 < MAIN_FRAME_RETRY_ATTEMPTS {
                sleep(MAIN_FRAME_RETRY_DELAY).await;
            }
        }
        Err(Error::JsEvaluation {
            reason: last_error.unwrap_or_else(|| "main frame unavailable".to_string()),
        })
    }
}

pub(super) async fn create_bowser_world(
    page: &ChromiumPage,
    frame_id: &FrameId,
) -> Result<ExecutionContextId> {
    cdp_trace::record_method("Page.createIsolatedWorld");
    page.execute(
        CreateIsolatedWorldParams::builder()
            .frame_id(frame_id.clone())
            .world_name(BOWSER_WORLD_NAME)
            .build()
            .map_err(|reason| Error::JsEvaluation { reason })?,
    )
    .await
    .map(|response| response.execution_context_id)
    .map_err(|err| Error::JsEvaluation {
        reason: err.to_string(),
    })
}

pub(super) async fn evaluate_bowser_world_value_for_page<T: for<'de> Deserialize<'de>>(
    page: &ChromiumPage,
    frame_id: &FrameId,
    expression: String,
) -> Result<T> {
    let context_id = create_bowser_world(page, frame_id).await?;
    cdp_trace::record_method("Runtime.evaluate");
    let params = EvaluateParams::builder()
        .expression(expression)
        .context_id(context_id)
        .await_promise(true)
        .return_by_value(true)
        .build()
        .map_err(|reason| Error::JsEvaluation { reason })?;
    let response = page
        .execute(params)
        .await
        .map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })?
        .result;
    if let Some(exception) = response.exception_details {
        return Err(Error::JsEvaluation {
            reason: format!("{exception:?}"),
        });
    }
    let value = response.result.value.unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value).map_err(|err| Error::JsEvaluation {
        reason: err.to_string(),
    })
}
