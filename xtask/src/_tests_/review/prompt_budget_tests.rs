use crate::review_prompt::{ReviewContext, build_review_prompt};

#[test]
fn review_prompt_caps_large_file_lists_and_diff_sections() {
    let reviewable_files = (0..2_000)
        .map(|index| format!("crates/tm-large-crate/src/file-{index:04}.rs"))
        .collect::<Vec<_>>();
    let branch_diff = format!(
        "diff --git a/early.rs b/early.rs\n{}\ndiff --git a/late.rs b/late.rs\n+late change\n",
        "+early change\n".repeat(30_000)
    );
    let context = ReviewContext {
        repository_path: "/repo".to_owned(),
        reviewable_files,
        omitted_paths: Vec::new(),
        omitted_paths_additional_count: 0,
        branch_diff,
        staged_diff: String::new(),
        unstaged_diff: String::new(),
        untracked_diff: String::new(),
    };

    let prompt = build_review_prompt(&context);

    assert!(prompt.len() < 900_000);
    assert!(prompt.contains("- Reviewable files: 2000"));
    assert!(prompt.contains("- Reviewable files listed: 512"));
    assert!(prompt.contains("- Additional reviewable files not listed: 1488"));
    assert!(prompt.contains("file-0000.rs"));
    assert!(!prompt.contains("file-1999.rs"));
    assert!(prompt.contains("truncated"));
    assert!(prompt.contains("Codex input limits"));
    assert!(prompt.contains("+late change"));
}
