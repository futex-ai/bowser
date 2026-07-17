//! Command helpers.

pub mod capture;
pub mod describe;
pub mod download;
pub mod expand;
pub mod get;
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
use crate::{PageFormat, StructuredFormat};

/// Writes output to stdout or a file.
pub async fn write_output(path: Option<&Path>, content: &str) -> Result<()> {
    if let Some(path) = path {
        tokio::fs::write(path, content)
            .await
            .map_err(|_| CliError::OutputWrite {
                path: path.display().to_string(),
            })?;
    } else {
        print!("{content}");
    }
    Ok(())
}

/// Renders the current page in a page-level format.
pub async fn render_page(page: &dyn bowser::PageEngine, format: PageFormat) -> Result<String> {
    match format {
        PageFormat::Html => Ok(page.rendered_html().await?),
        PageFormat::Yaml => {
            let capture = page.capture().await?;
            render_capture(&capture, StructuredFormat::Yaml)
        }
        PageFormat::Json => {
            let capture = page.capture().await?;
            render_capture(&capture, StructuredFormat::Json)
        }
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
