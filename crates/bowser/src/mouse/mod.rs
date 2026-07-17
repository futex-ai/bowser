//! Mouse movement helpers for realistic pointer clicks.

mod easing;
mod geometry;
mod path;
mod seed;
mod timing;
mod types;

pub(crate) use path::{initial_cursor_position, movement_steps};
pub(crate) use timing::click_hold_ms;
pub(crate) use types::{CursorPosition, ViewportArea};

#[cfg(test)]
#[path = "_tests_/mouse_tests.rs"]
mod mouse_tests;
