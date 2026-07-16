//! Stored-preview fallback helpers for interactive mode.

use std::future::Future;
use std::pin::Pin;

use bowser::{FileSessionStore, PageCapture, SessionStore, default_session_dir};

use crate::error::{CliError, Result};

pub(crate) async fn capture_with_stored_fallback(
    live_capture: Pin<Box<dyn Future<Output = bowser::Result<PageCapture>> + Send + '_>>,
    timeout: std::time::Duration,
    config: &bowser::BrowserConfig,
    session_id: &str,
) -> Result<PageCapture> {
    match tokio::time::timeout(timeout, live_capture).await {
        Ok(Ok(capture)) => Ok(capture),
        Ok(Err(err)) if can_use_stored_page_preview_fallback(&err) => {
            stored_selected_preview_or_error(config, session_id, err.into()).await
        }
        Ok(Err(err)) => Err(err.into()),
        Err(_) => {
            stored_selected_preview_or_error(
                config,
                session_id,
                bowser::Error::Timeout {
                    seconds: timeout.as_secs(),
                }
                .into(),
            )
            .await
        }
    }
}

pub(super) async fn stored_selected_preview(
    config: &bowser::BrowserConfig,
    session_id: &str,
) -> Result<Option<PageCapture>> {
    let store: std::sync::Arc<dyn SessionStore> = std::sync::Arc::new(FileSessionStore::new(
        config
            .session
            .dir
            .clone()
            .unwrap_or_else(default_session_dir),
    ));
    let metadata = store.load(session_id).await?;
    Ok(metadata
        .selected_page()
        .or_else(|| metadata.summary_page())
        .and_then(|page| page.preview_capture.clone()))
}

async fn stored_selected_preview_or_error(
    config: &bowser::BrowserConfig,
    session_id: &str,
    error: CliError,
) -> Result<PageCapture> {
    if let Some(capture) = stored_selected_preview(config, session_id).await? {
        return Ok(capture);
    }
    Err(error)
}

fn can_use_stored_page_preview_fallback(error: &bowser::Error) -> bool {
    matches!(
        error,
        bowser::Error::BrowserDisconnected
            | bowser::Error::PageTargetSessionInvalid { .. }
            | bowser::Error::Timeout { .. }
            | bowser::Error::StealthInjection { .. }
            | bowser::Error::CaptureScript { .. }
            | bowser::Error::JsEvaluation { .. }
            | bowser::Error::Cdp { .. }
    )
}
