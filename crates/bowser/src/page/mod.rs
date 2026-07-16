//! Page interaction and capture.

mod accessibility_capture;
mod capture;
mod describe;
mod download;
mod engine;
mod evaluate;
mod form_scripts;
mod frame_capture;
mod frame_capture_local;
mod frame_capture_remote;
mod frame_capture_support;
mod frame_input;
mod frame_metadata;
mod frame_runtime;
mod frame_screenshot;
mod frame_scroll;
mod iframe_input;
mod iframe_target;
mod input;
mod interaction_scripts;
mod isolated_world;
mod lifecycle;
mod low_runtime_input;
mod metadata;
mod navigation;
mod navigation_frames;
mod navigation_state;
mod page_state_strings;
mod preparation;
mod types;
mod viewport;

pub(crate) use self::types::LivePageSession;
pub use self::types::{LivePage, PageEngine};

#[cfg(test)]
#[path = "_tests_/page_tests.rs"]
mod page_tests;
