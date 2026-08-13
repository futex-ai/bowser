//! `bowser describe`.

use bowser::{Browser, BrowserEngine};

use crate::DescribeArgs;
use crate::commands::render_image_description;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};

/// Describes an image element from the active session page.
pub async fn run(
    config: &bowser::BrowserConfig,
    args: DescribeArgs,
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
    let description = page.describe(args.element_id).await;
    let detach = browser.detach().await;
    let description = description?;
    detach?;
    let output = render_image_description(&description, args.format)?;
    if let Some(path) = args.output.as_deref() {
        return Ok(CommandOutput::result(serde_json::json!({
            "output": { "path": path, "format": format_name(args.format) }
        }))?
        .text_file(path.to_path_buf(), output));
    }
    Ok(
        CommandOutput::result(serde_json::json!({ "description": description }))?
            .human_stdout(output),
    )
}

fn format_name(format: crate::StructuredFormat) -> &'static str {
    match format {
        crate::StructuredFormat::Yaml => "yaml",
        crate::StructuredFormat::Json => "json",
    }
}
