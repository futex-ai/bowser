//! `bowser meta`.

use bowser::{Browser, BrowserEngine};

use crate::MetaArgs;
use crate::commands::render_metadata;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};

/// Fetches metadata for an element from the active session page.
pub async fn run(
    config: &bowser::BrowserConfig,
    args: MetaArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    if config.session.id.is_none() {
        return Err(bowser::Error::SessionNotFound {
            session_id: "<missing --session>".to_string(),
        }
        .into());
    }
    let browser = Browser::launch(config.clone()).await?;
    context.update_session(&browser.session_info().await?);
    let page = browser.current_page().await?;
    let record = page.metadata(args.element_id).await;
    let detach = browser.detach().await;
    let record = record?;
    detach?;
    let output = render_metadata(&record, args.format)?;
    if let Some(path) = args.output.as_deref() {
        return Ok(CommandOutput::result(serde_json::json!({
            "output": { "path": path, "format": format_name(args.format) }
        }))?
        .text_file(path.to_path_buf(), output));
    }
    Ok(CommandOutput::result(serde_json::json!({ "metadata": record }))?.human_stdout(output))
}

fn format_name(format: crate::StructuredFormat) -> &'static str {
    match format {
        crate::StructuredFormat::Yaml => "yaml",
        crate::StructuredFormat::Json => "json",
    }
}
