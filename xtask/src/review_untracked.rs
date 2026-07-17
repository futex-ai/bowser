//! Synthetic diff generation for untracked review files.

#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::io::Read;
use std::path::{Component, Path};

#[cfg(unix)]
use rustix::fs::{Mode, OFlags, open as rustix_open, openat};

use crate::error::Result;
use crate::review_sensitive_paths::is_sensitive_untracked_relative_path;

#[cfg(unix)]
const MAX_UNTRACKED_FILE_BYTES: u64 = 256 * 1024;

/// Synthetic diff text plus whether file content was skipped.
#[expect(
    unreachable_pub,
    reason = "unimock's generated public ReviewProcessRunner mock exposes this type"
)]
pub struct UntrackedFileDiff {
    text: String,
    skipped: bool,
}

impl UntrackedFileDiff {
    /// Build a result for included file content.
    pub(crate) fn included(text: String) -> Self {
        Self {
            text,
            skipped: false,
        }
    }

    /// Build a result for skipped file content.
    pub(crate) fn skipped(text: String) -> Self {
        Self {
            text,
            skipped: true,
        }
    }

    /// Return the rendered synthetic diff text.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// Consume the result into rendered synthetic diff text.
    pub(crate) fn into_text(self) -> String {
        self.text
    }

    /// Return whether file content was skipped.
    pub(crate) fn is_skipped(&self) -> bool {
        self.skipped
    }
}

/// Build reviewable diff text for an untracked file path from `git ls-files`.
#[cfg(unix)]
pub(crate) fn build_untracked_file_diff(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<UntrackedFileDiff> {
    let header = format!("### untracked file {relative_path}\n");

    if !is_safe_relative_path(relative_path) {
        return Ok(skip(format!("{header}[skipped: unsafe relative path]\n")));
    }

    if is_sensitive_untracked_relative_path(relative_path) {
        return Ok(skip(format!("{header}[skipped: sensitive path]\n")));
    }

    let file = match open_file_without_following_symlinks(workspace_root, relative_path) {
        Ok(file) => file,
        Err(_) => return Ok(skip(format!("{header}[skipped: unreadable file]\n"))),
    };
    let opened_metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => return Ok(skip(format!("{header}[skipped: unreadable file]\n"))),
    };

    if !opened_metadata.is_file() {
        return Ok(skip(format!("{header}[skipped: not a regular file]\n")));
    }

    if opened_metadata.len() > MAX_UNTRACKED_FILE_BYTES {
        return Ok(skip(format!(
            "{header}[skipped: file exceeds {MAX_UNTRACKED_FILE_BYTES} bytes]\n"
        )));
    }

    let mut bytes = Vec::new();
    let read_limit = MAX_UNTRACKED_FILE_BYTES + 1;
    let mut limited_file = file.take(read_limit);
    if limited_file.read_to_end(&mut bytes).is_err() {
        return Ok(skip(format!("{header}[skipped: unreadable file]\n")));
    }
    if bytes.len() as u64 > MAX_UNTRACKED_FILE_BYTES {
        return Ok(skip(format!(
            "{header}[skipped: file exceeds {MAX_UNTRACKED_FILE_BYTES} bytes]\n"
        )));
    }

    let contents = match std::str::from_utf8(&bytes) {
        Ok(contents) => contents,
        Err(_) => {
            return Ok(skip(format!(
                "{header}[skipped: binary or non-UTF-8 file]\n"
            )));
        }
    };

    Ok(UntrackedFileDiff::included(format!(
        "{header}{}",
        synthetic_diff(relative_path, contents)
    )))
}

#[cfg(unix)]
fn skip(text: String) -> UntrackedFileDiff {
    UntrackedFileDiff::skipped(text)
}

#[cfg(unix)]
fn open_file_without_following_symlinks(
    workspace_root: &Path,
    relative_path: &str,
) -> std::result::Result<fs::File, ()> {
    let mut current = rustix_open(
        workspace_root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|_| ())?;
    let mut components = Path::new(relative_path)
        .components()
        .filter_map(normal_or_current_component)
        .peekable();

    while let Some(component) = components.next() {
        let is_file = components.peek().is_none();
        let flags = if is_file {
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW
        } else {
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW
        };
        let next = openat(&current, Path::new(component), flags, Mode::empty()).map_err(|_| ())?;

        if is_file {
            let file = fs::File::from(next);
            let metadata = file.metadata().map_err(|_| ())?;
            if metadata.is_file() {
                return Ok(file);
            }

            return Err(());
        }

        current = next;
    }

    Err(())
}

#[cfg(unix)]
fn normal_or_current_component(component: Component<'_>) -> Option<&OsStr> {
    match component {
        Component::Normal(value) => Some(value),
        Component::CurDir => None,
        _ => None,
    }
}

#[cfg(not(unix))]
pub(crate) fn build_untracked_file_diff(
    _workspace_root: &Path,
    relative_path: &str,
) -> Result<UntrackedFileDiff> {
    let header = format!("### untracked file {relative_path}\n");

    if !is_safe_relative_path(relative_path) {
        return Ok(UntrackedFileDiff::skipped(format!(
            "{header}[skipped: unsafe relative path]\n"
        )));
    }

    if is_sensitive_untracked_relative_path(relative_path) {
        return Ok(UntrackedFileDiff::skipped(format!(
            "{header}[skipped: sensitive path]\n"
        )));
    }

    Ok(UntrackedFileDiff::skipped(format!(
        "{header}[skipped: untracked content review requires Unix file identity support]\n"
    )))
}

fn is_safe_relative_path(relative_path: &str) -> bool {
    let path = Path::new(relative_path);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

#[cfg(unix)]
fn synthetic_diff(relative_path: &str, contents: &str) -> String {
    let line_count = contents.lines().count();
    let mut diff = format!(
        "diff --git a/{relative_path} b/{relative_path}\nnew file mode 100644\n--- /dev/null\n+++ b/{relative_path}\n@@ -0,0 +1,{line_count} @@\n"
    );

    for line in contents.split_inclusive('\n') {
        diff.push('+');
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }

    diff
}

#[cfg(test)]
#[path = "_tests_/review_untracked_tests.rs"]
mod review_untracked_tests;
