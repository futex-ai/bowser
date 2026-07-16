use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Error;

use super::*;

static TEMP_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn temp_root_selection_skips_workspace_descendants() {
    let workspace = temp_workspace();
    let inside = workspace.path().join("nested-temp");
    let outside = workspace.outside_sibling("outside-temp");
    fs::create_dir_all(&inside).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let selected = select_review_temp_root(workspace.path(), &[inside, outside.clone()]).unwrap();

    assert_eq!(selected, outside.canonicalize().unwrap());
}

#[test]
fn temp_root_selection_rejects_all_workspace_descendants() {
    let workspace = temp_workspace();
    let inside = workspace.path().join("nested-temp");
    fs::create_dir_all(&inside).unwrap();

    let error = select_review_temp_root(workspace.path(), &[inside]).unwrap_err();

    assert!(matches!(
        error,
        Error::ReviewSandboxRootInsideWorkspace { .. }
    ));
}

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn path(&self) -> &Path {
        &self.path
    }

    fn outside_sibling(&self, name: &str) -> PathBuf {
        self.path.parent().unwrap().join(format!(
            "{}-{name}",
            self.path.file_name().unwrap().to_string_lossy()
        ))
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        let _ = fs::remove_dir_all(self.outside_sibling("outside-temp"));
    }
}

fn temp_workspace() -> TempWorkspace {
    let unique = TEMP_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    let path = std::env::temp_dir().join(format!(
        "xtask-review-sandbox-{}-{nanos}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();

    TempWorkspace { path }
}
