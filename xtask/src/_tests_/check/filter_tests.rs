use crate::check_plan::{CheckPhase, CheckSelection, check_commands};

use super::super::*;

const HEADLESS_STEALTH_ENV: (&str, &str) = (
    "BOWSER_INTERNAL_STEALTH_FEATURES",
    "-launch-headed,-launch-native-window",
);
const BACKEND_FOCUS_ENV: (&str, &str) = (
    "BOWSER_INTERNAL_STEALTH_FEATURES",
    "+accessibility-capture,-launch-headed,-launch-native-window",
);
const HEADLESS_ENV: (&str, &str) = ("BOWSER_HEADLESS", "true");

#[test]
fn exclude_bowser_keeps_non_browser_phases() {
    let commands = check_commands(&CheckSelection::from_filters(
        Vec::new(),
        vec![CheckPhase::Bowser],
    ));

    assert!(commands.iter().any(|command| command.program == "bash"));
    assert!(
        commands
            .iter()
            .any(|command| command.program == "actionlint")
    );
    assert!(
        commands
            .iter()
            .any(|command| { command.program == "cargo" && command.args.contains(&"clippy") })
    );
    assert!(
        commands
            .iter()
            .all(|command| !is_bowser_package_command(command))
    );
}

#[test]
fn include_multiple_phases_runs_only_selected_phases() {
    let commands = check_commands(&CheckSelection::from_filters(
        vec![CheckPhase::Scripts, CheckPhase::Workflow],
        Vec::new(),
    ));

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].program, "bash");
    assert_eq!(commands[1].program, "shellcheck");
    assert_eq!(commands[2].program, "actionlint");
}

#[test]
fn include_bowser_runs_all_browser_and_smoke_commands() {
    let commands = check_commands(&CheckSelection::from_filters(
        vec![CheckPhase::Bowser],
        Vec::new(),
    ));

    assert_eq!(commands.len(), 7);
    assert_eq!(
        commands[0].args,
        ["test", "--locked", "-p", "bowser", "--lib"]
    );
    assert!(!commands[0].envs.contains(&HEADLESS_STEALTH_ENV));
    assert_eq!(
        commands[1].args,
        ["test", "--locked", "-p", "bowser", "--doc"]
    );
    assert!(commands[1].envs.contains(&HEADLESS_ENV));
    assert!(commands[1].envs.contains(&HEADLESS_STEALTH_ENV));
    assert!(commands[2].args.contains(&"browser_flows"));
    assert!(commands[2].envs.contains(&HEADLESS_ENV));
    assert!(commands[2].envs.contains(&BACKEND_FOCUS_ENV));
    assert!(commands[3].args.contains(&"browser_flows"));
    assert!(commands[3].envs.contains(&HEADLESS_ENV));
    assert!(commands[3].envs.contains(&HEADLESS_STEALTH_ENV));
    assert_eq!(
        commands[4].args,
        ["test", "--locked", "-p", "bowser-cli", "--bin", "bowser"]
    );
    assert!(commands[5].args.contains(&"interactive_repl"));
    assert_eq!(commands[6].args.last(), Some(&"--help"));
}

#[test]
fn include_rust_test_only_runs_xtask_tests() {
    let commands = check_commands(&CheckSelection::from_filters(
        vec![CheckPhase::RustTest],
        Vec::new(),
    ));

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].args, ["test", "--locked", "-p", "xtask"]);
}

fn is_bowser_package_command(command: &CheckCommand) -> bool {
    command.program == "cargo"
        && command
            .args
            .windows(2)
            .any(|args| matches!(args, ["-p", "bowser"] | ["-p", "bowser-cli"]))
}
