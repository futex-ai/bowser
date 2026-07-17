//! Git subprocess helpers for local review context collection.

use std::path::Path;
use std::process::{Command, Output};

use crate::error::{Error, Result};
use crate::review_command_display::{command_display, stderr_string};

/// Run Git and return stdout decoded as UTF-8 lossily.
pub(crate) fn git_stdout(workspace_root: &Path, args: &[&str]) -> Result<String> {
    let output = git_output(workspace_root, args)?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run Git with owned arguments and return stdout decoded as UTF-8 lossily.
pub(crate) fn git_stdout_dynamic(workspace_root: &Path, args: &[String]) -> Result<String> {
    let output = git_output_dynamic(workspace_root, args)?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run Git and return raw stdout bytes.
pub(crate) fn git_stdout_bytes(workspace_root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    Ok(git_output(workspace_root, args)?.stdout)
}

fn git_output(workspace_root: &Path, args: &[&str]) -> Result<Output> {
    let output = match Command::new("git")
        .args(args)
        .current_dir(workspace_root)
        .output()
    {
        Ok(output) => output,
        Err(source) => {
            return Err(Error::ReviewGitCommandStart {
                command: git_command_display(args),
                source,
            });
        }
    };

    if !output.status.success() {
        return Err(Error::ReviewGitCommandFailed {
            command: git_command_display(args),
            status: output.status,
            stderr: stderr_string(&output.stderr),
        });
    }

    Ok(output)
}

fn git_output_dynamic(workspace_root: &Path, args: &[String]) -> Result<Output> {
    let command = command_display("git", args);
    let output = match Command::new("git")
        .args(args)
        .current_dir(workspace_root)
        .output()
    {
        Ok(output) => output,
        Err(source) => {
            return Err(Error::ReviewGitCommandStart { command, source });
        }
    };

    if !output.status.success() {
        return Err(Error::ReviewGitCommandFailed {
            command,
            status: output.status,
            stderr: stderr_string(&output.stderr),
        });
    }

    Ok(output)
}

fn git_command_display(args: &[&str]) -> String {
    command_display(
        "git",
        &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
    )
}
