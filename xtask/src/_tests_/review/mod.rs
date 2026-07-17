use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::review_codex::{codex_command_display, codex_environment_from};

use super::*;

mod command_tests;
mod context_tests;
mod generated_artifact_tests;
mod prompt_budget_tests;
mod runner_tests;

fn review_runner_with_changes(
    git_calls: Arc<Mutex<Vec<Vec<String>>>>,
    codex_prompts: Arc<Mutex<Vec<String>>>,
) -> Unimock {
    Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers_arc({
                let git_calls = git_calls.clone();
                Arc::new(move |_, _, args| {
                    git_calls.lock().expect("git calls").push(args_to_vec(args));
                    Ok(changed_git_output(args).to_owned())
                })
            }),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers_arc({
                let git_calls = git_calls.clone();
                Arc::new(move |_, _, args| {
                    git_calls.lock().expect("git calls").push(args_to_vec(args));
                    Ok(changed_git_bytes(args))
                })
            }),
        ReviewProcessRunnerMock::untracked_file_diff
            .next_call(matching!(_, _))
            .answers(&|_, _, relative_path| {
                Ok(UntrackedFileDiff::included(format!(
                    "### untracked file {relative_path}\n+untracked review notes\n"
                )))
            }),
        ReviewProcessRunnerMock::run_codex
            .each_call(matching!(_, _))
            .answers_arc({
                let codex_prompts = codex_prompts.clone();
                Arc::new(move |_, _, prompt| {
                    codex_prompts
                        .lock()
                        .expect("codex prompts")
                        .push(prompt.to_owned());
                    Ok(())
                })
            }),
    ))
}

fn review_runner_with_no_changes() -> Unimock {
    Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(String::new())),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(Vec::new())),
    ))
}

fn review_runner_with_untracked_only_change(codex_prompts: Arc<Mutex<Vec<String>>>) -> Unimock {
    Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(String::new())),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == UNTRACKED_FILES_ARGS {
                    return Ok(b"notes/new-review-case.md\0".to_vec());
                }

                Ok(Vec::new())
            }),
        ReviewProcessRunnerMock::untracked_file_diff
            .next_call(matching!(_, _))
            .answers(&|_, _, relative_path| {
                Ok(UntrackedFileDiff::included(format!(
                    "### untracked file {relative_path}\n+pub fn untracked_review_case() {{}}\n"
                )))
            }),
        ReviewProcessRunnerMock::run_codex
            .next_call(matching!(_, _))
            .answers_arc({
                let codex_prompts = codex_prompts.clone();
                Arc::new(move |_, _, prompt| {
                    codex_prompts
                        .lock()
                        .expect("codex prompts")
                        .push(prompt.to_owned());
                    Ok(())
                })
            }),
    ))
}

fn review_runner_with_codex_failure() -> Unimock {
    Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers(&|_, _, args| Ok(changed_git_output(args).to_owned())),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| Ok(changed_git_bytes(args))),
        ReviewProcessRunnerMock::untracked_file_diff
            .next_call(matching!(_, _))
            .answers(&|_, _, relative_path| {
                Ok(UntrackedFileDiff::included(format!(
                    "### untracked file {relative_path}\n+untracked review notes\n"
                )))
            }),
        ReviewProcessRunnerMock::run_codex
            .next_call(matching!(_, _))
            .answers(&|_, _, _| {
                Err(Error::ReviewCodexPromptWrite {
                    command: "codex exec".to_owned(),
                    source: io::Error::other("prompt pipe closed"),
                })
            }),
    ))
}

fn review_runner_with_sensitive_staged_change(codex_prompts: Arc<Mutex<Vec<String>>>) -> Unimock {
    Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_DIFF_ARGS {
                    return Ok("API_TOKEN=secret\n".to_owned());
                }

                Ok(String::new())
            }),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_CHANGED_FILES_ARGS {
                    return Ok(b"M\0.env.local\0M\0secrets/rotate.rs\0".to_vec());
                }

                Ok(Vec::new())
            }),
        ReviewProcessRunnerMock::run_codex
            .next_call(matching!(_, _))
            .answers_arc({
                let codex_prompts = codex_prompts.clone();
                Arc::new(move |_, _, prompt| {
                    codex_prompts
                        .lock()
                        .expect("codex prompts")
                        .push(prompt.to_owned());
                    Ok(())
                })
            }),
    ))
}

fn review_runner_with_mixed_sensitive_staged_change(
    git_calls: Arc<Mutex<Vec<Vec<String>>>>,
    codex_prompts: Arc<Mutex<Vec<String>>>,
) -> Unimock {
    Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers_arc({
                let git_calls = git_calls.clone();
                Arc::new(move |_, _, args| {
                    git_calls.lock().expect("git calls").push(args_to_vec(args));
                    Ok(String::new())
                })
            }),
        ReviewProcessRunnerMock::git_stdout_dynamic
            .next_call(matching!(_, _))
            .answers_arc({
                let git_calls = git_calls.clone();
                Arc::new(move |_, _, args| {
                    git_calls.lock().expect("git calls").push(args.to_vec());
                    Ok("diff --git a/src/lib.rs b/src/lib.rs\n+pub fn changed() {}\n".to_owned())
                })
            }),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_CHANGED_FILES_ARGS {
                    return Ok(b"M\0.env.local\0M\0src/lib.rs\0".to_vec());
                }

                Ok(Vec::new())
            }),
        ReviewProcessRunnerMock::run_codex
            .next_call(matching!(_, _))
            .answers_arc({
                let codex_prompts = codex_prompts.clone();
                Arc::new(move |_, _, prompt| {
                    codex_prompts
                        .lock()
                        .expect("codex prompts")
                        .push(prompt.to_owned());
                    Ok(())
                })
            }),
    ))
}

fn review_runner_with_tracked_token_source_change(
    codex_prompts: Arc<Mutex<Vec<String>>>,
) -> Unimock {
    Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_DIFF_ARGS {
                    return Ok("diff includes tracked token source\n".to_owned());
                }

                Ok(String::new())
            }),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_CHANGED_FILES_ARGS {
                    return Ok(b"M\0crates/tm-control-plane/src/auth/token.rs\0".to_vec());
                }

                Ok(Vec::new())
            }),
        ReviewProcessRunnerMock::run_codex
            .next_call(matching!(_, _))
            .answers_arc({
                let codex_prompts = codex_prompts.clone();
                Arc::new(move |_, _, prompt| {
                    codex_prompts
                        .lock()
                        .expect("codex prompts")
                        .push(prompt.to_owned());
                    Ok(())
                })
            }),
    ))
}

fn changed_git_output(args: &[&str]) -> &'static str {
    if args == BRANCH_DIFF_ARGS {
        return "branch diff body\n";
    }
    if args == STAGED_DIFF_ARGS {
        return "staged diff body\n";
    }
    if args == UNSTAGED_DIFF_ARGS {
        return "unstaged diff body\n";
    }

    ""
}

fn changed_git_bytes(args: &[&str]) -> Vec<u8> {
    if args == BRANCH_CHANGED_FILES_ARGS {
        return b"M\0README.md\0M\0xtask/src/review.rs\0".to_vec();
    }
    if args == STAGED_CHANGED_FILES_ARGS {
        return b"M\0plans/ai-xtask-review-command.md\0".to_vec();
    }
    if args == UNSTAGED_CHANGED_FILES_ARGS {
        return b"M\0xtask/src/review.rs\0".to_vec();
    }
    if args == UNTRACKED_FILES_ARGS {
        return b"notes/new-review-case.md\0".to_vec();
    }

    Vec::new()
}

fn args_to_vec(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}
