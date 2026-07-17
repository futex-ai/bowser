//! Configuration loading and precedence handling.

mod defaults;
mod file;
mod overrides;
mod types;
mod validation;

pub use defaults::default_config_path;
pub use file::load_config;
pub use types::{
    AiConfig, AiProvider, BrowserConfig, ConfigOverrides, OutputConfig, SessionConfig,
};
pub(crate) use validation::validate_config;

#[cfg(any(test, doctest))]
pub(crate) use validation::parse_viewport;

#[cfg(test)]
#[path = "_tests_/config_tests.rs"]
mod config_tests;
