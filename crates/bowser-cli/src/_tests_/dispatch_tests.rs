//! Top-level dispatch tests.

use std::error::Error;

use tempfile::tempdir;

use super::load_browser_config;
use crate::error::CliError;

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

#[test]
fn explicit_missing_config_preserves_the_requested_path() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.yaml");

    let error = load_browser_config(Some(path.clone()), bowser::ConfigOverrides::default())
        .expect_err("missing explicit config");

    match error {
        CliError::ConfigLoad {
            path: selected_path,
            source: bowser::Error::ConfigFileNotFound { path: source_path },
        } => {
            assert_eq!(selected_path, path.display().to_string());
            assert_eq!(source_path, path);
        }
        other => panic!("unexpected config error: {other:?}"),
    }
}
