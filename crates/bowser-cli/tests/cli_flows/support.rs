#[path = "../../../bowser/tests/support/mod.rs"]
mod bowser_support;

use std::process::Output;
use std::thread;
use std::time::Duration;

use assert_cmd::Command;

pub(crate) use super::session_dir_support::test_session_dir;
pub(crate) use bowser_support::{browser_test_guard, chrome_path, spawn_server};

const HEADLESS_STEALTH_FEATURES: &str = "-launch-headed,-launch-native-window";

pub(crate) fn bowser_command() -> Command {
    let mut command = Command::cargo_bin("bowser").expect("bowser bin");
    command.env("BOWSER_HEADLESS", "true");
    command.env(
        "BOWSER_INTERNAL_STEALTH_FEATURES",
        HEADLESS_STEALTH_FEATURES,
    );
    command
}

pub(crate) fn parse_session_id(stderr: &str) -> String {
    stderr
        .lines()
        .find_map(|line| line.strip_prefix("Session: "))
        .map(ToString::to_string)
        .expect("session id")
}

pub(crate) fn parse_page_id(output: &str, needle: &str) -> String {
    output
        .lines()
        .find_map(|line| {
            if line.contains(needle) {
                let trimmed = line.trim_start();
                let trimmed = trimmed.strip_prefix("* ").unwrap_or(trimmed);
                trimmed.split_whitespace().next().map(ToString::to_string)
            } else {
                None
            }
        })
        .expect("page id")
}

pub(crate) fn run_bowser_with_retry(args: &[&str], attempts: usize) -> Output {
    let mut last_output = None;
    for attempt in 0..attempts {
        let output = bowser_command()
            .args(args)
            .output()
            .expect("run bowser command");
        if output.status.success() || attempt + 1 == attempts {
            return output;
        }
        last_output = Some(output);
        thread::sleep(Duration::from_millis(250));
    }
    last_output.expect("bowser output")
}
