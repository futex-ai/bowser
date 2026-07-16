//! Shared geometry and viewport types.

/// Viewport dimensions.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

/// Current user-visible state for a metadata target.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ElementVisibility {
    pub present: bool,
    pub in_viewport: bool,
    pub obscured: bool,
    pub enabled: bool,
    pub visible: bool,
    pub clickable: bool,
}

/// Current page-space bounds for a metadata target.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ElementBounds {
    pub top_left: (f64, f64),
    pub bottom_right: (f64, f64),
}

/// Scroll target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrollTarget {
    Down,
    Up,
    ToElement(u32),
}
