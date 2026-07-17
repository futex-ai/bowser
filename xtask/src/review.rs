//! Local AI review command for branch and working-tree changes.

use std::collections::BTreeSet;
use std::path::Path;

use crate::error::{Error, Result};
use crate::review_codex::run_codex_command;
use crate::review_git;
use crate::review_omitted_paths::{
    MAX_OMITTED_UNTRACKED_PATHS, OmittedPathReport, omit_current_and_remaining,
};
use crate::review_paths::{GitPath, parse_git_paths};
use crate::review_prompt::{BASE_REF, ReviewContext, build_review_prompt};
use crate::review_sensitive_paths::is_omitted_untracked_relative_path;
use crate::review_tracked::{
    add_reviewable_tracked_files, add_sensitive_tracked_paths, collect_tracked_diff,
    parse_tracked_path_changes,
};
use crate::review_untracked::{UntrackedFileDiff, build_untracked_file_diff};

const BRANCH_CHANGED_FILES_ARGS: &[&str] = &[
    "diff",
    "--no-ext-diff",
    "--no-textconv",
    "--find-renames",
    "--name-status",
    "-z",
    "origin/main...HEAD",
];
const STAGED_CHANGED_FILES_ARGS: &[&str] = &[
    "diff",
    "--no-ext-diff",
    "--no-textconv",
    "--find-renames",
    "--name-status",
    "-z",
    "--cached",
];
const UNSTAGED_CHANGED_FILES_ARGS: &[&str] = &[
    "diff",
    "--no-ext-diff",
    "--no-textconv",
    "--find-renames",
    "--name-status",
    "-z",
];
const UNTRACKED_FILES_ARGS: &[&str] = &["ls-files", "--others", "--exclude-standard", "-z"];

const BRANCH_DIFF_ARGS: &[&str] = &[
    "diff",
    "--no-ext-diff",
    "--no-textconv",
    "--find-renames",
    "origin/main...HEAD",
];
const STAGED_DIFF_ARGS: &[&str] = &[
    "diff",
    "--no-ext-diff",
    "--no-textconv",
    "--find-renames",
    "--cached",
];
const UNSTAGED_DIFF_ARGS: &[&str] = &["diff", "--no-ext-diff", "--no-textconv", "--find-renames"];
const MAX_UNTRACKED_FILES: usize = 64;
const MAX_UNTRACKED_DIFF_BYTES: usize = 512 * 1024;

#[cfg_attr(test, unimock::unimock(api = ReviewProcessRunnerMock))]
pub(crate) trait ReviewProcessRunner {
    fn git_stdout(&self, workspace_root: &Path, args: &[&str]) -> Result<String>;
    fn git_stdout_dynamic(&self, workspace_root: &Path, args: &[String]) -> Result<String>;
    fn git_stdout_bytes(&self, workspace_root: &Path, args: &[&str]) -> Result<Vec<u8>>;
    fn untracked_file_diff(
        &self,
        workspace_root: &Path,
        relative_path: &str,
    ) -> Result<UntrackedFileDiff>;
    fn run_codex(&self, workspace_root: &Path, prompt: &str) -> Result<()>;
}

struct RealReviewProcessRunner;

impl ReviewProcessRunner for RealReviewProcessRunner {
    fn git_stdout(&self, workspace_root: &Path, args: &[&str]) -> Result<String> {
        review_git::git_stdout(workspace_root, args)
    }

    fn git_stdout_dynamic(&self, workspace_root: &Path, args: &[String]) -> Result<String> {
        review_git::git_stdout_dynamic(workspace_root, args)
    }

    fn git_stdout_bytes(&self, workspace_root: &Path, args: &[&str]) -> Result<Vec<u8>> {
        review_git::git_stdout_bytes(workspace_root, args)
    }

    fn run_codex(&self, workspace_root: &Path, prompt: &str) -> Result<()> {
        run_codex_command(workspace_root, prompt)
    }

    fn untracked_file_diff(
        &self,
        workspace_root: &Path,
        relative_path: &str,
    ) -> Result<UntrackedFileDiff> {
        build_untracked_file_diff(workspace_root, relative_path)
    }
}

pub(crate) fn run_review(workspace_root: &Path) -> Result<()> {
    let runner = RealReviewProcessRunner;
    run_review_with_runner(&runner, workspace_root)
}

fn run_review_with_runner(runner: &dyn ReviewProcessRunner, workspace_root: &Path) -> Result<()> {
    let context = collect_review_context(runner, workspace_root)?;

    if !context.has_changes() {
        println!("No changes to review relative to {BASE_REF} or in the working tree.");
        return Ok(());
    }

    runner.run_codex(workspace_root, &build_review_prompt(&context))
}

