//! `bowser describe`.

use bowser::{Browser, BrowserEngine};

use crate::DescribeArgs;
use crate::commands::{render_image_description, write_output};
use crate::error::Result;

/// Describes an image element from the active session page.
pub async fn run(config: &bowser::BrowserConfig, args: DescribeArgs) -> Result<()> {
    if config.session.id.is_none() {
        return Err(bowser::Error::SessionNotFound {
            session_id: "<missing --session>".to_string(),
        }
        .into());
    }
    let browser = Browser::launch(config.clone()).await?;
    let page = browser.current_page().await?;
    let description = page.describe(args.element_id).await;
    let detach = browser.detach().await;
    let description = description?;
    detach?;
    let output = render_image_description(&description, args.format)?;
    write_output(args.output.as_deref(), &output).await?;
    Ok(())
}
