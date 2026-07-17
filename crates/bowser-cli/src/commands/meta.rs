//! `bowser meta`.

use bowser::{Browser, BrowserEngine};

use crate::MetaArgs;
use crate::commands::{render_metadata, write_output};
use crate::error::Result;

/// Fetches metadata for an element from the active session page.
pub async fn run(config: &bowser::BrowserConfig, args: MetaArgs) -> Result<()> {
    if config.session.id.is_none() {
        return Err(bowser::Error::SessionNotFound {
            session_id: "<missing --session>".to_string(),
        }
        .into());
    }
    let browser = Browser::launch(config.clone()).await?;
    let page = browser.current_page().await?;
    let record = page.metadata(args.element_id).await;
    let detach = browser.detach().await;
    let record = record?;
    detach?;
    let output = render_metadata(&record, args.format)?;
    write_output(args.output.as_deref(), &output).await?;
    Ok(())
}
