//! Neutral working directory for nested review agents.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};

static REVIEW_SANDBOX_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Empty directory used as the nested Codex working root.
pub(crate) struct ReviewSandbox {
    path: PathBuf,
}

impl ReviewSandbox {
    /// Create a neutral temporary directory outside the reviewed repository.
    pub(crate) fn create(workspace_root: &Path) -> Result<Self> {
        let temp_root = select_review_temp_root(
            workspace_root,
            &[
                std::env::temp_dir(),
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
            ],
        )?;
        let path = temp_root.join(format!(
            "bowser-xtask-review-{}-{}-{}",
            std::process::id(),
            timestamp_nanos(),
            REVIEW_SANDBOX_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));

        if let Err(source) = fs::create_dir(&path) {
            return Err(Error::ReviewSandboxCreate { path, source });
        }

        Ok(Self { path })
    }

    /// Return the neutral directory path.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ReviewSandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn timestamp_nanos() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    }
}

fn select_review_temp_root(workspace_root: &Path, candidate_roots: &[PathBuf]) -> Result<PathBuf> {
    let workspace_root = canonicalize_review_path(workspace_root)?;
    let mut checked_roots = Vec::new();

    for candidate_root in candidate_roots {
        let Ok(candidate_root) = candidate_root.canonicalize() else {
            continue;
        };
        checked_roots.push(candidate_root.clone());
        if !candidate_root.starts_with(&workspace_root) {
            return Ok(candidate_root);
        }
    }

    Err(Error::ReviewSandboxRootInsideWorkspace {
        workspace_root,
        checked_roots,
    })
}

fn canonicalize_review_path(path: &Path) -> Result<PathBuf> {
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(source) => Err(Error::ReviewSandboxCanonicalize {
            path: path.to_path_buf(),
            source,
        }),
    }
}

#[cfg(test)]
#[path = "_tests_/review_sandbox_tests.rs"]
mod review_sandbox_tests;
