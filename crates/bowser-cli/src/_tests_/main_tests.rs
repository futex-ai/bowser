use clap::Parser;

use crate::args::{Cli, Command};
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
