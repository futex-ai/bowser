//! Image-model helpers and describe output.

use url::Url;

/// On-demand description details for a captured image.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ImageDescription {
    pub element_id: u32,
    pub alt: String,
    pub src: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    pub description: String,
}

impl ImageDescription {
    /// Builds a new image-description payload.
    pub fn new(element_id: u32, alt: String, src: String, description: String) -> Self {
        Self {
            element_id,
            filename: image_filename(&src),
            alt,
            src,
            description,
        }
    }
}

/// Returns a compact display label for an image.
pub(crate) fn image_label(alt: &str, src: &str, description: Option<&str>) -> String {
    match (alt.is_empty(), image_filename(src)) {
        (false, Some(filename)) => format!("{alt} ({filename})"),
        (false, None) => alt.to_string(),
        (true, Some(filename)) => filename,
        (true, None) => description.unwrap_or("[image]").to_string(),
    }
}

/// Returns the filename portion of an image source, when one exists.
pub fn image_filename(src: &str) -> Option<String> {
    if src.is_empty() || src.starts_with("data:") || src.starts_with("blob:") {
        return None;
    }
    if let Ok(url) = Url::parse(src) {
        return filename_from_path(url.path());
    }
    let path = src
        .split_once('#')
        .map_or(src, |(prefix, _)| prefix)
        .split_once('?')
        .map_or(src, |(prefix, _)| prefix);
    filename_from_path(path)
}

fn filename_from_path(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    let filename = trimmed.rsplit('/').next()?;
    (!filename.is_empty()).then(|| filename.to_string())
}
