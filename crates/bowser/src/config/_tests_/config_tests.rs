use tempfile::tempdir;

use super::{AiProvider, BrowserConfig, ConfigOverrides, load_config, parse_viewport};
use crate::model::Viewport;

#[test]
fn parses_viewport() {
    let viewport = parse_viewport("1280x720").expect("viewport");
    assert_eq!(viewport.width, 1280);
    assert_eq!(viewport.height, 720);
}

#[test]
fn rejects_zero_viewport_dimensions() {
    assert!(parse_viewport("0x720").is_err());
    assert!(parse_viewport("1280x0").is_err());
}

#[test]
fn rejects_missing_explicit_config_file() {
    let config_dir = tempdir().expect("config dir");
    let config_path = config_dir.path().join("missing.yaml");

    let error = load_config(Some(&config_path), ConfigOverrides::default())
        .expect_err("an explicitly requested config file must exist");

    assert!(matches!(
        error,
        crate::error::Error::ConfigFileNotFound { path } if path == config_path
    ));
}

#[test]
fn rejects_zero_viewport_from_final_config_overrides() {
    let config_dir = tempdir().expect("config dir");
    let config_path = config_dir.path().join("missing.yaml");
    std::fs::write(&config_path, "{}\n").expect("empty config");
    let error = load_config(
        Some(&config_path),
        ConfigOverrides {
            viewport: Some(Viewport {
                width: 1280,
                height: 0,
            }),
            ..ConfigOverrides::default()
        },
    )
    .expect_err("zero viewport height");

    assert!(matches!(error, crate::error::Error::Config { .. }));
}

#[test]
fn rejects_chrome_arguments_reserved_for_session_identity() {
    for argument in [
        "--remote-debugging-port=9222",
        "--remote-debugging-port",
        "--user-data-dir=/tmp/not-bowser-owned",
        "--user-data-dir",
    ] {
        let error = load_config(
            None,
            ConfigOverrides {
                chrome_args: Some(vec![argument.to_string()]),
                ..ConfigOverrides::default()
            },
        )
        .expect_err("reserved Chrome argument");
        assert!(matches!(error, crate::error::Error::Config { .. }));
    }
}

#[test]
fn applies_overrides() {
    let config = load_config(
        None,
        ConfigOverrides {
            timeout_secs: Some(99),
            include_hidden: Some(true),
            headless: Some(false),
            persistent_profile: Some(true),
            ai_provider: Some(AiProvider::Ollama),
            ..ConfigOverrides::default()
        },
    )
    .expect("config");
    assert_eq!(config.timeout.as_secs(), 99);
    assert!(config.output.include_hidden);
    assert!(!config.headless);
    assert!(config.persistent_profile);
    assert_eq!(config.ai.provider, AiProvider::Ollama);
    assert_eq!(BrowserConfig::default().output.max_children, 50);
}
