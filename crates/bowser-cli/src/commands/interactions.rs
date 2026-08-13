//! Browser interaction subcommands.

use std::path::PathBuf;

use bowser::{Browser, BrowserEngine, PageEngine, ScrollTarget};

use crate::error::{CliError, Result};
use crate::{ElementActionArgs, KeyArgs, PageFormat, ScrollArgs, TypeArgs};

use super::page_output;
use crate::output::{CommandContext, CommandOutput};

/// Clicks an element and prints an updated capture.
pub async fn click(
    config: &bowser::BrowserConfig,
    args: ElementActionArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    execute(
        config,
        Interaction::Click(args.element_id),
        args.page_id,
        args.output,
        args.format,
        context,
    )
    .await
}

/// Types into an element and prints an updated capture.
pub async fn type_text(
    config: &bowser::BrowserConfig,
    args: TypeArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    execute(
        config,
        Interaction::TypeText {
            element_id: args.element_id,
            text: args.text,
        },
        args.page_id,
        args.output,
        args.format,
        context,
    )
    .await
}

/// Clears an element and prints an updated capture.
pub async fn clear(
    config: &bowser::BrowserConfig,
    args: ElementActionArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    execute(
        config,
        Interaction::Clear(args.element_id),
        args.page_id,
        args.output,
        args.format,
        context,
    )
    .await
}

/// Submits an element and prints an updated capture.
pub async fn submit(
    config: &bowser::BrowserConfig,
    args: ElementActionArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    execute(
        config,
        Interaction::Submit(args.element_id),
        args.page_id,
        args.output,
        args.format,
        context,
    )
    .await
}

/// Sends one key press and prints an updated capture.
pub async fn key(
    config: &bowser::BrowserConfig,
    args: KeyArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    execute(
        config,
        Interaction::Key(args.key),
        args.page_id,
        args.output,
        args.format,
        context,
    )
    .await
}

/// Scrolls a page or element and prints an updated capture.
pub async fn scroll(
    config: &bowser::BrowserConfig,
    args: ScrollArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    let target = scroll_target(&args)?;
    execute(
        config,
        Interaction::Scroll(target),
        args.page_id,
        args.output,
        args.format,
        context,
    )
    .await
}

async fn launch_session(config: &bowser::BrowserConfig) -> Result<Box<dyn BrowserEngine>> {
    require_session(config)?;
    Ok(Box::new(Browser::launch(config.clone()).await?))
}

async fn selected_page(
    browser: &dyn BrowserEngine,
    page_id: Option<&str>,
) -> Result<Box<dyn PageEngine>> {
    match page_id {
        Some(page_id) => Ok(browser.select_page(page_id).await?),
        None => Ok(browser.current_page().await?),
    }
}

async fn execute(
    config: &bowser::BrowserConfig,
    interaction: Interaction,
    page_id: Option<String>,
    output: Option<PathBuf>,
    format: PageFormat,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    let browser = launch_session(config).await?;
    context.update_session(&browser.session_info().await?);
    let operation_result = perform(
        browser.as_ref(),
        &interaction,
        page_id.as_deref(),
        format,
        output.as_deref(),
    )
    .await;
    let detach_result = browser.detach().await;
    let rendered = operation_result?;
    context.update_session(&browser.session_info().await?);
    detach_result?;
    Ok(rendered)
}

async fn perform(
    browser: &dyn BrowserEngine,
    interaction: &Interaction,
    page_id: Option<&str>,
    format: PageFormat,
    output: Option<&std::path::Path>,
) -> Result<CommandOutput> {
    let page = selected_page(browser, page_id).await?;
    match interaction {
        Interaction::Click(element_id) => page.click(*element_id).await?,
        Interaction::TypeText { element_id, text } => {
            page.type_text(*element_id, text).await?;
        }
        Interaction::Clear(element_id) => page.clear(*element_id).await?,
        Interaction::Submit(element_id) => page.submit(*element_id).await?,
        Interaction::Key(key) => page.press_keys(std::slice::from_ref(key)).await?,
        Interaction::Scroll(target) => page.scroll(target.clone()).await?,
    }
    page_output(page.as_ref(), format, output).await
}

enum Interaction {
    Click(u32),
    TypeText { element_id: u32, text: String },
    Clear(u32),
    Submit(u32),
    Key(String),
    Scroll(ScrollTarget),
}

fn scroll_target(args: &ScrollArgs) -> Result<ScrollTarget> {
    match (args.direction.as_deref(), args.element_id) {
        (_, Some(element_id)) => Ok(ScrollTarget::ToElement(element_id)),
        (Some("up"), None) => Ok(ScrollTarget::Up),
        (Some("down"), None) => Ok(ScrollTarget::Down),
        _ => Err(CliError::InvalidCommand {
            input: "scroll requires --direction up|down or --element-id".to_string(),
        }),
    }
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
