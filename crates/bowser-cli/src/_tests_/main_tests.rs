use clap::{Parser, error::ErrorKind};

use crate::args::{Cli, Command, SessionSubcommand};
use crate::cli::config_overrides;
use crate::dispatch::build_env_filter;

#[test]
fn env_filter_suppresses_chromiumoxide_handler_warnings() {
    let rendered = build_env_filter("warn").to_string();
    assert!(rendered.contains("chromiumoxide::handler=error"));
}

#[test]
fn cli_flags_enable_headed_and_persistent_profile_overrides() {
    let cli = Cli::parse_from([
        "bowser",
        "--headed",
        "--persistent-profile",
        "https://example.com",
    ]);
    let overrides = config_overrides(&cli);
    assert_eq!(overrides.headless, Some(false));
    assert_eq!(overrides.persistent_profile, Some(true));
}

#[test]
fn cli_rejects_zero_viewport_dimensions() {
    for viewport in ["0x720", "1280x0", "0x0"] {
        assert!(
            Cli::try_parse_from(["bowser", "--viewport", viewport, "https://example.com"]).is_err(),
            "accepted invalid viewport {viewport}"
        );
    }
}

#[test]
fn cli_version_flags_render_the_exact_package_version() {
    for flag in ["-V", "--version"] {
        let error = Cli::try_parse_from(["bowser", flag]).expect_err("version exits through Clap");
        assert_eq!(error.kind(), ErrorKind::DisplayVersion);
        assert_eq!(
            error.to_string(),
            format!("bowser {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn pointer_log_command_parses_defaults() {
    let cli = Cli::parse_from(["bowser", "pointer-log"]);
    let Some(Command::PointerLog(args)) = cli.command else {
        panic!("expected pointer-log command");
    };
    assert_eq!(args.bind.to_string(), "127.0.0.1:8765");
    assert!(args.output.is_none());
    assert!(!args.demo);
    assert_eq!(args.demo_clicks, None);
}

#[test]
fn pointer_log_command_parses_output_override() {
    let cli = Cli::parse_from(["bowser", "pointer-log", "--output", "custom.jsonl"]);
    let Some(Command::PointerLog(args)) = cli.command else {
        panic!("expected pointer-log command");
    };
    assert_eq!(
        args.output.as_ref().expect("output").to_string_lossy(),
        "custom.jsonl"
    );
}

#[test]
fn pointer_log_command_parses_demo_options() {
    let cli = Cli::parse_from(["bowser", "pointer-log", "--demo", "--demo-clicks", "3"]);
    let Some(Command::PointerLog(args)) = cli.command else {
        panic!("expected pointer-log command");
    };
    assert!(args.demo);
    assert_eq!(args.demo_clicks, Some(3));
}

#[test]
fn machine_and_checkpoint_commands_parse() {
    let export = Cli::parse_from([
        "bowser",
        "--json-envelope",
        "--session",
        "bsr_123",
        "session",
        "export",
        "--to",
        "checkpoint.json",
    ]);
    assert!(export.json_envelope);
    let Some(Command::Session {
        command: SessionSubcommand::Export { to },
    }) = export.command
    else {
        panic!("expected session export command");
    };
    assert_eq!(to.to_string_lossy(), "checkpoint.json");

    let restore = Cli::parse_from(["bowser", "session", "restore", "--from", "checkpoint.json"]);
    assert!(matches!(
        restore.command,
        Some(Command::Session {
            command: SessionSubcommand::Restore { .. }
        })
    ));
    assert!(matches!(
        Cli::parse_from(["bowser", "capabilities"]).command,
        Some(Command::Capabilities)
    ));
}

#[test]
fn history_commands_parse_page_and_output_options() {
    for command_name in ["back", "forward", "reload"] {
        let cli = Cli::parse_from([
            "bowser",
            "--session",
            "bsr_123",
            command_name,
            "--page-id",
            "pg_2",
            "--format",
            "json",
            "--output",
            "capture.json",
        ]);
        let args = match cli.command.expect("history command") {
            Command::Back(args) | Command::Forward(args) | Command::Reload(args) => args,
            other => panic!("unexpected command: {other:?}"),
        };
        assert_eq!(args.page_id.as_deref(), Some("pg_2"));
        assert_eq!(
            args.output.as_deref(),
            Some(std::path::Path::new("capture.json"))
        );
    }
}
