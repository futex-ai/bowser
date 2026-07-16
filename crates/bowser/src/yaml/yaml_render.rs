//! YAML rendering entrypoints.

use serde_yaml::{Mapping, Value};

use super::element_render::{render_children, render_element};
use crate::error::{Error, Result};
use crate::model::{Element, ImageDescription, MetadataRecord, PageCapture};

/// Serializes a page capture to compact YAML.
pub fn to_yaml(capture: &PageCapture) -> Result<String> {
    let mut root = Mapping::new();
    root.insert(Value::from("url"), Value::from(capture.url.clone()));
    root.insert(Value::from("title"), Value::from(capture.title.clone()));
    let mut content = Mapping::new();
    content.insert(
        Value::from("visible"),
        Value::Sequence(render_children(&capture.content.visible)),
    );
    content.insert(
        Value::from("obscured"),
        Value::Sequence(render_children(&capture.content.obscured)),
    );
    root.insert(Value::from("content"), Value::Mapping(content));
    serde_yaml::to_string(&Value::Mapping(root)).map_err(|err| Error::Config {
        reason: format!("failed to serialize YAML: {err}"),
    })
}

/// Serializes a single element subtree to compact YAML.
pub fn to_yaml_element(element: &Element) -> Result<String> {
    let value = render_element(element);
    serde_yaml::to_string(&value).map_err(|err| Error::Config {
        reason: format!("failed to serialize YAML element: {err}"),
    })
}

/// Serializes metadata to YAML.
pub fn metadata_to_yaml(record: &MetadataRecord) -> Result<String> {
    serde_yaml::to_string(record).map_err(|err| Error::Config {
        reason: format!("failed to serialize metadata YAML: {err}"),
    })
}

/// Serializes an image description to YAML.
pub fn image_description_to_yaml(description: &ImageDescription) -> Result<String> {
    serde_yaml::to_string(description).map_err(|err| Error::Config {
        reason: format!("failed to serialize image description YAML: {err}"),
    })
}
