//! Detached-session persistence.

mod cleanup;
mod metadata;
mod process;
mod profile;
mod selection;
mod store;
mod summary;

pub use cleanup::{
    cleanup_expired_sessions, terminate_process, terminate_session_processes,
    terminate_session_processes_and_wait,
};
pub use metadata::{SessionMetadata, SessionPageMetadata};
pub use profile::remove_owned_session_profile;
pub use store::{FileSessionStore, SessionStore, default_session_dir};

pub(crate) use process::LaunchedProcessGuard;
pub(crate) use profile::OwnedProfileGuard;

#[cfg(test)]
#[path = "_tests_/session_tests.rs"]
pub(crate) mod session_tests;

#[cfg(test)]
#[path = "_tests_/process_tests.rs"]
mod process_tests;

#[cfg(test)]
#[path = "_tests_/profile_tests.rs"]
mod profile_tests;
