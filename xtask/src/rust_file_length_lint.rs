//! Audit Rust files under `crates/` and `xtask/` for the repo-wide line-count cap.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use crate::error::{Error, Result};

const MAX_LINES: usize = 300;
const BRANCH_DIFF_ARGS: &[&str] = &[
    "diff",
    "--name-only",
    "-z",
    "origin/main...HEAD",
    "--",
    "crates",
    "xtask",
];
const WORKTREE_STATUS_ARGS: &[&str] = &[
    "status",
    "--porcelain=v1",
    "-z",
    "--untracked-files=all",
    "--",
    "crates",
    "xtask",
];
const ALL_FILES_ARGS: &[&str] = &[
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-standard",
    "--",
    "crates",
    "xtask",
];

/// Selects which Rust files to audit for the file-length lint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RustFileLengthLintMode {
    /// Audit only files changed relative to `origin/main` or present in the working tree.
    ChangedFiles,
    /// Audit every tracked or untracked Rust file under `crates/` and `xtask/`.
    AllFiles,
}

/// Run the file-length lint against the selected file set.
pub(crate) fn run_rust_file_length_lint(
    workspace_root: &Path,
    mode: RustFileLengthLintMode,
) -> Result<()> {
    let mut violations = Vec::new();

    for path in rust_file_paths(workspace_root, mode)? {
        if let Err(error) = verify_rust_file_length(workspace_root, &path) {
            violations.push(error);
        }
    }

    if violations.is_empty() {
        return Ok(());
    }

    Err(combine_rust_file_length_violations(violations))
}

fn rust_file_paths(workspace_root: &Path, mode: RustFileLengthLintMode) -> Result<Vec<PathBuf>> {
    match mode {
        RustFileLengthLintMode::ChangedFiles => {
            let branch_diff = git_stdout_bytes(workspace_root, BRANCH_DIFF_ARGS)?;
            let worktree_status = git_stdout_bytes(workspace_root, WORKTREE_STATUS_ARGS)?;
            Ok(collect_candidate_rust_files(
                workspace_root,
                &branch_diff,
                &worktree_status,
            ))
        }
        RustFileLengthLintMode::AllFiles => {
            let all_files = git_stdout_bytes(workspace_root, ALL_FILES_ARGS)?;
            Ok(collect_candidate_rust_files(
                workspace_root,
                &all_files,
                &[],
            ))
        }
    }
}

fn collect_candidate_rust_files(
    workspace_root: &Path,
    branch_diff: &[u8],
    worktree_status: &[u8],
) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();

    add_nul_relative_rust_files(workspace_root, branch_diff, &mut files);
    add_status_relative_rust_files(workspace_root, worktree_status, &mut files);

    files.into_iter().collect()
}

fn add_nul_relative_rust_files(
    workspace_root: &Path,
    output: &[u8],
    files: &mut BTreeSet<PathBuf>,
) {
    for relative in output
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
    {
        add_relative_rust_file(workspace_root, relative, files);
    }
}

fn add_status_relative_rust_files(
    workspace_root: &Path,
    output: &[u8],
    files: &mut BTreeSet<PathBuf>,
) {
    let mut records = output
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty());
    while let Some(record) = records.next() {
        let Some(path) = record.get(3..) else {
            continue;
        };
        let rename_or_copy = record
            .get(..2)
            .is_some_and(|status| status.iter().any(|byte| matches!(*byte, b'R' | b'C')));
        add_relative_rust_file(workspace_root, path, files);
        if rename_or_copy {
            let _ = records.next();
        }
    }
}

fn add_relative_rust_file(workspace_root: &Path, relative: &[u8], files: &mut BTreeSet<PathBuf>) {
    let path = workspace_root.join(path_from_git_bytes(relative));
    if crate::filesystem::is_regular_file_without_symlink(&path)
        && path.extension() == Some(std::ffi::OsStr::new("rs"))
    {
        files.insert(path);
    }
}

#[cfg(unix)]
fn path_from_git_bytes(path: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(path.to_vec()))
}

#[cfg(not(unix))]
fn path_from_git_bytes(path: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(path).into_owned())
}

fn git_stdout_bytes(workspace_root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = match Command::new("git")
        .args(args)
        .current_dir(workspace_root)
        .output()
    {
        Ok(output) => output,
        Err(source) => {
            return Err(Error::RustFileLengthGitCommandStart {
                command: format!("git {}", args.join(" ")),
                source,
            });
        }
    };

    if !output.status.success() {
        return Err(Error::RustFileLengthGitCommandFailed {
            command: format!("git {}", args.join(" ")),
            status: output.status,
            stderr: stderr_string(&output.stderr),
        });
    }

    Ok(output.stdout)
}

fn verify_rust_file_length(workspace_root: &Path, path: &Path) -> Result<()> {
    let relative = path
        .strip_prefix(workspace_root)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf);
    let contents = fs::read_to_string(path).map_err(|source| Error::RustFileLengthReadFile {
        path: relative.clone(),
        source,
    })?;
    let line_count = contents.lines().count();

    if line_count > MAX_LINES {
        return Err(Error::RustFileTooLong {
            path: relative,
            line_count,
            max_lines: MAX_LINES,
        });
    }

    Ok(())
}

fn combine_rust_file_length_violations(violations: Vec<Error>) -> Error {
    let count = violations.len();
    let details = violations
        .into_iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    Error::RustFileLengthViolations { count, details }
}

fn stderr_string(stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return String::new();
    }

    format!(": {stderr}")
}

#[cfg(test)]
#[path = "_tests_/rust_file_length_lint_tests.rs"]
mod rust_file_length_lint_tests;
