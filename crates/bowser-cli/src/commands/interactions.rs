//! Browser interaction subcommands.

use bowser::{Browser, BrowserEngine, PageEngine, ScrollTarget};

use crate::error::{CliError, Result};
use crate::{ElementActionArgs, KeyArgs, ScrollArgs, TypeArgs};

use super::{render_page, write_output};

/// Clicks an element and prints an updated capture.
pub async fn click(config: &bowser::BrowserConfig, args: ElementActionArgs) -> Result<()> {
    let browser = launch_session(config).await?;
    let page = selected_page(browser.as_ref(), args.page_id.as_deref()).await?;
    page.click(args.element_id).await?;
    finish(browser, page, args.output.as_deref(), args.format).await
}

/// Types into an element and prints an updated capture.
pub async fn type_text(config: &bowser::BrowserConfig, args: TypeArgs) -> Result<()> {
    let browser = launch_session(config).await?;
    let page = selected_page(browser.as_ref(), args.page_id.as_deref()).await?;
    page.type_text(args.element_id, &args.text).await?;
    finish(browser, page, args.output.as_deref(), args.format).await
}

/// Clears an element and prints an updated capture.
pub async fn clear(config: &bowser::BrowserConfig, args: ElementActionArgs) -> Result<()> {
    let browser = launch_session(config).await?;
    let page = selected_page(browser.as_ref(), args.page_id.as_deref()).await?;
    page.clear(args.element_id).await?;
    finish(browser, page, args.output.as_deref(), args.format).await
}

/// Submits an element and prints an updated capture.
pub async fn submit(config: &bowser::BrowserConfig, args: ElementActionArgs) -> Result<()> {
    let browser = launch_session(config).await?;
    let page = selected_page(browser.as_ref(), args.page_id.as_deref()).await?;
    page.submit(args.element_id).await?;
    finish(browser, page, args.output.as_deref(), args.format).await
}

/// Sends one key press and prints an updated capture.
pub async fn key(config: &bowser::BrowserConfig, args: KeyArgs) -> Result<()> {
    let browser = launch_session(config).await?;
    let page = selected_page(browser.as_ref(), args.page_id.as_deref()).await?;
    page.press_keys(std::slice::from_ref(&args.key)).await?;
    finish(browser, page, args.output.as_deref(), args.format).await
}

/// Scrolls a page or element and prints an updated capture.
pub async fn scroll(config: &bowser::BrowserConfig, args: ScrollArgs) -> Result<()> {
    let browser = launch_session(config).await?;
    let page = selected_page(browser.as_ref(), args.page_id.as_deref()).await?;
    page.scroll(scroll_target(&args)?).await?;
    finish(browser, page, args.output.as_deref(), args.format).await
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

async fn finish(
    browser: Box<dyn BrowserEngine>,
    page: Box<dyn PageEngine>,
    output: Option<&std::path::Path>,
    format: crate::PageFormat,
) -> Result<()> {
    let rendered = render_page(page.as_ref(), format).await?;
    write_output(output, &rendered).await?;
    browser.detach().await?;
    Ok(())
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
