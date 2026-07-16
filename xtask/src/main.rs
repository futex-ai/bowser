//! Command-line entrypoint for workspace automation tasks.

#![warn(unreachable_pub)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::check::{CommandRunner, RealCommandRunner, run_check};
use crate::check_plan::{CheckPhase, CheckSelection};
use crate::error::Result;
use crate::review::run_review;
use crate::rust_file_length_lint::{RustFileLengthLintMode, run_rust_file_length_lint};
use crate::rust_source_audit::run_rust_source_audit;
use crate::rust_trait_audit::run_rust_trait_audit;

mod check;
mod check_plan;
mod check_plan_bowser;
mod error;
mod review;
mod review_codex;
mod review_command_display;
mod review_git;
mod review_omitted_paths;
mod review_paths;
mod review_prompt;
mod review_sandbox;
mod review_sensitive_paths;
mod review_tracked;
mod review_untracked;
mod rust_file_length_lint;
mod rust_source_audit;
mod rust_trait_audit;
mod rust_trait_audit_allowlist;
mod source_layout_lint;

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Bowser workspace automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the local equivalent of repository CI.
    Check(CheckArgs),
    /// Run a read-only AI code review against origin/main and local changes.
    Review,
    /// Audit changed Rust files for file-length limits.
    RustFileLengthLint(RustFileLengthLintArgs),
    /// Audit workspace Rust source graphs for orphan files and modules.
    RustSourceAudit,
    /// Audit Rust trait-boundary testing rules.
    RustTraitAudit,
}

#[derive(Args, Debug)]
struct CheckArgs {
    /// Run only these verification phases. Repeat or use comma-separated values.
    #[arg(long, value_enum, value_delimiter = ',')]
    include: Vec<CheckPhaseArg>,
    /// Skip these verification phases. Repeat or use comma-separated values.
    #[arg(long, value_enum, value_delimiter = ',')]
    exclude: Vec<CheckPhaseArg>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CheckPhaseArg {
    /// Release helper script tests.
    Scripts,
    /// GitHub Actions workflow linting.
    Workflow,
    /// Rust source, file-length, and trait-boundary audits.
    RustAudit,
    /// Rust formatting checks.
    RustFmt,
    /// Rust clippy checks.
    RustClippy,
    /// Workspace automation tests.
    RustTest,
    /// Bowser library, CLI, browser integration, and smoke tests.
    Bowser,
}

impl From<CheckPhaseArg> for CheckPhase {
    fn from(phase: CheckPhaseArg) -> Self {
        match phase {
            CheckPhaseArg::Scripts => Self::Scripts,
            CheckPhaseArg::Workflow => Self::Workflow,
            CheckPhaseArg::RustAudit => Self::RustAudit,
            CheckPhaseArg::RustFmt => Self::RustFmt,
            CheckPhaseArg::RustClippy => Self::RustClippy,
            CheckPhaseArg::RustTest => Self::RustTest,
            CheckPhaseArg::Bowser => Self::Bowser,
        }
    }
}

#[derive(Args, Debug)]
struct RustFileLengthLintArgs {
    /// Audit all Rust files under crates/ and xtask.
    #[arg(long)]
    all: bool,
}

fn main() -> ExitCode {
    let runner = RealCommandRunner;

    match run(Cli::parse(), &runner) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli, runner: &dyn CommandRunner) -> Result<()> {
    match cli.command {
        Commands::Check(args) => {
            let selection = CheckSelection::from_filters(
                args.include.into_iter().map(CheckPhase::from).collect(),
                args.exclude.into_iter().map(CheckPhase::from).collect(),
            );
            run_check(runner, workspace_root()?.as_path(), &selection)
        }
        Commands::Review => run_review(workspace_root()?.as_path()),
        Commands::RustFileLengthLint(args) => run_rust_file_length_lint(
            workspace_root()?.as_path(),
            if args.all {
                RustFileLengthLintMode::AllFiles
            } else {
                RustFileLengthLintMode::ChangedFiles
            },
        ),
        Commands::RustSourceAudit => run_rust_source_audit(workspace_root()?.as_path()),
        Commands::RustTraitAudit => run_rust_trait_audit(workspace_root()?.as_path()),
    }
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(workspace_root) = manifest_dir.parent() else {
        return Err(crate::error::Error::MissingWorkspaceRoot { manifest_dir });
    };

    Ok(workspace_root.to_path_buf())
}

#[cfg(test)]
#[path = "_tests_/main_tests.rs"]
mod main_tests;
