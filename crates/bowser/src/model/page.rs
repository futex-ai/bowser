//! Page capture model types.

use serde::{Deserialize, Deserializer};

use super::Element;

/// A captured page.
#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub struct PageCapture {
    /// Final URL of the captured page.
    pub url: String,
    /// Current document title.
    pub title: String,
    /// Synthetic body element ID for visible root expansion.
    #[serde(default)]
    pub body_id: Option<u32>,
    /// Synthetic body element ID for obscured root expansion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obscured_body_id: Option<u32>,
    /// Root content split by current user-visible state.
    pub content: PageContent,
}

impl<'de> Deserialize<'de> for PageCapture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PageCaptureWire::deserialize(deserializer)?;
        Ok(Self {
            url: wire.url,
            title: wire.title,
            body_id: wire.body_id,
            obscured_body_id: wire.obscured_body_id,
            content: wire
                .content
                .unwrap_or_else(|| PageContent::visible_only(wire.children.unwrap_or_default())),
        })
    }
}

#[derive(Deserialize)]
struct PageCaptureWire {
    url: String,
    title: String,
    #[serde(default)]
    body_id: Option<u32>,
    #[serde(default)]
    obscured_body_id: Option<u32>,
    #[serde(default)]
    content: Option<PageContent>,
    #[serde(default)]
    children: Option<Vec<Element>>,
}

/// Root page content split into visible and obscured buckets.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PageContent {
    /// Rendered content available to a user, including below-the-fold content.
    #[serde(default)]
    pub visible: Vec<Element>,
    /// Semantic content that is present but currently obscured or hidden.
    #[serde(default)]
    pub obscured: Vec<Element>,
}

impl PageContent {
    /// Builds root content from explicit visible and obscured buckets.
    pub fn new(visible: Vec<Element>, obscured: Vec<Element>) -> Self {
        Self { visible, obscured }
    }

    /// Builds root content with only visible entries.
    pub fn visible_only(visible: Vec<Element>) -> Self {
        Self {
            visible,
            obscured: Vec::new(),
        }
    }

    /// Returns all root buckets in display order.
    pub fn buckets(&self) -> [&[Element]; 2] {
        [&self.visible, &self.obscured]
    }

    /// Returns all mutable root buckets in display order.
    pub fn buckets_mut(&mut self) -> [&mut Vec<Element>; 2] {
        [&mut self.visible, &mut self.obscured]
    }
}

/// Truncation information.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TruncationInfo {
    /// Number of entries included in the preview.
    pub shown: usize,
    /// Total number of entries available in the full capture.
    pub total: usize,
}
