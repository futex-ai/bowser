//! Owned browser-profile validation and cleanup.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use crate::error::{Error, Result};

use super::metadata::SessionMetadata;

const REMOVE_ATTEMPTS: usize = 20;
const REMOVE_RETRY_DELAY: Duration = Duration::from_millis(50);

#[cfg_attr(test, unimock::unimock(api = ProfileDirectoryControlMock))]
pub(crate) trait ProfileDirectoryControl: Send + Sync {
    fn remove(&self, path: &Path) -> Result<()>;
}

struct SystemProfileDirectoryControl;

impl ProfileDirectoryControl for SystemProfileDirectoryControl {
    fn remove(&self, path: &Path) -> Result<()> {
        let Some(parent) = path.parent() else {
            return Err(Error::InvalidOwnedProfilePath {
                path: path.to_path_buf(),
            });
        };
        let parent_metadata = match fs::symlink_metadata(parent) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(Error::SessionProfileRemove {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        if parent_metadata.file_type().is_symlink() || !parent_metadata.file_type().is_dir() {
            return Err(Error::InvalidOwnedProfilePath {
                path: path.to_path_buf(),
            });
        }
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(Error::SessionProfileRemove {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        let is_directory = metadata.file_type().is_dir() && !metadata.file_type().is_symlink();
        for attempt in 0..REMOVE_ATTEMPTS {
            let result = if is_directory {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            };
            match result {
                Ok(()) => return Ok(()),
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(source)
                    if attempt + 1 < REMOVE_ATTEMPTS && is_transient_remove_error(&source) =>
                {
                    std::thread::sleep(REMOVE_RETRY_DELAY);
                }
                Err(source) => {
                    return Err(Error::SessionProfileRemove {
                        path: path.to_path_buf(),
                        source,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Removes a profile only when metadata marks it as Bowser-owned and its path is safe.
pub async fn remove_owned_session_profile(
    session_root: &Path,
    metadata: &SessionMetadata,
) -> Result<()> {
    if !metadata.owns_user_data_dir {
        return Ok(());
    }
    validate_owned_profile_path(session_root, &metadata.user_data_dir)?;
    let path = metadata.user_data_dir.clone();
    match tokio::task::spawn_blocking(move || SystemProfileDirectoryControl.remove(&path)).await {
        Ok(result) => result,
        Err(source) => Err(Error::SessionProfileTask { source }),
    }
}

/// Removes an unfinished ephemeral profile unless session persistence takes ownership.
pub(crate) struct OwnedProfileGuard {
    control: Arc<dyn ProfileDirectoryControl>,
    path: Option<PathBuf>,
}

impl OwnedProfileGuard {
    pub(crate) fn system(session_root: &Path, path: PathBuf, owned: bool) -> Result<Self> {
        Self::new(
            Arc::new(SystemProfileDirectoryControl),
            session_root,
            path,
            owned,
        )
    }

    pub(crate) fn new(
        control: Arc<dyn ProfileDirectoryControl>,
        session_root: &Path,
        path: PathBuf,
        owned: bool,
    ) -> Result<Self> {
        if owned {
            validate_owned_profile_path(session_root, &path)?;
        }
        Ok(Self {
            control,
            path: owned.then_some(path),
        })
    }

    pub(crate) fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for OwnedProfileGuard {
    fn drop(&mut self) {
        if let Some(path) = self.path.as_deref() {
            let _ = self.control.remove(path);
        }
    }
}

fn validate_owned_profile_path(session_root: &Path, profile: &Path) -> Result<()> {
    let profiles_root = session_root.join("profiles");
    let valid = profile
        .strip_prefix(profiles_root)
        .ok()
        .and_then(single_normal_component)
        .and_then(|value| value.to_str())
        .is_some_and(|value| Uuid::parse_str(value).is_ok());
    if valid {
        return Ok(());
    }
    Err(Error::InvalidOwnedProfilePath {
        path: profile.to_path_buf(),
    })
}

fn single_normal_component(path: &Path) -> Option<&std::ffi::OsStr> {
    let mut components = path.components();
    let Component::Normal(value) = components.next()? else {
        return None;
    };
    components.next().is_none().then_some(value)
}

fn is_transient_remove_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::DirectoryNotEmpty | std::io::ErrorKind::PermissionDenied
    )
}
