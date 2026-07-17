use super::*;

#[test]
fn codex_command_uses_gpt_5_5_with_extra_high_reasoning() {
    let command = codex_command_display(Path::new("/neutral review root"));

    assert!(command.contains("codex --ask-for-approval never exec"));
    assert!(command.contains("--ephemeral"));
    assert!(command.contains("--ignore-rules"));
    assert!(command.contains("--model gpt-5.5"));
    assert!(command.contains(r#"--config 'model_reasoning_effort="xhigh"'"#));
    assert!(command.contains("--sandbox read-only"));
    assert!(command.contains("--skip-git-repo-check"));
    assert!(command.contains("--cd '/neutral review root'"));
}

#[test]
fn codex_environment_scrubs_secret_values() {
    let environment = codex_environment_from(
        [
            ("PATH".into(), "/usr/bin".into()),
            ("HOME".into(), "/Users/reviewer".into()),
            ("CODEX_HOME".into(), "/Users/reviewer/.codex".into()),
            ("OPENAI_API_KEY".into(), "secret".into()),
            ("AWS_SECRET_ACCESS_KEY".into(), "secret".into()),
        ],
        Path::new("/neutral-review-root"),
    );
    let names = environment
        .iter()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert!(names.contains(&"PATH".to_owned()));
    assert!(names.contains(&"HOME".to_owned()));
    assert!(names.contains(&"CODEX_HOME".to_owned()));
    assert!(names.contains(&"TMPDIR".to_owned()));
    assert!(names.contains(&"TMP".to_owned()));
    assert!(names.contains(&"TEMP".to_owned()));
    assert!(!names.contains(&"OPENAI_API_KEY".to_owned()));
    assert!(!names.contains(&"AWS_SECRET_ACCESS_KEY".to_owned()));
    assert!(environment.iter().any(|(name, value)| {
        name == "TMPDIR" && value.to_string_lossy() == "/neutral-review-root"
    }));
}
