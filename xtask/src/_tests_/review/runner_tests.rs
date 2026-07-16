use super::*;

#[test]
fn run_review_collects_all_local_change_sources() {
    let git_calls = Arc::new(Mutex::new(Vec::new()));
    let codex_prompts = Arc::new(Mutex::new(Vec::new()));
    let runner = review_runner_with_changes(git_calls.clone(), codex_prompts.clone());

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();

    assert_eq!(
        git_calls.lock().expect("git calls").clone(),
        vec![
            args_to_vec(BRANCH_CHANGED_FILES_ARGS),
            args_to_vec(STAGED_CHANGED_FILES_ARGS),
            args_to_vec(UNSTAGED_CHANGED_FILES_ARGS),
            args_to_vec(UNTRACKED_FILES_ARGS),
            args_to_vec(BRANCH_DIFF_ARGS),
            args_to_vec(STAGED_DIFF_ARGS),
            args_to_vec(UNSTAGED_DIFF_ARGS),
        ]
    );

    let prompts = codex_prompts.lock().expect("codex prompts");
    assert_eq!(prompts.len(), 1);
    let prompt = &prompts[0];
    assert!(prompt.contains("README.md"));
    assert!(prompt.contains("plans/ai-xtask-review-command.md"));
    assert!(prompt.contains("xtask/src/review.rs"));
    assert!(prompt.contains("notes/new-review-case.md"));
    assert!(prompt.contains("branch diff body"));
    assert!(prompt.contains("staged diff body"));
    assert!(prompt.contains("unstaged diff body"));
    assert!(prompt.contains("untracked file synthetic diffs"));
    assert!(prompt.contains("+untracked review notes"));
    assert!(
        git_calls
            .lock()
            .expect("git calls")
            .iter()
            .filter(|args| args.first().is_some_and(|arg| arg == "diff"))
            .all(|args| args.contains(&"--no-ext-diff".to_owned())
                && args.contains(&"--no-textconv".to_owned()))
    );
}

#[test]
fn run_review_skips_codex_when_there_are_no_changes() {
    let runner = review_runner_with_no_changes();

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();
}

#[test]
fn run_review_invokes_codex_for_untracked_only_changes() {
    let codex_prompts = Arc::new(Mutex::new(Vec::new()));
    let runner = review_runner_with_untracked_only_change(codex_prompts.clone());

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();

    let prompts = codex_prompts.lock().expect("codex prompts");
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains("notes/new-review-case.md"));
    assert!(prompts[0].contains("pub fn untracked_review_case() {}"));
}

#[test]
fn tracked_sensitive_paths_omit_diff_body() {
    let codex_prompts = Arc::new(Mutex::new(Vec::new()));
    let runner = review_runner_with_sensitive_staged_change(codex_prompts.clone());

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();

    let prompts = codex_prompts.lock().expect("codex prompts");
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains(".env.local"));
    assert!(prompts[0].contains("<omitted_paths_json"));
    assert!(prompts[0].contains("\".env.local\""));
    assert!(prompts[0].contains(
        "<reviewable_files_json encoding=\"json-with-angle-brackets-escaped\">\n[]\n</reviewable_files_json>"
    ));
    assert!(prompts[0].contains("tracked diff body omitted"));
    assert!(!prompts[0].contains("API_TOKEN=secret"));
}

#[test]
fn mixed_sensitive_tracked_paths_keep_reviewable_diff_body() {
    let git_calls = Arc::new(Mutex::new(Vec::new()));
    let codex_prompts = Arc::new(Mutex::new(Vec::new()));
    let runner =
        review_runner_with_mixed_sensitive_staged_change(git_calls.clone(), codex_prompts.clone());

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();

    let prompts = codex_prompts.lock().expect("codex prompts");
    assert_eq!(prompts.len(), 1);
    let prompt = &prompts[0];
    assert!(prompt.contains("\".env.local\""));
    assert!(prompt.contains("diff --git a/src/lib.rs b/src/lib.rs"));
    assert!(prompt.contains("omitted path changes omitted from this diff section"));
    assert!(!prompt.contains("API_TOKEN=secret"));

    let calls = git_calls.lock().expect("git calls");
    let filtered_diff_args = calls
        .iter()
        .find(|args| args.contains(&":(literal)src/lib.rs".to_owned()))
        .expect("filtered tracked diff call");
    assert!(filtered_diff_args.contains(&"--no-ext-diff".to_owned()));
    assert!(filtered_diff_args.contains(&"--no-textconv".to_owned()));
    assert!(!filtered_diff_args.contains(&":(literal).env.local".to_owned()));
}

