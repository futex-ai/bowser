//! Temporary session directories that clean up detached test browsers.

use std::fs;
use std::path::Path;
use std::thread;

use bowser::{SessionMetadata, terminate_session_processes_and_wait};
use tempfile::TempDir;

/// Temporary Bowser session root with process cleanup on drop.
pub(crate) struct TestSessionDir {
    inner: TempDir,
}

impl TestSessionDir {
    /// Returns the temporary session root.
    pub(crate) fn path(&self) -> &Path {
        self.inner.path()
    }
}

impl Drop for TestSessionDir {
    fn drop(&mut self) {
        let sessions = load_sessions(self.path());
        if sessions.is_empty() {
            return;
        }
        let cleanup = thread::Builder::new()
            .name("bowser-test-session-cleanup".to_string())
            .spawn(move || {
                let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                    .enable_time()
                    .build()
                else {
                    return;
                };
                for session in sessions {
                    let _ = runtime.block_on(terminate_session_processes_and_wait(&session));
                }
            });
        if let Ok(cleanup) = cleanup {
            let _ = cleanup.join();
        }
    }
}

/// Creates a temporary Bowser session root with process cleanup on drop.
pub(crate) fn test_session_dir() -> TestSessionDir {
    TestSessionDir {
        inner: tempfile::tempdir().expect("session dir"),
    }
}

fn load_sessions(path: &Path) -> Vec<SessionMetadata> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()?.to_str() == Some("json")).then_some(path)
        })
        .filter_map(|path| fs::read(path).ok())
        .filter_map(|contents| serde_json::from_slice(&contents).ok())
        .collect()
}
