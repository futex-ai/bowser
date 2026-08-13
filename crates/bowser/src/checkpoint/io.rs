//! Atomic checkpoint serialization and filesystem I/O.

use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use crate::error::{Error, Result};

use super::SessionCheckpoint;

/// Reads and validates a portable checkpoint file.
pub fn read_checkpoint(path: &Path) -> Result<SessionCheckpoint> {
    let bytes = std::fs::read(path).map_err(|source| Error::CheckpointRead {
        path: path.to_path_buf(),
        source,
    })?;
    let checkpoint = serde_json::from_slice::<SessionCheckpoint>(&bytes).map_err(|source| {
        Error::CheckpointInvalid {
            reason: source.to_string(),
        }
    })?;
    checkpoint.validate()?;
    Ok(checkpoint)
}

/// Atomically writes a validated portable checkpoint file.
pub fn write_checkpoint(path: &Path, checkpoint: &SessionCheckpoint) -> Result<()> {
    checkpoint.validate()?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|source| Error::CheckpointWrite {
        path: path.to_path_buf(),
        source,
    })?;
    let bytes =
        serde_json::to_vec_pretty(checkpoint).map_err(|source| Error::CheckpointInvalid {
            reason: source.to_string(),
        })?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|source| Error::CheckpointWrite {
        path: path.to_path_buf(),
        source,
    })?;
    temporary
        .write_all(&bytes)
        .map_err(|source| Error::CheckpointWrite {
            path: path.to_path_buf(),
            source,
        })?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| Error::CheckpointWrite {
            path: path.to_path_buf(),
            source,
        })?;
    temporary
        .persist(path)
        .map_err(|failure| Error::CheckpointWrite {
            path: path.to_path_buf(),
            source: failure.error,
        })?;
    Ok(())
}
