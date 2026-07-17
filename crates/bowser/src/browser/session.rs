//! Page target-session activation and validation helpers.

use std::time::Duration;

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;

use crate::cdp_trace;
use crate::error::{Error, Result, is_target_session_missing};

const PAGE_ACTIVATION_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) async fn ensure_page_activated(page: &chromiumoxide::Page) -> Result<()> {
    let mut last_reason = None;
    for attempt in 0..3 {
        if let Err(failure) = activate_page(page).await
            && !failure.is_retryable()
        {
            return Err(Error::cdp(format!(
                "failed to activate page: {}",
                failure.reason()
            )));
        }
        match probe_page_session(page).await {
            Ok(_) => return Ok(()),
            Err(failure) => {
                if !failure.is_retryable() {
                    return Err(Error::PageTargetSessionInvalid {
                        reason: failure.reason(),
                    });
                }
                last_reason = Some(failure.reason());
            }
        }
        if attempt < 2 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    Err(Error::PageTargetSessionInvalid {
        reason: last_reason.unwrap_or_else(|| "unknown target session error".to_string()),
    })
}

enum PageActivationFailure {
    TargetSessionMissing(String),
    TimedOut {
        operation: &'static str,
        seconds: u64,
    },
    Other(String),
}

impl PageActivationFailure {
    fn reason(&self) -> String {
        match self {
            Self::TargetSessionMissing(reason) | Self::Other(reason) => reason.clone(),
            Self::TimedOut { operation, seconds } => {
                format!("{operation} timed out after {seconds}s")
            }
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::TargetSessionMissing(_) | Self::TimedOut { .. })
    }
}

async fn activate_page(
    page: &chromiumoxide::Page,
) -> std::result::Result<(), PageActivationFailure> {
    match tokio::time::timeout(PAGE_ACTIVATION_TIMEOUT, page.activate()).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(classify_page_activation_error(
            "activate page",
            err.to_string(),
        )),
        Err(_) => Err(PageActivationFailure::TimedOut {
            operation: "activate page",
            seconds: PAGE_ACTIVATION_TIMEOUT.as_secs(),
        }),
    }
}

async fn probe_page_session(
    page: &chromiumoxide::Page,
) -> std::result::Result<(), PageActivationFailure> {
    let mut params = EvaluateParams::new("1");
    params.await_promise = Some(true);
    params.return_by_value = Some(true);
    cdp_trace::record_method("Runtime.evaluate");
    match tokio::time::timeout(PAGE_ACTIVATION_TIMEOUT, page.execute(params)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(classify_page_activation_error(
            "probe page session",
            err.to_string(),
        )),
        Err(_) => Err(PageActivationFailure::TimedOut {
            operation: "probe page session",
            seconds: PAGE_ACTIVATION_TIMEOUT.as_secs(),
        }),
    }
}

fn classify_page_activation_error(
    operation: &'static str,
    reason: String,
) -> PageActivationFailure {
    if is_target_session_missing(&reason) {
        PageActivationFailure::TargetSessionMissing(reason)
    } else {
        PageActivationFailure::Other(format!("{operation}: {reason}"))
    }
}
