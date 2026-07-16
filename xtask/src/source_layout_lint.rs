//! Enforce the repository's source-adjacent Rust test layout.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub(crate) fn verify_source_adjacent_test_layout(workspace_root: &Path) -> Result<()> {
    for root in [workspace_root.join("crates"), workspace_root.join("xtask")] {
        verify_entry(workspace_root, &root)?;
    }
    Ok(())
}

fn verify_entry(workspace_root: &Path, path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(Error::ReadDirectory {
                path: relative_path(workspace_root, path),
                source,
            });
        }
    };
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    if metadata.is_dir() {
        if is_source_adjacent_tests_directory(path) {
            return Err(Error::SourceAdjacentTestsDirectory {
                path: relative_path(workspace_root, path),
                suggested_path: suggested_tests_directory_path(workspace_root, path),
            });
        }

        for entry in fs::read_dir(path).map_err(|source| Error::ReadDirectory {
            path: relative_path(workspace_root, path),
            source,
        })? {
            let entry = entry.map_err(|source| Error::ReadDirectory {
                path: relative_path(workspace_root, path),
                source,
            })?;
            verify_entry(workspace_root, &entry.path())?;
        }
        return Ok(());
    }

    let components = path_components(path);
    let is_rust_source = path.extension().and_then(|value| value.to_str()) == Some("rs");
    let is_test_tree = components
        .iter()
        .any(|component| *component == "_tests_" || *component == "tests");

    if is_rust_source && !is_test_tree {
        verify_source_file_test_attributes(workspace_root, path)?;
    }
    Ok(())
}

fn verify_source_file_test_attributes(workspace_root: &Path, path: &Path) -> Result<()> {
    let relative = relative_path(workspace_root, path);
    let contents = fs::read_to_string(path).map_err(|source| Error::ReadSourceFile {
        path: relative.clone(),
        source,
    })?;
    let lines: Vec<&str> = contents.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "#[cfg(test)]" {
            let path_line = lines.get(index + 1).map(|value| value.trim());
            let mod_line = lines.get(index + 2).map(|value| value.trim());
            let valid_path = path_line
                .and_then(|value| value.strip_prefix("#[path = \"_tests_/"))
                .and_then(|value| value.strip_suffix("\"]"))
                .is_some();
            let valid_mod = mod_line
                .and_then(external_test_module_name)
                .is_some_and(is_valid_module_name);
            if !valid_path || !valid_mod {
                return Err(Error::InvalidCfgTestUsage {
                    path: relative,
                    line: index + 1,
                });
            }
            continue;
        }

        if trimmed.starts_with("#[test")
            || trimmed.starts_with("#[tokio::test")
            || trimmed.starts_with("#[async_std::test")
        {
            return Err(Error::InlineRustTestAttribute {
                path: relative,
                line: index + 1,
            });
        }
    }
    Ok(())
}

fn external_test_module_name(line: &str) -> Option<&str> {
    [
        "mod ",
        "pub(crate) mod ",
        "pub(super) mod ",
        "pub(self) mod ",
        "pub mod ",
    ]
    .iter()
    .find_map(|prefix| line.strip_prefix(prefix))
    .and_then(|value| value.strip_suffix(';'))
}

fn is_valid_module_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn is_source_adjacent_tests_directory(path: &Path) -> bool {
    let components = path_components(path);
    components.last() == Some(&"tests")
        && components
            .iter()
            .position(|component| *component == "src")
            .is_some_and(|src_index| src_index < components.len().saturating_sub(1))
}

fn suggested_tests_directory_path(workspace_root: &Path, path: &Path) -> PathBuf {
    let mut suggested = path.to_path_buf();
    suggested.set_file_name("_tests_");
    relative_path(workspace_root, &suggested)
}

fn path_components(path: &Path) -> Vec<&str> {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect()
}

fn relative_path(workspace_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(workspace_root)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}

#[cfg(test)]
#[path = "_tests_/source_layout_lint_tests.rs"]
mod source_layout_lint_tests;
