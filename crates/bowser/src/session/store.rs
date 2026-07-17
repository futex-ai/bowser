//! Session storage trait and filesystem implementation.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tempfile::NamedTempFile;

use crate::error::{Error, Result};

use super::metadata::{RawSessionMetadata, SessionMetadata};

const MAX_SESSION_ID_LEN: usize = 128;

/// Session storage abstraction.
#[cfg_attr(test, unimock::unimock(api = SessionStoreMock))]
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn load(&self, session_id: &str) -> Result<SessionMetadata>;
    async fn save(&self, metadata: &SessionMetadata) -> Result<()>;
    async fn list(&self) -> Result<Vec<SessionMetadata>>;
    async fn remove(&self, session_id: &str) -> Result<()>;
}

/// Filesystem-backed session store.
#[derive(Clone, Debug)]
pub struct FileSessionStore {
    root: PathBuf,
}

impl FileSessionStore {
    /// Creates a file-backed session store.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Returns the root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn session_path(&self, session_id: &str) -> Result<PathBuf> {
        validate_session_id(session_id)?;
        Ok(self.root.join(format!("{session_id}.json")))
    }
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn load(&self, session_id: &str) -> Result<SessionMetadata> {
        let path = self.session_path(session_id)?;
        let contents = fs::read_to_string(&path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                Error::SessionNotFound {
                    session_id: session_id.to_string(),
                }
            } else {
                Error::io("read session metadata", err)
            }
        })?;
        deserialize_metadata(&contents, session_id)
    }

    async fn save(&self, metadata: &SessionMetadata) -> Result<()> {
        let path = self.session_path(&metadata.id)?;
        fs::create_dir_all(&self.root).map_err(|err| Error::io("create session directory", err))?;
        let contents = serde_json::to_string_pretty(metadata).map_err(|err| {
            Error::session(format!(
                "failed to serialize session {}: {err}",
                metadata.id
            ))
        })?;
        write_metadata_atomically(&self.root, &path, contents.as_bytes())
    }

    async fn list(&self) -> Result<Vec<SessionMetadata>> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut sessions = Vec::new();
        for entry in
            fs::read_dir(&self.root).map_err(|err| Error::io("read session directory", err))?
        {
            let entry = entry.map_err(|err| Error::io("read session entry", err))?;
            if entry
                .file_type()
                .map_err(|err| Error::io("read session entry type", err))?
                .is_file()
            {
                if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                let path = entry.path();
                let Some(session_id) = path.file_stem().and_then(|value| value.to_str()) else {
                    continue;
                };
                if validate_session_id(session_id).is_err() {
                    continue;
                }
                let contents = match fs::read_to_string(&path) {
                    Ok(contents) => contents,
                    Err(error) => {
                        tracing::warn!(
                            session_id,
                            error = %error,
                            "skipping unreadable session metadata"
                        );
                        continue;
                    }
                };
                let metadata = match deserialize_metadata(&contents, session_id) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        tracing::warn!(
                            session_id,
                            error = %error,
                            "skipping invalid session metadata"
                        );
                        continue;
                    }
                };
                sessions.push(metadata);
            }
        }
        sessions.sort_by_key(|session| session.updated_at);
        sessions.reverse();
        Ok(sessions)
    }

    async fn remove(&self, session_id: &str) -> Result<()> {
        let path = self.session_path(session_id)?;
        if path.exists() {
            fs::remove_file(path).map_err(|err| Error::io("remove session metadata", err))?;
        }
        Ok(())
    }
}

fn write_metadata_atomically(root: &Path, path: &Path, contents: &[u8]) -> Result<()> {
    let mut temporary = match NamedTempFile::new_in(root) {
        Ok(temporary) => temporary,
        Err(source) => return Err(Error::io("create session metadata temporary file", source)),
    };
    if let Err(source) = temporary.write_all(contents) {
        return Err(Error::io("write session metadata temporary file", source));
    }
    if let Err(source) = temporary.as_file().sync_all() {
        return Err(Error::io("sync session metadata temporary file", source));
    }
    match temporary.persist(path) {
        Ok(_) => Ok(()),
        Err(error) => Err(Error::io("replace session metadata", error.error)),
    }
}

fn validate_session_id(session_id: &str) -> Result<()> {
    let suffix = session_id.strip_prefix("bsr_");
    let valid = session_id.len() <= MAX_SESSION_ID_LEN
        && suffix.is_some_and(|value| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        });
    if valid {
        return Ok(());
    }
    Err(Error::InvalidSessionId {
        session_id: session_id.to_string(),
    })
}

fn deserialize_metadata(contents: &str, expected_session_id: &str) -> Result<SessionMetadata> {
    let raw = serde_json::from_str::<RawSessionMetadata>(contents).map_err(|err| {
        Error::session(format!(
            "failed to deserialize session {expected_session_id}: {err}"
        ))
    })?;
    let metadata = SessionMetadata::from(raw);
    if metadata.id != expected_session_id {
        return Err(Error::SessionIdMismatch {
            expected_session_id: expected_session_id.to_string(),
            actual_session_id: metadata.id,
        });
    }
    Ok(metadata)
}

/// Returns the default session directory.
pub fn default_session_dir() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bowser")
        .join("sessions")
}
