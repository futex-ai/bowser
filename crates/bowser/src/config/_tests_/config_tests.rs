use super::{AiProvider, BrowserConfig, ConfigOverrides, load_config, parse_viewport};

#[test]
fn parses_viewport() {
    let viewport = parse_viewport("1280x720").expect("viewport");
    assert_eq!(viewport.width, 1280);
    assert_eq!(viewport.height, 720);
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
