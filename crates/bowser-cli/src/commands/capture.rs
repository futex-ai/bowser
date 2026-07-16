//! `bowser capture`.

use bowser::{Browser, BrowserEngine};

use crate::CaptureArgs;
use crate::commands::{render_page, write_output};
use crate::error::Result;

/// Captures the selected or explicit page from a detached session.
pub async fn run(config: &bowser::BrowserConfig, args: CaptureArgs) -> Result<()> {
    require_session(config)?;
    let browser = Browser::launch(config.clone()).await?;
    let operation_result = async {
        let page = match args.page_id.as_deref() {
            Some(page_id) => browser.select_page(page_id).await?,
            None => browser.current_page().await?,
        };
        render_page(page.as_ref(), args.format).await
    }
    .await;
    let detach_result = browser.detach().await;
    let output = operation_result?;
    detach_result?;
    write_output(args.output.as_deref(), &output).await
}

fn require_session(config: &bowser::BrowserConfig) -> Result<()> {
    if config.session.id.is_none() {
        return Err(bowser::Error::SessionNotFound {
            session_id: "<missing --session>".to_string(),
        }
        .into());
    }
    Ok(())
}
