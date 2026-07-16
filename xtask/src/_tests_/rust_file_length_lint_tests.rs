use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

static TEMP_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn collects_only_changed_rust_files() {
    let workspace = temp_workspace();
    fs::create_dir_all(workspace.path().join("crates/demo/src")).unwrap();
    fs::write(
        workspace.path().join("crates/demo/src/lib.rs"),
        "fn demo() {}\n",
    )
    .unwrap();
    fs::write(workspace.path().join("xtask/src/main.rs"), "fn main() {}\n").unwrap();

    let files = collect_candidate_rust_files(
        workspace.path(),
        b"crates/demo/src/lib.rs\0README.md\0",
        b" M xtask/src/main.rs\0?? crates/demo/src/notes.txt\0",
    );

    assert_eq!(
        files,
        vec![
            workspace.path().join("crates/demo/src/lib.rs"),
            workspace.path().join("xtask/src/main.rs"),
        ]
    );
}

#[cfg(unix)]
#[test]
fn changed_file_parsing_preserves_newlines_and_renames() {
    let workspace = temp_workspace();
    let newline_path = workspace.path().join("crates/demo/src/line\nbreak.rs");
    let renamed_path = workspace.path().join("xtask/src/renamed.rs");
    write_rust_file(&newline_path, 1);
    write_rust_file(&renamed_path, 1);

    let files = collect_candidate_rust_files(
        workspace.path(),
        b"crates/demo/src/line\nbreak.rs\0",
        b"R  xtask/src/renamed.rs\0xtask/src/old.rs\0",
    );

    assert_eq!(files, vec![newline_path, renamed_path]);
}

#[test]
fn all_mode_audits_tracked_rust_files() {
    let workspace = temp_workspace();
    write_rust_file(&workspace.path().join("crates/demo/src/lib.rs"), 301);
    write_rust_file(&workspace.path().join("xtask/src/main.rs"), 302);
    init_git_repo(workspace.path());

    let error =
        run_rust_file_length_lint(workspace.path(), RustFileLengthLintMode::AllFiles).unwrap_err();

    assert!(matches!(
        error,
        Error::RustFileLengthViolations { count: 2, .. }
    ));
}

#[cfg(unix)]
#[test]
fn all_mode_audits_tracked_rust_filename_with_newline() {
    let workspace = temp_workspace();
    write_rust_file(
        &workspace.path().join("crates/demo/src/line\nbreak.rs"),
        301,
    );
    init_git_repo(workspace.path());

    let error =
        run_rust_file_length_lint(workspace.path(), RustFileLengthLintMode::AllFiles).unwrap_err();

    assert!(matches!(
        error,
        Error::RustFileLengthViolations { count: 1, .. }
    ));
}

#[test]
fn all_mode_snapshots_all_violations() {
    let workspace = temp_workspace();
    write_rust_file(&workspace.path().join("crates/demo/src/lib.rs"), 301);
    write_rust_file(&workspace.path().join("xtask/src/main.rs"), 302);
    init_git_repo(workspace.path());

    let error =
        run_rust_file_length_lint(workspace.path(), RustFileLengthLintMode::AllFiles).unwrap_err();

    insta::assert_snapshot!("all_mode_reports_all_violations", error.to_string());
}

#[test]
fn passes_when_file_stays_within_limit() {
    let workspace = temp_workspace();
    let path = workspace.path().join("xtask/src/check.rs");
    write_rust_file(&path, 300);

    verify_rust_file_length(workspace.path(), &path).unwrap();
}

#[test]
fn rejects_file_over_limit() {
    let workspace = temp_workspace();
    let path = workspace.path().join("xtask/src/check.rs");
    write_rust_file(&path, 301);

    let error = verify_rust_file_length(workspace.path(), &path).unwrap_err();

    assert!(matches!(
        error,
        Error::RustFileTooLong {
            line_count: 301,
            max_lines: MAX_LINES,
            ..
        }
    ));
}

#[test]
fn snapshots_file_too_long_error() {
    let workspace = temp_workspace();
    let path = workspace.path().join("xtask/src/check.rs");
    write_rust_file(&path, 301);

    let error = verify_rust_file_length(workspace.path(), &path).unwrap_err();

    insta::assert_snapshot!("rust_file_too_long_error", error.to_string());
}

struct TempWorkspace {
    path: std::path::PathBuf,
}

impl TempWorkspace {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn temp_workspace() -> TempWorkspace {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique = TEMP_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "xtask-rust-file-length-{}-{nanos}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(path.join("xtask/src")).unwrap();
    TempWorkspace { path }
}

fn write_rust_file(path: &Path, total_lines: usize) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    let mut contents = String::new();
    for index in 0..total_lines {
        contents.push_str(&format!("fn line_{index}() {{}}\n"));
    }

    fs::write(path, contents).unwrap();
}

fn init_git_repo(path: &Path) {
    for args in [
        &["init"][..],
        &["config", "user.name", "xtask-test"],
        &["config", "user.email", "xtask-test@example.com"],
        &["add", "."],
        &["commit", "-m", "init"],
    ] {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .unwrap();

        assert!(
            status.success(),
            "git command failed: git {}",
            args.join(" ")
        );
    }
}
