use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemImpl, Type};

use crate::error::{Error, Result};
use crate::rust_trait_audit_allowlist::{APPROVED_TEST_DOUBLES, ApprovedTestDouble};

pub(crate) const TRAIT_GUIDANCE_PATH: &str = "docs/dev/rust/architecture/dependencies.md";
const RUST_WORKSPACE_ROOTS: &[&str] = &["crates", "xtask"];
const IGNORED_TEST_IMPL_TRAITS: &[&str] = &["Drop", "std::error::Error", "std::fmt::Display"];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ObservedTestDouble {
    path: String,
    trait_name: String,
    impl_name: String,
    line_number: usize,
}

pub(crate) fn run_rust_trait_audit(workspace_root: &Path) -> Result<()> {
    let guidance_path = workspace_root.join(TRAIT_GUIDANCE_PATH);
    if !guidance_path.is_file() {
        return Err(Error::RustTraitAuditMissingDoc {
            path: guidance_path,
        });
    }

    let mut observed_approved = BTreeSet::new();
    let mut unapproved = Vec::new();

    for root in RUST_WORKSPACE_ROOTS {
        let root_path = workspace_root.join(root);
        if !root_path.exists() {
            continue;
        }
        visit_rust_files(
            workspace_root,
            &root_path,
            &mut |relative_path, full_path| {
                audit_test_file(
                    relative_path,
                    full_path,
                    &mut observed_approved,
                    &mut unapproved,
                )
            },
        )?;
    }

    let stale_allowlist: Vec<_> = APPROVED_TEST_DOUBLES
        .iter()
        .copied()
        .filter(|exception| !observed_approved.contains(exception))
        .collect();

    if unapproved.is_empty() && stale_allowlist.is_empty() {
        return Ok(());
    }

    let mut details = String::from("rust trait audit failed");
    if !unapproved.is_empty() {
        details.push_str("\n\nUnapproved handwritten test doubles:");
        for violation in &unapproved {
            details.push_str("\n- ");
            details.push_str(&format!(
                "{}:{}: `impl {} for {}`",
                violation.path, violation.line_number, violation.trait_name, violation.impl_name
            ));
        }
    }
    if !stale_allowlist.is_empty() {
        details.push_str("\n\nStale approved exceptions:");
        for exception in &stale_allowlist {
            details.push_str("\n- ");
            details.push_str(&format!(
                "{}: `impl {} for {}` ({})",
                exception.path, exception.trait_name, exception.impl_name, exception.reason
            ));
        }
    }

    Err(Error::RustTraitAuditViolations { details })
}

fn visit_rust_files(
    workspace_root: &Path,
    root: &Path,
    visitor: &mut dyn FnMut(&str, &Path) -> Result<()>,
) -> Result<()> {
    let entries = fs::read_dir(root).map_err(|source| Error::RustTraitAuditReadDir {
        path: root.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| Error::RustTraitAuditReadDir {
            path: root.to_path_buf(),
            source,
        })?;
        let file_type = entry
            .file_type()
            .map_err(|source| Error::RustTraitAuditReadDir {
                path: entry.path(),
                source,
            })?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            visit_rust_files(workspace_root, &path, visitor)?;
            continue;
        }
        if !file_type.is_file() || path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let relative_path = relative_path(workspace_root, &path)?;
        visitor(relative_path, &path)?;
    }

    Ok(())
}

fn audit_test_file(
    relative_path: &str,
    full_path: &Path,
    observed_approved: &mut BTreeSet<ApprovedTestDouble>,
    unapproved: &mut Vec<ObservedTestDouble>,
) -> Result<()> {
    if !is_test_source(relative_path) {
        return Ok(());
    }

    let source = fs::read_to_string(full_path).map_err(|source| Error::RustTraitAuditReadFile {
        path: full_path.to_path_buf(),
        source,
    })?;
    let syntax = syn::parse_file(&source).map_err(|source| Error::RustTraitAuditParseFile {
        path: full_path.to_path_buf(),
        source,
    })?;
    let mut visitor = TraitImplVisitor::default();
    visitor.visit_file(&syntax);

    for implementation in visitor.implementations {
        if IGNORED_TEST_IMPL_TRAITS.contains(&implementation.trait_name.as_str()) {
            continue;
        }

        let Some(exception) = APPROVED_TEST_DOUBLES.iter().copied().find(|exception| {
            exception.path == relative_path
                && exception.trait_name == implementation.trait_name
                && exception.impl_name == implementation.impl_name
        }) else {
            unapproved.push(ObservedTestDouble {
                path: relative_path.to_owned(),
                trait_name: implementation.trait_name,
                impl_name: implementation.impl_name,
                line_number: implementation.line_number,
            });
            continue;
        };

        observed_approved.insert(exception);
    }

    Ok(())
}

fn is_test_source(relative_path: &str) -> bool {
    !relative_path.contains("/tests/")
        && (relative_path.contains("/_tests_/") || relative_path.ends_with("_tests.rs"))
}

#[derive(Default)]
struct TraitImplVisitor {
    implementations: Vec<ParsedTraitImpl>,
}

impl<'ast> Visit<'ast> for TraitImplVisitor {
    fn visit_item_impl(&mut self, implementation: &'ast ItemImpl) {
        if let Some((_, trait_path, _)) = &implementation.trait_ {
            self.implementations.push(ParsedTraitImpl {
                trait_name: trait_path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::"),
                impl_name: self_type_name(&implementation.self_ty),
                line_number: implementation.impl_token.span().start().line,
            });
        }
        visit::visit_item_impl(self, implementation);
    }
}

struct ParsedTraitImpl {
    trait_name: String,
    impl_name: String,
    line_number: usize,
}

fn self_type_name(self_type: &Type) -> String {
    match self_type {
        Type::Group(group) => self_type_name(&group.elem),
        Type::Paren(paren) => self_type_name(&paren.elem),
        Type::Path(path) => path.path.segments.last().map_or_else(
            || "<unknown>".to_owned(),
            |segment| segment.ident.to_string(),
        ),
        Type::Reference(reference) => self_type_name(&reference.elem),
        _ => "<unknown>".to_owned(),
    }
}

fn relative_path<'a>(workspace_root: &'a Path, path: &'a Path) -> Result<&'a str> {
    let relative =
        path.strip_prefix(workspace_root)
            .map_err(|source| Error::RustTraitAuditRelativePath {
                workspace_root: workspace_root.to_path_buf(),
                path: path.to_path_buf(),
                source,
            })?;
    relative
        .to_str()
        .ok_or_else(|| Error::RustTraitAuditNonUtf8Path {
            path: path.to_path_buf(),
        })
}

#[cfg(test)]
#[path = "_tests_/rust_trait_audit_tests.rs"]
mod rust_trait_audit_tests;
