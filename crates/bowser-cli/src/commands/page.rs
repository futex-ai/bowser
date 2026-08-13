//! `bowser page`.

use bowser::{Browser, BrowserEngine, SessionPageSummary};

use crate::PageSubcommand;
use crate::commands::page_output;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};
use crate::url::normalize_navigation_target;

/// Runs page management commands against a detached session.
pub async fn run(
    config: bowser::BrowserConfig,
    command: PageSubcommand,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    require_session(&config)?;
    let browser = Browser::launch(config).await?;
    context.update_session(&browser.session_info().await?);
    let operation_result = execute(&browser, command).await;
    let detach_result = browser.detach().await;
    let output = operation_result?;
    context.update_session(&browser.session_info().await?);
    detach_result?;
    Ok(output)
}

async fn execute(browser: &dyn BrowserEngine, command: PageSubcommand) -> Result<CommandOutput> {
    match command {
        PageSubcommand::List => {
            let pages = browser.list_pages().await?;
            Ok(
                CommandOutput::result(serde_json::json!({ "pages": pages }))?
                    .human_stdout(render_page_summaries(&pages)),
            )
        }
        PageSubcommand::Select { page_id, format } => {
            let page = browser.select_page(&page_id).await?;
            page_output(page.as_ref(), format, None).await
        }
        PageSubcommand::New { url, format } => {
            let url = url.map(|url| normalize_navigation_target(&url));
            let page = browser.new_page(url.as_deref()).await?;
            page_output(page.as_ref(), format, None).await
        }
        PageSubcommand::Close { page_id, format } => {
            let page = browser.close_page(page_id.as_deref()).await?;
            page_output(page.as_ref(), format, None).await
        }
    }
}

/// Renders page summaries for CLI and REPL output.
pub fn render_page_summaries(pages: &[SessionPageSummary]) -> String {
    if pages.is_empty() {
        return String::from("No pages.\n");
    }
    let mut output = String::new();
    for page in pages {
        let marker = if page.selected { "*" } else { " " };
        let status = if page.live { "live" } else { "stale" };
        let url = page.url.clone().unwrap_or_default();
        let title = page.title.clone().unwrap_or_default();
        output.push_str(&format!(
            "{marker} {id}\t{status}\t{url}\t{title}\n",
            id = page.id
        ));
    }
    output
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
