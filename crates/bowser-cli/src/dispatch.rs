//! Top-level runtime dispatch.

use std::path::PathBuf;

use clap::Parser;
use clap::error::ErrorKind;
use tracing_subscriber::EnvFilter;

use crate::args::{Cli, Command};
use crate::cli::{config_overrides, resolve_command};
use crate::commands;
use crate::error::{CliError, Result};
use crate::output::{CommandContext, CommandOutput, emit_failure, emit_success};

pub(crate) async fn run() -> i32 {
    let machine_requested = machine_flag_requested(std::env::args_os());
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) if machine_requested && !is_display_request(error.kind()) => {
            let context = CommandContext::default();
            let error = CliError::Arguments {
                message: error.to_string(),
            };
            let _ = emit_failure(true, &context, &error);
            return 2;
        }
        Err(error) => {
            let exit_code = error.exit_code();
            let _ = error.print();
            return exit_code;
        }
    };
    init_tracing(cli.verbose, cli.quiet);
    let machine = cli.json_envelope;
    let mut context = CommandContext {
        session: cli.session.clone(),
        page: None,
        announce_session_on_failure: false,
    };
    let result = execute(cli, &mut context).await;
    match result {
        Ok(output) => {
            if let Err(error) = output.commit_files().await {
                let _ = emit_failure(machine, &context, &error);
                return 1;
            }
            match emit_success(machine, &context, &output) {
                Ok(()) => 0,
                Err(error) => {
                    let _ = emit_failure(machine, &context, &error);
                    1
                }
            }
        }
        Err(error) => {
            context.update_from_error(&error);
            let _ = emit_failure(machine, &context, &error);
            1
        }
    }
}

fn machine_flag_requested(args: impl IntoIterator<Item = std::ffi::OsString>) -> bool {
    args.into_iter()
        .any(|argument| argument == "--json-envelope")
}

fn is_display_request(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::DisplayHelp | ErrorKind::DisplayVersion)
}

async fn execute(cli: Cli, context: &mut CommandContext) -> Result<CommandOutput> {
    let config_path = cli.config.clone();
    let overrides = config_overrides(&cli);
    let machine = cli.json_envelope;
    let command = match resolve_command(cli)? {
        Command::PointerLog(args) => {
            if machine {
                return Err(CliError::InvalidCommand {
                    input: "--json-envelope is not supported by pointer-log".to_string(),
                });
            }
            let config = if args.demo {
                Some(load_browser_config(config_path, overrides)?)
            } else {
                None
            };
            commands::pointer_log::run(args, config).await?;
            return CommandOutput::result(serde_json::json!({}));
        }
        Command::Interactive(_) if machine => {
            return Err(CliError::InvalidCommand {
                input: "--json-envelope is not supported by interactive mode".to_string(),
            });
        }
        command => command,
    };
    if matches!(command, Command::Capabilities) {
        return commands::capabilities::run();
    }
    let config = load_browser_config(config_path, overrides)?;
    match command {
        Command::Get(args) => commands::get::run(config, args, context).await,
        Command::Capture(args) => commands::capture::run(&config, args, context).await,
        Command::Download(args) => commands::download::run(config, args, context).await,
        Command::Expand(args) => commands::expand::run(&config, args, context).await,
        Command::Meta(args) => commands::meta::run(&config, args, context).await,
        Command::Describe(args) => commands::describe::run(&config, args, context).await,
        Command::Click(args) => commands::interactions::click(&config, args, context).await,
        Command::Type(args) => commands::interactions::type_text(&config, args, context).await,
        Command::Clear(args) => commands::interactions::clear(&config, args, context).await,
        Command::Submit(args) => commands::interactions::submit(&config, args, context).await,
        Command::Key(args) => commands::interactions::key(&config, args, context).await,
        Command::Scroll(args) => commands::interactions::scroll(&config, args, context).await,
        Command::Interactive(args) => {
            commands::interactive::run(config, args).await?;
            CommandOutput::result(serde_json::json!({}))
        }
        Command::Page { command } => commands::page::run(config, command, context).await,
        Command::Session { command } => commands::session::run(&config, command, context).await,
        Command::Capabilities => commands::capabilities::run(),
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
        .with_writer(std::io::stderr)
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
