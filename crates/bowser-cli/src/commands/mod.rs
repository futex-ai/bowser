//! Command helpers.

pub mod capabilities;
pub mod capture;
pub mod describe;
pub mod download;
pub mod expand;
pub mod get;
pub mod history;
pub mod interactions;
pub mod interactive;
pub mod meta;
pub mod page;
pub mod pointer_log;
mod repl;
mod repl_help;
pub mod session;

use std::path::Path;

use crate::error::{CliError, Result};
use crate::output::CommandOutput;
use crate::{PageFormat, StructuredFormat};

/// Renders a page and returns either embedded machine data or a file reference.
pub async fn page_output(
    page: &dyn bowser::PageEngine,
    format: PageFormat,
    path: Option<&Path>,
) -> Result<CommandOutput> {
    let (human, embedded) = match format {
        PageFormat::Html => {
            let html = page.rendered_html().await?;
            (html.clone(), serde_json::json!({ "html": html }))
        }
        PageFormat::Yaml => {
            let capture = page.capture().await?;
            (
                render_capture(&capture, StructuredFormat::Yaml)?,
                serde_json::json!({ "capture": capture }),
            )
        }
        PageFormat::Json => {
            let capture = page.capture().await?;
            (
                render_capture(&capture, StructuredFormat::Json)?,
                serde_json::json!({ "capture": capture }),
            )
        }
    };
    if let Some(path) = path {
        return Ok(CommandOutput::result(serde_json::json!({
            "output": {
                "path": path,
                "format": format_name(format),
            }
        }))?
        .text_file(path.to_path_buf(), human));
    }
    Ok(CommandOutput::result(embedded)?.human_stdout(human))
}

fn format_name(format: PageFormat) -> &'static str {
    match format {
        PageFormat::Yaml => "yaml",
        PageFormat::Json => "json",
        PageFormat::Html => "html",
    }
}

/// Renders a page capture.
pub fn render_capture(capture: &bowser::PageCapture, format: StructuredFormat) -> Result<String> {
    match format {
        StructuredFormat::Yaml => Ok(bowser::to_yaml(capture)?),
        StructuredFormat::Json => {
            serde_json::to_string_pretty(capture).map_err(|err| CliError::Readline {
                reason: err.to_string(),
            })
        }
    }
}

/// Renders a single element.
pub fn render_element(element: &bowser::Element, format: StructuredFormat) -> Result<String> {
    match format {
        StructuredFormat::Yaml => Ok(bowser::to_yaml_element(element)?),
        StructuredFormat::Json => {
            serde_json::to_string_pretty(element).map_err(|err| CliError::Readline {
                reason: err.to_string(),
            })
        }
    }
}

/// Renders a metadata record.
pub fn render_metadata(
    record: &bowser::MetadataRecord,
    format: StructuredFormat,
) -> Result<String> {
    match format {
        StructuredFormat::Yaml => Ok(bowser::metadata_to_yaml(record)?),
        StructuredFormat::Json => {
            serde_json::to_string_pretty(record).map_err(|err| CliError::Readline {
                reason: err.to_string(),
            })
        }
    }
}

/// Renders an image description.
pub fn render_image_description(
    description: &bowser::ImageDescription,
    format: StructuredFormat,
) -> Result<String> {
    match format {
        StructuredFormat::Yaml => Ok(bowser::image_description_to_yaml(description)?),
        StructuredFormat::Json => {
            serde_json::to_string_pretty(description).map_err(|err| CliError::Readline {
                reason: err.to_string(),
            })
        }
    }
}
