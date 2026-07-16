//! Browser lifecycle and session attachment.

mod display;
mod engine;
mod lifecycle;
mod page_selection;
mod page_state;
mod page_targets;
mod pages;
mod session;
mod state;

pub use engine::{Browser, BrowserEngine};

#[cfg(any(test, doctest))]
use lifecycle::{build_chrome_args, resolve_user_data_dir};

#[cfg(test)]
#[path = "_tests_/browser_tests.rs"]
mod browser_tests;
