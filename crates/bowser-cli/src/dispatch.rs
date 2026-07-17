//! Top-level runtime dispatch.

use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::args::{Cli, Command};
use crate::cli::{config_overrides, resolve_command};
use crate::commands;
use crate::error::{CliError, Result};

pub(crate) async fn run() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose, cli.quiet);
    let config_path = cli.config.clone();
    let overrides = config_overrides(&cli);
    let command = match resolve_command(cli)? {
        Command::PointerLog(args) => {
            let config = if args.demo {
                Some(load_browser_config(config_path, overrides)?)
            } else {
                None
            };
            return commands::pointer_log::run(args, config).await;
        }
        command => command,
    };
    let config = load_browser_config(config_path, overrides)?;
    match command {
        Command::Get(args) => commands::get::run(config, args).await,
        Command::Capture(args) => commands::capture::run(&config, args).await,
        Command::Download(args) => commands::download::run(config, args).await,
        Command::Expand(args) => commands::expand::run(&config, args).await,
        Command::Meta(args) => commands::meta::run(&config, args).await,
        Command::Describe(args) => commands::describe::run(&config, args).await,
        Command::Click(args) => commands::interactions::click(&config, args).await,
        Command::Type(args) => commands::interactions::type_text(&config, args).await,
        Command::Clear(args) => commands::interactions::clear(&config, args).await,
        Command::Submit(args) => commands::interactions::submit(&config, args).await,
        Command::Key(args) => commands::interactions::key(&config, args).await,
        Command::Scroll(args) => commands::interactions::scroll(&config, args).await,
        Command::Interactive(args) => commands::interactive::run(config, args).await,
        Command::Page { command } => commands::page::run(config, command).await,
        Command::Session { command } => commands::session::run(&config, command).await,
        Command::PointerLog(_) => Err(CliError::InvalidCommand {
            input: "unexpected pointer-log command".to_string(),
        }),
    }
}

fn load_browser_config(
    config_path: Option<PathBuf>,
    overrides: bowser::ConfigOverrides,
) -> Result<bowser::BrowserConfig> {
    match bowser::load_config(config_path.as_deref(), overrides) {
        Ok(config) => Ok(config),
        Err(source) => Err(CliError::ConfigLoad {
            path: config_path
                .unwrap_or_else(bowser::default_config_path)
                .display()
                .to_string(),
            source,
        }),
    }
}

fn init_tracing(verbose: u8, quiet: bool) {
    let filter = if quiet {
        build_env_filter("error")
    } else {
        build_env_filter(match verbose {
            0 => "warn",
            1 => "info",
            _ => "debug",
        })
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .try_init();
}

pub(crate) fn build_env_filter(level: &str) -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(
            level
                .parse()
                .unwrap_or_else(|_| tracing::Level::WARN.into()),
        )
        .parse_lossy("chromiumoxide::handler=error")
}

#[cfg(test)]
#[path = "_tests_/dispatch_tests.rs"]
mod dispatch_tests;
