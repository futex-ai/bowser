//! Detached-session persistence.

mod cleanup;
mod metadata;
mod process;
mod selection;
mod store;
mod summary;

pub use cleanup::{cleanup_expired_sessions, terminate_process, terminate_session_processes};
pub use metadata::{SessionMetadata, SessionPageMetadata};
pub use store::{FileSessionStore, SessionStore, default_session_dir};

pub(crate) use process::LaunchedProcessGuard;

#[cfg(test)]
#[path = "_tests_/session_tests.rs"]
pub(crate) mod session_tests;

#[cfg(test)]
#[path = "_tests_/process_tests.rs"]
mod process_tests;
