//! Command plan for selectable workspace verification phases.

use crate::check::CheckCommand;
use crate::check_plan_bowser::BOWSER_COMMANDS;

const CI_CARGO_ENV: &[(&str, &str)] = &[
    ("CARGO_INCREMENTAL", "0"),
    ("CARGO_PROFILE_DEV_DEBUG", "0"),
    ("CARGO_PROFILE_TEST_DEBUG", "0"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckPhase {
    Scripts,
    Workflow,
    RustAudit,
    RustFmt,
    RustClippy,
    RustTest,
    Bowser,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CheckSelection {
    include: Vec<CheckPhase>,
    exclude: Vec<CheckPhase>,
}

impl CheckSelection {
    pub(crate) fn from_filters(include: Vec<CheckPhase>, exclude: Vec<CheckPhase>) -> Self {
        Self { include, exclude }
    }

    pub(crate) fn includes(&self, phase: CheckPhase) -> bool {
        (self.include.is_empty() || self.include.contains(&phase)) && !self.exclude.contains(&phase)
    }
}

const SCRIPT_COMMANDS: &[CheckCommand] = &[
    CheckCommand::root("bash", &["scripts/test-release.sh"], &[]),
    CheckCommand::root(
        "shellcheck",
        &[
            "scripts/bowser-google-control.sh",
            "scripts/bowser-google-smoke-matrix.sh",
            "scripts/release.sh",
            "scripts/test-release.sh",
        ],
        &[],
    ),
];

const WORKFLOW_COMMANDS: &[CheckCommand] = &[CheckCommand::root(
    "actionlint",
    &[
        ".github/workflows/main.yml",
        ".github/workflows/pull-request.yml",
        ".github/workflows/release.yml",
    ],
    &[],
)];

const RUST_AUDIT_COMMANDS: &[CheckCommand] = &[
    CheckCommand::root(
        "cargo",
        &["run", "--locked", "-p", "xtask", "--", "rust-source-audit"],
        CI_CARGO_ENV,
    ),
    CheckCommand::root(
        "cargo",
        &[
            "run",
            "--locked",
            "-p",
            "xtask",
            "--",
            "rust-file-length-lint",
            "--all",
        ],
        CI_CARGO_ENV,
    ),
    CheckCommand::root(
        "cargo",
        &["run", "--locked", "-p", "xtask", "--", "rust-trait-audit"],
        CI_CARGO_ENV,
    ),
];

const RUST_FMT_COMMANDS: &[CheckCommand] = &[CheckCommand::root(
    "cargo",
    &["fmt", "--all", "--", "--check"],
    CI_CARGO_ENV,
)];

const RUST_CLIPPY_COMMANDS: &[CheckCommand] = &[CheckCommand::root(
    "cargo",
    &[
        "clippy",
        "--locked",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ],
    CI_CARGO_ENV,
)];

const RUST_TEST_COMMANDS: &[CheckCommand] = &[CheckCommand::root(
    "cargo",
    &["test", "--locked", "-p", "xtask"],
    CI_CARGO_ENV,
)];

pub(crate) fn check_commands(selection: &CheckSelection) -> Vec<CheckCommand> {
    let mut commands = Vec::new();

    if selection.includes(CheckPhase::Scripts) {
        commands.extend_from_slice(SCRIPT_COMMANDS);
    }
    if selection.includes(CheckPhase::Workflow) {
        commands.extend_from_slice(WORKFLOW_COMMANDS);
    }
    if selection.includes(CheckPhase::RustAudit) {
        commands.extend_from_slice(RUST_AUDIT_COMMANDS);
    }
    if selection.includes(CheckPhase::RustFmt) {
        commands.extend_from_slice(RUST_FMT_COMMANDS);
    }
    if selection.includes(CheckPhase::RustClippy) {
        commands.extend_from_slice(RUST_CLIPPY_COMMANDS);
    }
    if selection.includes(CheckPhase::RustTest) {
        commands.extend_from_slice(RUST_TEST_COMMANDS);
    }
    if selection.includes(CheckPhase::Bowser) {
        commands.extend_from_slice(BOWSER_COMMANDS);
    }

    commands
}
