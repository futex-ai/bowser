//! Ephemeral favicon inventory model types.

use std::fmt;

/// A normalized browser tab favicon.
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BrowserFavicon {
    /// A canonical padded standard-base64 32 by 32 RGBA PNG.
    pub png_base64: String,
}

impl fmt::Debug for BrowserFavicon {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserFavicon")
            .field("png_base64", &"[redacted]")
            .finish()
    }
}

/// Current information about one live top-level browser page.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct LiveSessionPage {
    /// Stable Bowser page identifier.
    pub page_id: String,
    /// Current document URL.
    pub url: String,
    /// Current document title when it is non-empty.
    pub title: Option<String>,
    /// Whether the page is selected for subsequent Bowser commands.
    pub selected: bool,
    /// Best favicon resolved within the bounded inventory request, if any.
    pub favicon: Option<BrowserFavicon>,
}

#[cfg(test)]
#[path = "_tests_/favicon_tests.rs"]
mod favicon_tests;
