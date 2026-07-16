use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::review_sensitive_paths::{
    is_omitted_tracked_relative_path, is_omitted_untracked_relative_path,
    is_sensitive_tracked_relative_path,
};

use super::*;

static TEMP_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
#[test]
fn skips_untracked_symlinks_without_reading_target() {
    let workspace = temp_workspace();
    let outside_target = workspace.parent().join("target.txt");
    fs::write(&outside_target, "do-not-include-this-target").unwrap();
    std::os::unix::fs::symlink(&outside_target, workspace.path().join("linked-target.txt"))
        .unwrap();

    let diff = build_untracked_file_diff(workspace.path(), "linked-target.txt").unwrap();

    assert!(diff.text().contains("[skipped: unreadable file]"));
    assert!(!diff.text().contains("do-not-include-this-target"));
    assert!(diff.is_skipped());
}

#[cfg(unix)]
#[test]
fn skips_untracked_paths_with_symlinked_parent() {
    let workspace = temp_workspace();
    let outside_dir = workspace.parent().join("outside-dir");
    fs::create_dir_all(&outside_dir).unwrap();
    fs::write(outside_dir.join("target.txt"), "do-not-include-this-target").unwrap();
    std::os::unix::fs::symlink(&outside_dir, workspace.path().join("linked-dir")).unwrap();

    let diff = build_untracked_file_diff(workspace.path(), "linked-dir/target.txt").unwrap();

    assert!(diff.text().contains("[skipped: unreadable file]"));
    assert!(!diff.text().contains("do-not-include-this-target"));
    assert!(diff.is_skipped());
}

#[cfg(unix)]
#[test]
fn skips_binary_untracked_files() {
    let workspace = temp_workspace();
    fs::write(workspace.path().join("binary.bin"), [0, 159, 146, 150]).unwrap();

    let diff = build_untracked_file_diff(workspace.path(), "binary.bin").unwrap();

    assert!(diff.text().contains("[skipped: binary or non-UTF-8 file]"));
    assert!(diff.is_skipped());
}

#[test]
fn skips_sensitive_untracked_paths_without_reading_contents() {
    let workspace = temp_workspace();
    fs::write(workspace.path().join(".env.local"), "API_TOKEN=secret").unwrap();

    let diff = build_untracked_file_diff(workspace.path(), ".env.local").unwrap();

    assert!(diff.text().contains("[skipped: sensitive path]"));
    assert!(!diff.text().contains("API_TOKEN"));
    assert!(!diff.text().contains("secret"));
    assert!(diff.is_skipped());
}

#[test]
fn tracked_sensitive_config_filenames_with_extensions_are_sensitive() {
    assert!(is_sensitive_tracked_relative_path("config/secrets.yaml"));
    assert!(is_sensitive_tracked_relative_path("credentials.json"));
    assert!(is_sensitive_tracked_relative_path("private-key.json"));
    assert!(is_sensitive_tracked_relative_path("tokens.local.json"));
    assert!(is_sensitive_tracked_relative_path("config/api_token.txt"));
    assert!(is_sensitive_tracked_relative_path("tokens/prod.yml"));
    assert!(!is_sensitive_tracked_relative_path("src/auth/token.rs"));
    assert!(!is_sensitive_tracked_relative_path("src/auth/token/mod.rs"));
    assert!(!is_sensitive_tracked_relative_path(
        "crates/password/src/lib.rs"
    ));
    assert!(!is_sensitive_tracked_relative_path("src/secrets.rs"));
    assert!(is_sensitive_tracked_relative_path(".aws/token.rs"));
    assert!(is_sensitive_tracked_relative_path(".env/mod.rs"));
}

#[test]
fn mockup_page_artifacts_are_omitted_from_review_context() {
    assert!(is_omitted_tracked_relative_path(
        "docs/mockups/app/chat/index.html"
    ));
    assert!(!is_omitted_tracked_relative_path(
        "docs/mockups/src/pages/app/chat/index.source.ts"
    ));
    assert!(!is_omitted_untracked_relative_path(
        "docs/mockups/src/pages/marketing/index.source.ts"
    ));
    assert!(!is_omitted_tracked_relative_path(
        "docs/mockups/src/components.ts"
    ));
    assert!(!is_omitted_tracked_relative_path("docs/mockups/README.md"));
}

#[cfg(unix)]
#[test]
fn skips_unreadable_untracked_files() {
    let workspace = temp_workspace();

    let diff = build_untracked_file_diff(workspace.path(), "deleted-before-read.txt").unwrap();

    assert!(diff.text().contains("[skipped: unreadable file]"));
    assert!(diff.is_skipped());
}

#[cfg(unix)]
#[test]
fn skips_large_untracked_files() {
    let workspace = temp_workspace();
    fs::write(
        workspace.path().join("large.txt"),
        vec![b'a'; MAX_UNTRACKED_FILE_BYTES as usize + 1],
    )
    .unwrap();

    let diff = build_untracked_file_diff(workspace.path(), "large.txt").unwrap();

    assert!(diff.text().contains("[skipped: file exceeds 262144 bytes]"));
    assert!(diff.is_skipped());
}

#[cfg(not(unix))]
#[test]
fn skips_untracked_content_without_file_identity_support() {
    let diff = build_untracked_file_diff(Path::new("/repo"), "notes/new-review-case.md").unwrap();

    assert!(diff.text().contains("requires Unix file identity support"));
    assert!(diff.is_skipped());
}

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn path(&self) -> &Path {
        &self.path
    }

    fn parent(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.parent());
    }
}

fn temp_workspace() -> TempWorkspace {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique = TEMP_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let parent = std::env::temp_dir().join(format!(
        "xtask-review-untracked-{}-{nanos}-{unique}",
        std::process::id()
    ));
    let path = parent.join("workspace");
    fs::create_dir_all(&path).unwrap();
    TempWorkspace { path }
}
