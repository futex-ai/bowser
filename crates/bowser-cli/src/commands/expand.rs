//! `bowser expand`.

use std::sync::Arc;

use bowser::{FileSessionStore, SessionStore, default_session_dir};

use crate::ExpandArgs;
use crate::commands::render_element;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};

/// Runs element expansion against stored session state.
pub async fn run(
    config: &bowser::BrowserConfig,
    args: ExpandArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    let store = store(config).await;
    let session_id = config
        .session
        .id
        .clone()
        .ok_or_else(|| bowser::Error::SessionNotFound {
            session_id: "<missing --session>".to_string(),
        })?;
    let metadata = store.load(&session_id).await?;
    context.session = Some(session_id);
    context.page = metadata.selected_page_id.clone();
    let capture = metadata
        .selected_page()
        .and_then(|page| page.full_capture.clone().or(page.preview_capture.clone()))
        .ok_or(bowser::Error::ExpandNotFound {
            element_id: args.element_id,
        })?;
    let element =
        bowser::expand_element(&capture, args.element_id).ok_or(bowser::Error::ExpandNotFound {
            element_id: args.element_id,
        })?;
    let output = render_element(&element, args.format)?;
    if let Some(path) = args.output.as_deref() {
        return Ok(CommandOutput::result(serde_json::json!({
            "output": { "path": path, "format": structured_format_name(args.format) }
        }))?
        .text_file(path.to_path_buf(), output));
    }
    Ok(CommandOutput::result(serde_json::json!({ "element": element }))?.human_stdout(output))
}

fn structured_format_name(format: crate::StructuredFormat) -> &'static str {
    match format {
        crate::StructuredFormat::Yaml => "yaml",
        crate::StructuredFormat::Json => "json",
    }
}

async fn store(config: &bowser::BrowserConfig) -> Arc<dyn SessionStore> {
    Arc::new(FileSessionStore::new(
        config
            .session
            .dir
            .clone()
            .unwrap_or_else(default_session_dir),
    ))
}
