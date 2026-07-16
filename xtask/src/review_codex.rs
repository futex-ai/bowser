//! Nested Codex process invocation for local review.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::{Error as IoError, ErrorKind, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::error::{Error, Result};
use crate::review_command_display::command_display;
use crate::review_sandbox::ReviewSandbox;

const CODEX_PROGRAM: &str = "codex";
const CODEX_MODEL: &str = "gpt-5.5";
const CODEX_REASONING_CONFIG: &str = r#"model_reasoning_effort="xhigh""#;
const CODEX_ENV_ALLOWLIST: &[&str] = &["CODEX_HOME", "HOME", "PATH", "TERM"];
const CODEX_TEMP_ENV: &[&str] = &["TMPDIR", "TMP", "TEMP"];

/// Run a nested Codex review process with the supplied prompt.
pub(crate) fn run_codex_command(workspace_root: &Path, prompt: &str) -> Result<()> {
    let sandbox = ReviewSandbox::create(workspace_root)?;
    let review_root = sandbox.path();
    let command_display = codex_command_display(review_root);
    let mut command = codex_command(review_root);
    let mut child = match command.stdin(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(source) => {
            return Err(Error::ReviewCodexStart {
                command: command_display.clone(),
                source,
            });
        }
    };

    if let Err(error) = write_codex_prompt(&mut child, &command_display, prompt) {
        terminate_codex_child(&mut child);
        return Err(error);
    }

    let status = match child.wait() {
        Ok(status) => status,
        Err(source) => {
            return Err(Error::ReviewCodexWait {
                command: command_display.clone(),
                source,
            });
        }
    };

    if status.success() {
        return Ok(());
    }

    Err(Error::ReviewCodexFailed {
        command: command_display,
        status,
    })
}

/// Format the nested Codex command for diagnostics and tests.
pub(crate) fn codex_command_display(review_root: &Path) -> String {
    command_display(
        CODEX_PROGRAM,
        &[
            "--ask-for-approval".to_owned(),
            "never".to_owned(),
            "exec".to_owned(),
            "--ephemeral".to_owned(),
            "--ignore-rules".to_owned(),
            "--model".to_owned(),
            CODEX_MODEL.to_owned(),
            "--config".to_owned(),
            CODEX_REASONING_CONFIG.to_owned(),
            "--sandbox".to_owned(),
            "read-only".to_owned(),
            "--skip-git-repo-check".to_owned(),
            "--cd".to_owned(),
            review_root.display().to_string(),
            "-".to_owned(),
        ],
    )
}

fn codex_command(review_root: &Path) -> Command {
    let mut command = Command::new(CODEX_PROGRAM);
    command.current_dir(review_root).env_clear();
    for (name, value) in codex_environment_from(std::env::vars_os(), review_root) {
        command.env(name, value);
    }
    command
        .args([
            "--ask-for-approval",
            "never",
            "exec",
            "--ephemeral",
            "--ignore-rules",
            "--model",
            CODEX_MODEL,
            "--config",
            CODEX_REASONING_CONFIG,
            "--sandbox",
            "read-only",
            "--skip-git-repo-check",
            "--cd",
        ])
        .arg(review_root)
        .arg("-");
    command
}

pub(crate) fn codex_environment_from(
    current_environment: impl IntoIterator<Item = (OsString, OsString)>,
    review_root: &Path,
) -> Vec<(OsString, OsString)> {
    let mut current_values = BTreeMap::new();
    for (name, value) in current_environment {
        if let Some(name) = name.to_str() {
            current_values.insert(name.to_owned(), value);
        }
    }

    let mut environment = Vec::new();
    for name in CODEX_ENV_ALLOWLIST {
        if let Some(value) = current_values.get(*name) {
            environment.push((OsString::from(*name), value.clone()));
        }
    }
    for name in CODEX_TEMP_ENV {
        environment.push((
            OsString::from(*name),
            review_root.as_os_str().to_os_string(),
        ));
    }

    environment
}

fn write_codex_prompt(child: &mut Child, command: &str, prompt: &str) -> Result<()> {
    let Some(mut stdin) = child.stdin.take() else {
        return Err(Error::ReviewCodexPromptWrite {
            command: command.to_owned(),
            source: IoError::new(ErrorKind::BrokenPipe, "codex stdin pipe unavailable"),
        });
    };

    if let Err(source) = stdin.write_all(prompt.as_bytes()) {
        return Err(Error::ReviewCodexPromptWrite {
            command: command.to_owned(),
            source,
        });
    }

    Ok(())
}

fn terminate_codex_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
