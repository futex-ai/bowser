//! Capture rendering helpers for interactive mode.

use std::sync::{Arc, Mutex};

use bowser::{PageCapture, PageEngine};

use crate::commands::repl::{HelperState, update_capture_state};
use crate::error::Result;

use super::preview::capture_with_stored_fallback;

const PAGE_MANAGEMENT_CAPTURE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) async fn capture_and_print(
    page: &dyn PageEngine,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    let capture = page.capture().await?;
    update_capture_state(helper_state, &capture);
    print_full_capture(&capture)
}

pub(super) async fn capture_page_management_and_print(
    page: &dyn PageEngine,
    config: &bowser::BrowserConfig,
    session_id: &str,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    let capture = capture_with_stored_fallback(
        Box::pin(page.capture()),
        PAGE_MANAGEMENT_CAPTURE_TIMEOUT,
        config,
        session_id,
    )
    .await?;
    update_capture_state(helper_state, &capture);
    print_full_capture(&capture)
}

pub(super) fn print_full_capture(capture: &PageCapture) -> Result<()> {
    println!("---");
    println!("{}", bowser::to_yaml(capture)?);
    Ok(())
}
