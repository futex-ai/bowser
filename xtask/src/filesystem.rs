//! Shared filesystem helpers for repository audits.

use std::fs;
use std::path::Path;

/// Returns true only for a regular file at the path itself, never its symlink target.
pub(crate) fn is_regular_file_without_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_file())
}

/// Formats a path relative to the workspace when it is contained by that root.
pub(crate) fn display_relative_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root).map_or_else(
        |_| path.display().to_string(),
        |relative| relative.display().to_string(),
    )
}
