//! Tests for Bowser check-plan target coverage.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::BOWSER_COMMANDS;
use crate::check::CheckCommand;

const BOWSER_TEST_PACKAGES: &[&str] = &["bowser", "bowser-cli"];
const BOWSER_HEADLESS_ENV: (&str, &str) = ("BOWSER_HEADLESS", "true");
const BOWSER_STEALTH_FEATURES_ENV: &str = "BOWSER_INTERNAL_STEALTH_FEATURES";
const BACKEND_FOCUS_TEST_NAME: &str = "stealth_typing_replaces_prefilled_same_page_text";

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    targets: Vec<CargoTarget>,
}

#[derive(Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
}

#[test]
fn browser_test_commands_cover_cargo_test_targets() {
    let metadata = cargo_metadata(workspace_root().as_path());
    let expected_targets = metadata_test_targets_by_package(&metadata);
    let command_targets = command_test_targets_by_package();

    assert_eq!(command_targets, expected_targets);
}

#[test]
fn browser_running_commands_use_one_test_thread() {
    for command in BOWSER_COMMANDS {
        if command.args.contains(&"--test") {
            assert!(command.args.ends_with(&["--", "--test-threads=1"]));
        }
    }
}

#[test]
fn browser_running_commands_force_headless_mode() {
    for command in BOWSER_COMMANDS {
        if command
            .envs
            .iter()
            .any(|(name, _)| *name == BOWSER_STEALTH_FEATURES_ENV)
        {
            assert!(command.envs.contains(&BOWSER_HEADLESS_ENV));
        }
    }
}

#[test]
fn bowser_doctests_force_headless_mode() {
    let doctest = BOWSER_COMMANDS
        .iter()
        .find(|command| command.args == ["test", "--locked", "-p", "bowser", "--doc"])
        .expect("Bowser doctest command");

    assert!(doctest.envs.contains(&BOWSER_HEADLESS_ENV));
    assert!(
        doctest
            .envs
            .iter()
            .any(|(name, _)| *name == BOWSER_STEALTH_FEATURES_ENV)
    );
}

#[test]
fn backend_focus_regression_enables_accessibility_capture() {
    let command = BOWSER_COMMANDS
        .iter()
        .find(|command| command.args.contains(&BACKEND_FOCUS_TEST_NAME))
        .expect("backend-focus regression command");

    assert!(command.envs.contains(&BOWSER_HEADLESS_ENV));
    assert!(command.envs.contains(&(
        BOWSER_STEALTH_FEATURES_ENV,
        "+accessibility-capture,-launch-headed,-launch-native-window",
    )));
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a workspace parent")
        .to_path_buf()
}

fn cargo_metadata(workspace_root: &Path) -> CargoMetadata {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(workspace_root)
        .output()
        .expect("cargo metadata starts");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("cargo metadata output parses")
}

fn metadata_test_targets_by_package(
    metadata: &CargoMetadata,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut targets_by_package = BTreeMap::new();

    for package_name in BOWSER_TEST_PACKAGES {
        let package = metadata
            .packages
            .iter()
            .find(|package| package.name == *package_name)
            .expect("Bowser package exists in cargo metadata");
        let targets = package
            .targets
            .iter()
            .filter(|target| target.kind.iter().any(|kind| kind == "test"))
            .map(|target| target.name.clone())
            .collect();

        targets_by_package.insert((*package_name).to_string(), targets);
    }

    targets_by_package
}

fn command_test_targets_by_package() -> BTreeMap<String, BTreeSet<String>> {
    let mut targets_by_package = BTreeMap::new();

    for package_name in BOWSER_TEST_PACKAGES {
        targets_by_package.insert((*package_name).to_string(), BTreeSet::new());
    }

    for command in BOWSER_COMMANDS {
        let Some(package_name) = cargo_package(command) else {
            continue;
        };
        if !BOWSER_TEST_PACKAGES.contains(&package_name) {
            continue;
        }

        let targets = targets_by_package
            .get_mut(package_name)
            .expect("Bowser package target set exists");
        for target in command_test_targets(command) {
            targets.insert(target.to_string());
        }
    }

    targets_by_package
}

fn cargo_package(command: &CheckCommand) -> Option<&'static str> {
    command.args.windows(2).find_map(|args| match args {
        ["-p", package_name] => Some(*package_name),
        _ => None,
    })
}

fn command_test_targets(command: &CheckCommand) -> Vec<&'static str> {
    command
        .args
        .windows(2)
        .filter_map(|args| match args {
            ["--test", target_name] => Some(*target_name),
            _ => None,
        })
        .collect()
}
