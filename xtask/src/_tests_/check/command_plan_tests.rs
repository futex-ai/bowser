use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::check_plan::{CheckPhase, CheckSelection, check_commands};

use super::super::*;

static TEMP_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);
const HEADLESS_STEALTH_ENV: (&str, &str) = (
    "BOWSER_INTERNAL_STEALTH_FEATURES",
    "-launch-headed,-launch-native-window",
);
const HEADLESS_ENV: (&str, &str) = ("BOWSER_HEADLESS", "true");

#[test]
fn check_runs_the_complete_standalone_bowser_plan() {
    let commands = Arc::new(Mutex::new(Vec::new()));
    let runner = Unimock::new(
        CommandRunnerMock::run
            .each_call(matching!(_, _))
            .answers_arc({
                let commands = commands.clone();
                Arc::new(move |_, command, _| {
                    commands.lock().expect("commands").push(command);
                    Ok(())
                })
            }),
    );
    let workspace = temp_workspace();
    let selection = CheckSelection::from_filters(Vec::new(), Vec::new());

    run_check(&runner, workspace.path(), &selection).unwrap();

    let commands = commands.lock().expect("commands");
    assert_eq!(commands.as_slice(), check_commands(&selection).as_slice());
    assert!(commands.iter().any(|command| {
        command.program == "bash" && command.args == ["scripts/test-release.sh"]
    }));
    assert!(commands.iter().any(|command| {
        command.program == "shellcheck"
            && command.args.contains(&"scripts/bowser-google-control.sh")
            && command
                .args
                .contains(&"scripts/bowser-google-smoke-matrix.sh")
            && command.args.contains(&"scripts/release.sh")
            && command.args.contains(&"scripts/test-release.sh")
    }));
    assert!(commands.iter().any(|command| {
        command.program == "actionlint"
            && command.args.contains(&".github/workflows/pull-request.yml")
            && command.args.contains(&".github/workflows/main.yml")
            && command.args.contains(&".github/workflows/release.yml")
    }));
    assert!(commands.iter().any(|command| {
        command.program == "cargo"
            && command.args == ["run", "--locked", "-p", "xtask", "--", "rust-source-audit"]
    }));
    assert!(commands.iter().any(|command| {
        command.program == "cargo"
            && command.args
                == [
                    "run",
                    "--locked",
                    "-p",
                    "xtask",
                    "--",
                    "rust-file-length-lint",
                    "--all",
                ]
    }));
    assert!(commands.iter().any(|command| {
        command.program == "cargo"
            && command.args == ["test", "--locked", "-p", "bowser", "--lib"]
            && !command.envs.contains(&HEADLESS_STEALTH_ENV)
    }));
    assert!(commands.iter().any(|command| {
        command.program == "cargo"
            && command.args == ["test", "--locked", "-p", "bowser", "--doc"]
            && command.envs.contains(&HEADLESS_ENV)
            && command.envs.contains(&HEADLESS_STEALTH_ENV)
    }));
    assert!(commands.iter().any(|command| {
        command.args.contains(&"browser_flows")
            && command.envs.contains(&HEADLESS_ENV)
            && command.envs.contains(&HEADLESS_STEALTH_ENV)
    }));
    assert!(commands.iter().any(|command| {
        command.args.contains(&"interactive_repl")
            && command.envs.contains(&HEADLESS_ENV)
            && command.envs.contains(&HEADLESS_STEALTH_ENV)
    }));
    assert!(commands.iter().any(|command| {
        command.program == "cargo"
            && command.args
                == [
                    "run",
                    "--locked",
                    "-p",
                    "bowser-cli",
                    "--bin",
                    "bowser",
                    "--",
                    "--help",
                ]
    }));
    assert!(
        commands
            .iter()
            .all(|command| command.program != "cargo" || command.args != ["clean"])
    );
}

#[test]
fn focused_check_does_not_run_an_unselected_source_layout_audit() {
    let runner = Unimock::new(
        CommandRunnerMock::run
            .next_call(matching!(_, _))
            .returns(Ok(())),
    );
    let workspace = temp_workspace();
    std::fs::write(
        workspace.path().join("xtask/src/invalid.rs"),
        "#[test]\nfn inline_test() {}\n",
    )
    .unwrap();
    let selection = CheckSelection::from_filters(vec![CheckPhase::RustFmt], Vec::new());

    run_check(&runner, workspace.path(), &selection).unwrap();
}

struct TempWorkspace {
    path: std::path::PathBuf,
}

impl TempWorkspace {
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn temp_workspace() -> TempWorkspace {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique = TEMP_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "bowser-xtask-check-{}-{nanos}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.join("xtask/src")).unwrap();
    TempWorkspace { path }
}
