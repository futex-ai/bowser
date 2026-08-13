//! Versioned, host-portable browser session checkpoints.

mod io;
mod types;

pub use io::{read_checkpoint, write_checkpoint};
pub use types::{
    CHECKPOINT_VERSION, CheckpointCookie, CheckpointOrigin, CheckpointPage, CheckpointStorageEntry,
    CheckpointSummary, SessionCheckpoint,
};

#[cfg(test)]
#[path = "_tests_/checkpoint_tests.rs"]
mod checkpoint_tests;
