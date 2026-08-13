//! `bowser capture`.

use bowser::{Browser, BrowserEngine};

use crate::CaptureArgs;
use crate::commands::page_output;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};

/// Captures the selected or explicit page from a detached session.
pub async fn run(
    config: &bowser::BrowserConfig,
    args: CaptureArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    require_session(config)?;
    let browser = Browser::launch(config.clone()).await?;
    context.update_session(&browser.session_info().await?);
    let operation_result = async {
        let page = match args.page_id.as_deref() {
            Some(page_id) => browser.select_page(page_id).await?,
            None => browser.current_page().await?,
        };
        let info = browser.session_info().await?;
        context.update_session(&info);
        page_output(page.as_ref(), args.format, args.output.as_deref()).await
    }
    .await;
    let detach_result = browser.detach().await;
    let output = operation_result?;
    detach_result?;
    Ok(output)
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
