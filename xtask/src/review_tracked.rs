//! Rename-aware tracked diff collection for local AI reviews.

use std::collections::BTreeSet;
use std::path::Path;

use crate::error::Result;
use crate::review::ReviewProcessRunner;
use crate::review_paths::GitPath;
use crate::review_sensitive_paths::is_omitted_tracked_relative_path;

/// A single tracked Git change and the paths Git reports for it.
pub(crate) struct TrackedPathChange {
    paths: Vec<GitPath>,
}

impl TrackedPathChange {
    fn new(paths: Vec<GitPath>) -> Self {
        Self { paths }
    }

    fn paths(&self) -> impl Iterator<Item = &GitPath> {
        self.paths.iter()
    }

    fn has_sensitive_path(&self) -> bool {
        self.paths()
            .any(|path| is_omitted_tracked_relative_path(path.display()))
    }
}

/// Parse `git diff --name-status -z` output, including rename old/new pairs.
pub(crate) fn parse_tracked_path_changes(output: &[u8]) -> Vec<TrackedPathChange> {
    let tokens = output
        .split(|byte| *byte == 0)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let mut changes = Vec::new();
    let mut index = 0;

    while let Some(status) = tokens.get(index) {
        index += 1;
        let path_count = tracked_status_path_count(status);
        let mut paths = Vec::new();
        for _ in 0..path_count {
            let Some(path) = tokens.get(index) else {
                break;
            };
            index += 1;
            paths.push(GitPath::from_bytes(path));
        }
        if !paths.is_empty() {
            changes.push(TrackedPathChange::new(paths));
        }
    }

    changes
}

/// Add paths from sensitive tracked changes to the global omission set.
pub(crate) fn add_sensitive_tracked_paths(
    changes: &[TrackedPathChange],
    omitted_paths: &mut BTreeSet<String>,
) {
    for change in changes {
        if change.has_sensitive_path() {
            for path in change.paths() {
                omitted_paths.insert(path.display().to_owned());
            }
        }
    }
}

/// Add reviewable tracked paths to prompt file lists.
pub(crate) fn add_reviewable_tracked_files(
    changes: &[TrackedPathChange],
    reviewable_files: &mut BTreeSet<String>,
    omitted_paths: &BTreeSet<String>,
) {
    for change in changes {
        if change.has_sensitive_path() || change.has_omitted_path(omitted_paths) {
            continue;
        }

        for path in change.paths() {
            reviewable_files.insert(path.display().to_owned());
        }
    }
}

pub(crate) fn collect_tracked_diff(
    runner: &dyn ReviewProcessRunner,
    workspace_root: &Path,
    args: &[&str],
    changes: &[TrackedPathChange],
    omitted_paths: &BTreeSet<String>,
) -> Result<String> {
    let mut has_omitted_change = false;
    let mut reviewable_paths = BTreeSet::new();

    for change in changes {
        if change.has_sensitive_path() || change.has_omitted_path(omitted_paths) {
            has_omitted_change = true;
        } else {
            reviewable_paths.extend(change.paths().map(|path| path.display().to_owned()));
        }
    }

    if !has_omitted_change {
        return runner.git_stdout(workspace_root, args);
    }

    if reviewable_paths.is_empty() {
        return Ok(
            "[skipped: omitted path changes detected; tracked diff body omitted]\n".to_owned(),
        );
    }

    let reviewable_paths = reviewable_paths
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut diff = runner.git_stdout_dynamic(
        workspace_root,
        &tracked_diff_args_for_paths(args, &reviewable_paths),
    )?;
    if !diff.ends_with('\n') {
        diff.push('\n');
    }
    diff.push_str("[skipped: omitted path changes omitted from this diff section]\n");

    Ok(diff)
}

impl TrackedPathChange {
    fn has_omitted_path(&self, omitted_paths: &BTreeSet<String>) -> bool {
        self.paths()
            .any(|path| omitted_paths.contains(path.display()))
    }
}

fn tracked_status_path_count(status: &[u8]) -> usize {
    match status.first().copied() {
        Some(b'R' | b'C') => 2,
        _ => 1,
    }
}

fn tracked_diff_args_for_paths(args: &[&str], reviewable_paths: &[&str]) -> Vec<String> {
    let mut args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    args.push("--".to_owned());
    args.extend(
        reviewable_paths
            .iter()
            .map(|path| format!(":(literal){path}")),
    );
    args
}
