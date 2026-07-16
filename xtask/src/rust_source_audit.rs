//! Audit Rust workspace packages for orphan source files outside any target module graph.
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use syn::{Attribute, Expr, ExprLit, Item, ItemMod, Lit, Meta};

use crate::error::{Error, Result};
use crate::filesystem::{display_relative_path, is_regular_file_without_symlink};

const CARGO_METADATA_ARGS: &[&str] = &["metadata", "--no-deps", "--format-version", "1"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleOrigin {
    Conventional,
    PathAttribute,
    TargetRoot,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    manifest_path: PathBuf,
    targets: Vec<MetadataTarget>,
}

#[derive(Debug, Deserialize)]
struct MetadataTarget {
    src_path: PathBuf,
}

pub(crate) fn run_rust_source_audit(workspace_root: &Path) -> Result<()> {
    let metadata = load_workspace_metadata(workspace_root)?;
    let workspace_members = metadata
        .workspace_members
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut orphaned_files = Vec::new();

    for package in metadata
        .packages
        .iter()
        .filter(|package| workspace_members.contains(&package.id))
    {
        orphaned_files.extend(find_orphaned_package_files(package)?);
    }

    if orphaned_files.is_empty() {
        return Ok(());
    }

    let count = orphaned_files.len();
    let details = orphaned_files
        .into_iter()
        .map(|path| format!("- {}", display_relative_path(workspace_root, &path)))
        .collect::<Vec<_>>()
        .join("\n");

    Err(Error::RustSourceAuditViolations { count, details })
}

fn load_workspace_metadata(workspace_root: &Path) -> Result<CargoMetadata> {
    let output = Command::new("cargo")
        .args(CARGO_METADATA_ARGS)
        .current_dir(workspace_root)
        .output()
        .map_err(|source| Error::RustSourceAuditMetadataCommandStart { source })?;

    if !output.status.success() {
        return Err(Error::RustSourceAuditMetadataCommandFailed {
            status: output.status,
            stderr: stderr_string(&output.stderr),
        });
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|source| Error::RustSourceAuditMetadataParse { source })
}

fn find_orphaned_package_files(package: &MetadataPackage) -> Result<Vec<PathBuf>> {
    let Some(package_root) = package.manifest_path.parent() else {
        return Err(Error::RustSourceAuditPackageRoot {
            manifest_path: package.manifest_path.clone(),
        });
    };

    let mut reachable_files = BTreeSet::new();
    for target in &package.targets {
        if target.src_path.extension() != Some(OsStr::new("rs"))
            || !is_regular_file_without_symlink(&target.src_path)
        {
            continue;
        }

        visit_module_file(
            &target.src_path,
            ModuleOrigin::TargetRoot,
            &mut reachable_files,
        )?;
    }

    let mut package_files = BTreeSet::new();
    collect_package_rust_files(package_root, &mut package_files)?;

    Ok(package_files
        .into_iter()
        .filter(|path| !reachable_files.contains(path))
        .collect())
}

fn visit_module_file(
    path: &Path,
    origin: ModuleOrigin,
    reachable_files: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if !reachable_files.insert(path.to_path_buf()) {
        return Ok(());
    }

    let source = fs::read_to_string(path).map_err(|source| Error::RustSourceAuditReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let syntax = syn::parse_file(&source).map_err(|source| Error::RustSourceAuditParseFile {
        path: path.to_path_buf(),
        source,
    })?;
    let module_dir = child_module_directory(path, origin);

    visit_items(path, &module_dir, &syntax.items, reachable_files)
}

fn visit_items(
    source_path: &Path,
    module_dir: &Path,
    items: &[Item],
    reachable_files: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    for item in items {
        let Item::Mod(module) = item else {
            continue;
        };

        if let Some((_, inline_items)) = &module.content {
            let inline_module_dir = module_dir.join(module.ident.to_string());
            visit_items(
                source_path,
                &inline_module_dir,
                inline_items,
                reachable_files,
            )?;
            continue;
        }

        let Some(resolved_module) = resolve_external_module_path(source_path, module_dir, module)?
        else {
            continue;
        };

        visit_module_file(
            &resolved_module.path,
            resolved_module.origin,
            reachable_files,
        )?;
    }

    Ok(())
}

fn resolve_external_module_path(
    source_path: &Path,
    module_dir: &Path,
    module: &ItemMod,
) -> Result<Option<ResolvedModule>> {
    if let Some(path_override) = module_path_override(&module.attrs) {
        let Some(source_dir) = source_path.parent() else {
            return Ok(None);
        };
        let resolved = source_dir.join(path_override);
        return Ok(
            is_regular_file_without_symlink(&resolved).then_some(ResolvedModule {
                path: resolved,
                origin: ModuleOrigin::PathAttribute,
            }),
        );
    }

    let module_name = module.ident.to_string();
    let direct_path = module_dir.join(format!("{module_name}.rs"));
    if is_regular_file_without_symlink(&direct_path) {
        return Ok(Some(ResolvedModule {
            path: direct_path,
            origin: ModuleOrigin::Conventional,
        }));
    }

    let nested_path = module_dir.join(module_name).join("mod.rs");
    Ok(
        is_regular_file_without_symlink(&nested_path).then_some(ResolvedModule {
            path: nested_path,
            origin: ModuleOrigin::Conventional,
        }),
    )
}

fn module_path_override(attributes: &[Attribute]) -> Option<PathBuf> {
    attributes.iter().find_map(|attribute| {
        let Meta::NameValue(meta) = &attribute.meta else {
            return None;
        };
        if !meta.path.is_ident("path") {
            return None;
        }

        let Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) = &meta.value
        else {
            return None;
        };

        Some(PathBuf::from(value.value()))
    })
}

fn child_module_directory(path: &Path, origin: ModuleOrigin) -> PathBuf {
    let parent = path.parent().map_or_else(PathBuf::new, Path::to_path_buf);
    if origin != ModuleOrigin::Conventional || path.file_name() == Some(OsStr::new("mod.rs")) {
        return parent;
    }

    path.file_stem()
        .map(|stem| parent.join(stem))
        .unwrap_or(parent)
}

fn collect_package_rust_files(path: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    if path.file_name() == Some(OsStr::new("target")) {
        return Ok(());
    }

    let metadata =
        fs::symlink_metadata(path).map_err(|source| Error::RustSourceAuditReadDirectory {
            path: path.to_path_buf(),
            source,
        })?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if path.extension() == Some(OsStr::new("rs")) {
            files.insert(path.to_path_buf());
        }
        return Ok(());
    }

    for entry in fs::read_dir(path).map_err(|source| Error::RustSourceAuditReadDirectory {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| Error::RustSourceAuditReadDirectory {
            path: path.to_path_buf(),
            source,
        })?;
        collect_package_rust_files(&entry.path(), files)?;
    }

    Ok(())
}

fn stderr_string(stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return String::new();
    }

    format!(": {stderr}")
}

struct ResolvedModule {
    path: PathBuf,
    origin: ModuleOrigin,
}

#[cfg(test)]
#[path = "_tests_/rust_source_audit_tests.rs"]
mod rust_source_audit_tests;
