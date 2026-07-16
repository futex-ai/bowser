//! Top-level dispatch tests.

use std::error::Error;

use tempfile::tempdir;

use super::load_browser_config;

#[test]
fn config_load_errors_preserve_the_library_cause() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("invalid.yaml");
    std::fs::write(&path, "chrome: [").expect("write invalid config");
    let library_error = bowser::load_config(Some(&path), bowser::ConfigOverrides::default())
        .expect_err("library config failure");

    let cli_error = load_browser_config(Some(path), bowser::ConfigOverrides::default())
        .expect_err("CLI config failure");

    assert!(cli_error.source().is_some());
    assert!(cli_error.to_string().contains(&library_error.to_string()));
}
