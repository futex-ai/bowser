//! Mouse movement data types.

/// A viewport-relative cursor position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

/// Viewport bounds used to generate cursor paths.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ViewportArea {
    pub width: f64,
    pub height: f64,
}

/// A single cursor move plus the pause that follows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MouseMoveStep {
    pub position: CursorPosition,
    pub pause_ms: u64,
}
