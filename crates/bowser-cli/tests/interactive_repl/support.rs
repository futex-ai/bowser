#![cfg(unix)]

#[path = "../../../bowser/tests/support/mod.rs"]
mod bowser_support;

use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::time::Duration;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin;
use expectrl::session::OsSession;
use expectrl::{Eof, Expect, Regex, Session};

pub(crate) use bowser_support::{browser_test_guard, chrome_path, spawn_server};

pub(crate) const PROMPT_PATTERN: &str = r"bowser>(?:\x1b\[[0-9;?]*[ -/]*[@-~])* ";
pub(crate) const HEADLESS_STEALTH_FEATURES: &str = "-launch-headed,-launch-native-window";
pub(crate) const REPL_EXPECT_TIMEOUT: Duration = Duration::from_secs(90);

pub(crate) fn only_session_id(session_dir: &Path) -> String {
    let mut entries: Vec<(std::time::SystemTime, String)> = fs::read_dir(session_dir)
        .expect("read session dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()?.to_str() == Some("json")).then(|| {
                let modified = fs::metadata(&path)
                    .and_then(|metadata| metadata.modified())
                    .expect("modified time");
                let session_id = path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .expect("file stem")
                    .to_string();
                (modified, session_id)
            })
        })
        .collect();
    entries.sort_by_key(|(modified, _)| *modified);
    entries
        .pop()
        .map(|(_, session_id)| session_id)
        .expect("session metadata")
}

pub(crate) fn close_session(session_dir: &Path, session_id: &str) {
    let output = bowser_command()
        .args([
            "--session-dir",
            session_dir.to_str().expect("session dir"),
            "session",
            "close",
            session_id,
        ])
        .output()
        .expect("close session");
    assert!(
        output.status.success(),
        "close session failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(crate) fn expect_quit_and_close(mut repl: OsSession, session_dir: &Path) {
    repl.expect(Regex("Session: bsr_")).expect("session line");
    repl.expect(Eof).expect("repl eof");
    let session_id = only_session_id(session_dir);
    close_session(session_dir, &session_id);
}

pub(crate) fn expect_prompt(session: &mut OsSession, message: &str) {
    session.expect(Regex(PROMPT_PATTERN)).expect(message);
}

pub(crate) fn repl_command(
    binary: &Path,
    chrome_path: &Path,
    session_dir: &Path,
    tail_args: &[&str],
) -> OsSession {
    let mut command = ProcessCommand::new(binary);
    command
        .env("BOWSER_HEADLESS", "true")
        .env(
            "BOWSER_INTERNAL_STEALTH_FEATURES",
            HEADLESS_STEALTH_FEATURES,
        )
        .arg("--no-ai")
        .arg("--chrome-path")
        .arg(chrome_path)
        .arg("--session-dir")
        .arg(session_dir)
        .args(tail_args);
    let mut session = Session::spawn(command).expect("spawn repl command");
    session.set_expect_timeout(Some(REPL_EXPECT_TIMEOUT));
    session
}

pub(crate) fn binary_path() -> std::path::PathBuf {
    cargo_bin("bowser")
}

pub(crate) fn bowser_command() -> Command {
    let mut command = Command::cargo_bin("bowser").expect("bowser bin");
    command.env("BOWSER_HEADLESS", "true");
    command.env(
        "BOWSER_INTERNAL_STEALTH_FEATURES",
        HEADLESS_STEALTH_FEATURES,
    );
    command
}
