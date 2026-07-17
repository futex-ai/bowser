use super::*;

#[test]
fn generated_mockup_artifacts_are_omitted_from_review_diff() {
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
                    Ok(
                        "diff --git a/docs/mockups/src/pages/app/chat/index.source.ts b/docs/mockups/src/pages/app/chat/index.source.ts\n+module.exports.source = () => \"\";\n\
diff --git a/docs/mockups/src/core.ts b/docs/mockups/src/core.ts\n+export {}\n"
                            .to_owned(),
                    )
                })
            }),
        ReviewProcessRunnerMock::git_stdout_bytes
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                if args == STAGED_CHANGED_FILES_ARGS {
                    return Ok(
                        b"M\0docs/mockups/app/chat/index.html\0M\0docs/mockups/src/pages/app/chat/index.source.ts\0M\0docs/mockups/src/core.ts\0"
                            .to_vec(),
                    );
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
    assert!(prompt.contains("\"docs/mockups/app/chat/index.html\""));
    assert!(prompt.contains("\"docs/mockups/src/pages/app/chat/index.source.ts\""));
    assert!(prompt.contains("diff --git a/docs/mockups/src/core.ts"));
    assert!(prompt.contains("diff --git a/docs/mockups/src/pages/app/chat/index.source.ts"));
    assert!(prompt.contains("omitted path changes omitted from this diff section"));
    assert!(!prompt.contains("docs/mockups/app/chat/index.html b/docs/mockups"));

    let calls = git_calls.lock().expect("git calls");
    let filtered_diff_args = calls.first().expect("filtered tracked diff call");
    assert!(filtered_diff_args.contains(&":(literal)docs/mockups/src/core.ts".to_owned()));
    assert!(
        filtered_diff_args
            .contains(&":(literal)docs/mockups/src/pages/app/chat/index.source.ts".to_owned())
    );
    assert!(!filtered_diff_args.contains(&":(literal)docs/mockups/app/chat/index.html".to_owned()));
}
