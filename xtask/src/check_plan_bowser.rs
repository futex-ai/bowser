//! Bowser verification commands for the workspace check plan.

use crate::check::CheckCommand;

const CI_CARGO_ENV: &[(&str, &str)] = &[
    ("CARGO_INCREMENTAL", "0"),
    ("CARGO_PROFILE_DEV_DEBUG", "0"),
    ("CARGO_PROFILE_TEST_DEBUG", "0"),
];
const BOWSER_BROWSER_TEST_ENV: &[(&str, &str)] = &[
    ("CARGO_INCREMENTAL", "0"),
    ("CARGO_PROFILE_DEV_DEBUG", "0"),
    ("CARGO_PROFILE_TEST_DEBUG", "0"),
    ("BOWSER_HEADLESS", "true"),
    (
        "BOWSER_INTERNAL_STEALTH_FEATURES",
        "-launch-headed,-launch-native-window",
    ),
];

/// Commands for Bowser's pure tests, browser integration tests, and CLI smoke.
pub(crate) const BOWSER_COMMANDS: &[CheckCommand] = &[
    CheckCommand::root(
        "cargo",
        &["test", "--locked", "-p", "bowser", "--lib"],
        CI_CARGO_ENV,
    ),
    CheckCommand::root(
        "cargo",
        &["test", "--locked", "-p", "bowser", "--doc"],
        BOWSER_BROWSER_TEST_ENV,
    ),
    CheckCommand::root(
        "cargo",
        &[
            "test",
            "--locked",
            "-p",
            "bowser",
            "--test",
            "browser_flows",
            "--test",
            "click_focus",
            "--test",
            "document_capture",
            "--test",
            "input_flows",
            "--test",
            "keypress",
            "--test",
            "navigation",
            "--test",
            "navigation_cancellation",
            "--test",
            "navigation_history",
            "--test",
            "navigation_resumed_tests",
            "--test",
            "navigation_timeout",
            "--test",
            "pointer_click",
            "--test",
            "session_pages",
            "--",
            "--test-threads=1",
        ],
        BOWSER_BROWSER_TEST_ENV,
    ),
    CheckCommand::root(
        "cargo",
        &["test", "--locked", "-p", "bowser-cli", "--bin", "bowser"],
        CI_CARGO_ENV,
    ),
    CheckCommand::root(
        "cargo",
        &[
            "test",
            "--locked",
            "-p",
            "bowser-cli",
            "--test",
            "cli_flows",
            "--test",
            "interactive_repl",
            "--",
            "--test-threads=1",
        ],
        BOWSER_BROWSER_TEST_ENV,
    ),
    CheckCommand::root(
        "cargo",
        &[
            "run",
            "--locked",
            "-p",
            "bowser-cli",
            "--bin",
            "bowser",
            "--",
            "--help",
        ],
        CI_CARGO_ENV,
    ),
];

#[cfg(test)]
#[path = "_tests_/check_plan_bowser_tests.rs"]
mod check_plan_bowser_tests;
