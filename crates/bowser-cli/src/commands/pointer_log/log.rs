//! Pointer telemetry log sinks.

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::{CliError, Result};

use super::event::{PointerEventBatch, PointerLogRecord};

/// Storage boundary for accepted pointer telemetry batches.
pub(super) trait PointerEventLog {
    /// Appends a validated event batch and returns the number of events written.
    fn append_batch(&self, batch: &PointerEventBatch) -> Result<usize>;
}

/// Pointer telemetry sink that accepts events without persisting them.
pub(super) struct NoopPointerEventLog;

impl PointerEventLog for NoopPointerEventLog {
    fn append_batch(&self, batch: &PointerEventBatch) -> Result<usize> {
        Ok(batch.events.len())
    }
}

/// File-backed pointer telemetry log.
pub(super) struct FilePointerEventLog {
    path: PathBuf,
    file: Mutex<File>,
}

impl FilePointerEventLog {
    /// Opens a JSONL log file in append mode, creating parent directories.
    pub(super) fn open(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && let Err(source) = create_dir_all(parent)
        {
            return Err(CliError::PointerLogCreateDir {
                path: parent.display().to_string(),
                source,
            });
        }
        let file = match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => file,
            Err(source) => {
                return Err(CliError::PointerLogOpen {
                    path: path.display().to_string(),
                    source,
                });
            }
        };
        Ok(Self {
            path,
            file: Mutex::new(file),
        })
    }
}

impl PointerEventLog for FilePointerEventLog {
    fn append_batch(&self, batch: &PointerEventBatch) -> Result<usize> {
        let mut file = match self.file.lock() {
            Ok(file) => file,
            Err(_) => {
                return Err(CliError::PointerLogLock {
                    path: self.path.display().to_string(),
                });
            }
        };
        for event in &batch.events {
            let record = PointerLogRecord::new(batch, event);
            if let Err(source) = serde_json::to_writer(&mut *file, &record) {
                return Err(CliError::PointerLogSerialize {
                    path: self.path.display().to_string(),
                    source,
                });
            }
            if let Err(source) = file.write_all(b"\n") {
                return Err(CliError::PointerLogAppend {
                    path: self.path.display().to_string(),
                    source,
                });
            }
        }
        if let Err(source) = file.flush() {
            return Err(CliError::PointerLogAppend {
                path: self.path.display().to_string(),
                source,
            });
        }
        Ok(batch.events.len())
    }
}
