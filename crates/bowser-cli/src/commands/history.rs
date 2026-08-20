//! One-shot browser history commands.

use bowser::{Browser, BrowserEngine, PageEngine};

use crate::{
    HistoryArgs,
    commands::page_output,
    error::Result,
    output::{CommandContext, CommandOutput},
};

#[derive(Clone, Copy)]
enum HistoryAction {
    Back,
    Forward,
    Reload,
}

/// Goes to the selected page's previous history entry and prints a capture.
pub async fn back(
    config: &bowser::BrowserConfig,
    args: HistoryArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    run(config, args, context, HistoryAction::Back).await
}

/// Goes to the selected page's next history entry and prints a capture.
pub async fn forward(
    config: &bowser::BrowserConfig,
    args: HistoryArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    run(config, args, context, HistoryAction::Forward).await
}

/// Reloads the selected page and prints a capture.
pub async fn reload(
    config: &bowser::BrowserConfig,
    args: HistoryArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    run(config, args, context, HistoryAction::Reload).await
}

async fn run(
    config: &bowser::BrowserConfig,
    args: HistoryArgs,
    context: &mut CommandContext,
    action: HistoryAction,
) -> Result<CommandOutput> {
    require_session(config)?;
    let browser = Browser::launch(config.clone()).await?;
    context.update_session(&browser.session_info().await?);
    let operation_result = perform(&browser, &args, action).await;
    let detach_result = browser.detach().await;
    let output = operation_result?;
    context.update_session(&browser.session_info().await?);
    detach_result?;
    Ok(output)
}

async fn perform(
    browser: &dyn BrowserEngine,
    args: &HistoryArgs,
    action: HistoryAction,
) -> Result<CommandOutput> {
    let page = match args.page_id.as_deref() {
        Some(page_id) => browser.select_page(page_id).await?,
        None => browser.current_page().await?,
    };
    apply(page.as_ref(), action).await?;
    page_output(page.as_ref(), args.format, args.output.as_deref()).await
}

async fn apply(page: &dyn PageEngine, action: HistoryAction) -> Result<()> {
    match action {
        HistoryAction::Back => page.back().await?,
        HistoryAction::Forward => page.forward().await?,
        HistoryAction::Reload => page.reload().await?,
    }
    Ok(())
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
