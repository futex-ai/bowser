//! `bowser page`.

use bowser::{Browser, BrowserEngine, SessionPageSummary};

use crate::PageSubcommand;
use crate::commands::render_page;
use crate::error::Result;
use crate::url::normalize_navigation_target;

/// Runs page management commands against a detached session.
pub async fn run(config: bowser::BrowserConfig, command: PageSubcommand) -> Result<()> {
    require_session(&config)?;
    let browser = Browser::launch(config).await?;
    let operation_result = execute(&browser, command).await;
    let detach_result = browser.detach().await;
    let output = operation_result?;
    detach_result?;
    print!("{output}");
    Ok(())
}

async fn execute(browser: &dyn BrowserEngine, command: PageSubcommand) -> Result<String> {
    match command {
        PageSubcommand::List => Ok(render_page_summaries(&browser.list_pages().await?)),
        PageSubcommand::Select { page_id, format } => {
            let page = browser.select_page(&page_id).await?;
            render_page(page.as_ref(), format).await
        }
        PageSubcommand::New { url, format } => {
            let url = url.map(|url| normalize_navigation_target(&url));
            let page = browser.new_page(url.as_deref()).await?;
            render_page(page.as_ref(), format).await
        }
        PageSubcommand::Close { page_id, format } => {
            let page = browser.close_page(page_id.as_deref()).await?;
            render_page(page.as_ref(), format).await
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