#[test]
fn sensitive_tracked_renames_omit_both_sides_from_filtered_diff() {
    let git_calls = Arc::new(Mutex::new(Vec::new()));
    let codex_prompts = Arc::new(Mutex::new(Vec::new()));
    let runner = Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(String::new())),
        ReviewProcessRunnerMock::git_stdout_dynamic
            .next_call(matching!(_, _))
            .answers_arc({
                let git_calls = git_calls.clone();
                Arc::new(move |_, _, args| {
                    git_calls.lock().expect("git calls").push(args.to_vec());
                    Ok("diff --git a/src/main.rs b/src/main.rs\n+pub fn changed() {}\n".to_owned())
                })
            }),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_CHANGED_FILES_ARGS {
                    return Ok(b"R100\0.env.local\0src/lib.rs\0M\0src/main.rs\0".to_vec());
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
    ));

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();

    let prompt = codex_prompts.lock().expect("codex prompts").remove(0);
    assert!(prompt.contains("\".env.local\""));
    assert!(prompt.contains("\"src/lib.rs\""));
    assert!(prompt.contains("diff --git a/src/main.rs b/src/main.rs"));
    assert!(prompt.contains("omitted path changes omitted from this diff section"));

    let calls = git_calls.lock().expect("git calls");
    let filtered_diff_args = calls.first().expect("filtered tracked diff call");
    assert!(filtered_diff_args.contains(&":(literal)src/main.rs".to_owned()));
    assert!(!filtered_diff_args.contains(&":(literal).env.local".to_owned()));
    assert!(!filtered_diff_args.contains(&":(literal)src/lib.rs".to_owned()));
}

#[test]
fn globally_omitted_sensitive_rename_path_cannot_reenter_later_diff_section() {
    let git_calls = Arc::new(Mutex::new(Vec::new()));
    let codex_prompts = Arc::new(Mutex::new(Vec::new()));
    let runner = Unimock::new((
        ReviewProcessRunnerMock::git_stdout
            .each_call(matching!(_, _))
            .answers_arc({
                let git_calls = git_calls.clone();
                Arc::new(move |_, _, args| {
                    git_calls.lock().expect("git calls").push(args_to_vec(args));
                    if args == UNSTAGED_DIFF_ARGS {
                        return Ok("API_TOKEN=secret\n".to_owned());
                    }

                    Ok(String::new())
                })
            }),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_CHANGED_FILES_ARGS {
                    return Ok(b"R100\0.env.local\0src/lib.rs\0".to_vec());
                }
                if args == UNSTAGED_CHANGED_FILES_ARGS {
                    return Ok(b"M\0src/lib.rs\0".to_vec());
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
    ));

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();

    let prompt = codex_prompts.lock().expect("codex prompts").remove(0);
    assert!(prompt.contains("\".env.local\""));
    assert!(prompt.contains("\"src/lib.rs\""));
    assert!(prompt.contains("tracked diff body omitted"));
    assert!(!prompt.contains("API_TOKEN=secret"));

    let calls = git_calls.lock().expect("git calls");
    assert!(!calls.contains(&args_to_vec(UNSTAGED_DIFF_ARGS)));
}

#[test]
fn tracked_source_paths_with_sensitive_words_remain_reviewable() {
    let codex_prompts = Arc::new(Mutex::new(Vec::new()));
    let runner = review_runner_with_tracked_token_source_change(codex_prompts.clone());

    run_review_with_runner(&runner, Path::new("/repo")).unwrap();

    let prompts = codex_prompts.lock().expect("codex prompts");
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains("crates/tm-control-plane/src/auth/token.rs"));
    assert!(prompts[0].contains("diff includes tracked token source"));
    assert!(prompts[0].contains(
        "<omitted_paths_json encoding=\"json-with-angle-brackets-escaped\">\n[]\n</omitted_paths_json>"
    ));
}

#[test]
fn run_review_propagates_codex_failures() {
    let runner = review_runner_with_codex_failure();

    let error = run_review_with_runner(&runner, Path::new("/repo")).unwrap_err();

    assert!(matches!(error, Error::ReviewCodexPromptWrite { .. }));
}