fn collect_review_context(
    runner: &dyn ReviewProcessRunner,
    workspace_root: &Path,
) -> Result<ReviewContext> {
    let branch_changes = parse_tracked_path_changes(
        &runner.git_stdout_bytes(workspace_root, BRANCH_CHANGED_FILES_ARGS)?,
    );
    let staged_changes = parse_tracked_path_changes(
        &runner.git_stdout_bytes(workspace_root, STAGED_CHANGED_FILES_ARGS)?,
    );
    let unstaged_changes = parse_tracked_path_changes(
        &runner.git_stdout_bytes(workspace_root, UNSTAGED_CHANGED_FILES_ARGS)?,
    );
    let untracked_files =
        parse_git_paths(&runner.git_stdout_bytes(workspace_root, UNTRACKED_FILES_ARGS)?);

    let mut tracked_omitted_paths = BTreeSet::new();
    add_sensitive_tracked_paths(&branch_changes, &mut tracked_omitted_paths);
    add_sensitive_tracked_paths(&staged_changes, &mut tracked_omitted_paths);
    add_sensitive_tracked_paths(&unstaged_changes, &mut tracked_omitted_paths);

    let mut reviewable_files = BTreeSet::new();
    add_reviewable_tracked_files(
        &branch_changes,
        &mut reviewable_files,
        &tracked_omitted_paths,
    );
    add_reviewable_tracked_files(
        &staged_changes,
        &mut reviewable_files,
        &tracked_omitted_paths,
    );
    add_reviewable_tracked_files(
        &unstaged_changes,
        &mut reviewable_files,
        &tracked_omitted_paths,
    );

    let mut omitted_paths = tracked_omitted_paths.clone();
    let untracked_diff = collect_untracked_diffs(runner, workspace_root, &untracked_files)?;
    if untracked_diff.hidden_omitted_count > 0 {
        return Err(Error::ReviewOmittedPathReportExceeded {
            cap: MAX_OMITTED_UNTRACKED_PATHS,
            hidden_count: untracked_diff.hidden_omitted_count,
        });
    }
    for path in &untracked_diff.reviewable_paths {
        if !omitted_paths.contains(path) {
            reviewable_files.insert(path.to_owned());
        }
    }
    for path in &untracked_diff.omitted_paths {
        omitted_paths.insert(path.to_owned());
    }
    reviewable_files.retain(|path| !omitted_paths.contains(path));

    let branch_diff = collect_tracked_diff(
        runner,
        workspace_root,
        BRANCH_DIFF_ARGS,
        &branch_changes,
        &omitted_paths,
    )?;
    let staged_diff = collect_tracked_diff(
        runner,
        workspace_root,
        STAGED_DIFF_ARGS,
        &staged_changes,
        &omitted_paths,
    )?;
    let unstaged_diff = collect_tracked_diff(
        runner,
        workspace_root,
        UNSTAGED_DIFF_ARGS,
        &unstaged_changes,
        &omitted_paths,
    )?;

    Ok(ReviewContext {
        repository_path: workspace_root.display().to_string(),
        reviewable_files: reviewable_files.into_iter().collect(),
        omitted_paths: omitted_paths.into_iter().collect(),
        omitted_paths_additional_count: untracked_diff.hidden_omitted_count,
        branch_diff,
        staged_diff,
        unstaged_diff,
        untracked_diff: untracked_diff.diff,
    })
}

struct UntrackedDiffs {
    diff: String,
    reviewable_paths: Vec<String>,
    omitted_paths: Vec<String>,
    hidden_omitted_count: usize,
}

fn collect_untracked_diffs(
    runner: &dyn ReviewProcessRunner,
    workspace_root: &Path,
    untracked_files: &[GitPath],
) -> Result<UntrackedDiffs> {
    let mut diffs = Vec::new();
    let mut omitted_paths = OmittedPathReport::new(MAX_OMITTED_UNTRACKED_PATHS);
    let mut reviewable_paths = Vec::new();
    let mut total_bytes = 0usize;
    let mut included_files = 0;

    for file in untracked_files {
        if included_files >= MAX_UNTRACKED_FILES {
            omit_current_and_remaining(untracked_files, file, &mut omitted_paths);
            break;
        }

        if is_omitted_untracked_relative_path(file.display()) {
            diffs.push(format!(
                "### untracked file {}\n[skipped: sensitive path]\n",
                file.display()
            ));
            omitted_paths.push(file.display().to_owned());
            included_files += 1;
            continue;
        }

        let file_diff = match file.readable() {
            Some(file) => runner.untracked_file_diff(workspace_root, file)?,
            None => UntrackedFileDiff::skipped(format!(
                "### untracked file {}\n[skipped: non-UTF-8 path]\n",
                file.display()
            )),
        };
        let next_total = total_bytes.saturating_add(file_diff.text().len());
        if next_total > MAX_UNTRACKED_DIFF_BYTES {
            omit_current_and_remaining(untracked_files, file, &mut omitted_paths);
            break;
        }

        total_bytes = next_total;
        included_files += 1;
        if file_diff.is_skipped() {
            omitted_paths.push(file.display().to_owned());
        } else {
            reviewable_paths.push(file.display().to_owned());
        }
        diffs.push(file_diff.into_text());
    }

    let hidden_omitted_count = omitted_paths.hidden_count();
    omitted_paths.append_summary(&mut diffs);

    Ok(UntrackedDiffs {
        diff: diffs.join("\n"),
        reviewable_paths,
        omitted_paths: omitted_paths.into_paths(),
        hidden_omitted_count,
    })
}

#[cfg(test)]
#[path = "_tests_/review/mod.rs"]
mod review_tests;
