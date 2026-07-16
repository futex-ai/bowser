use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

static TEST_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn audit_passes_with_documented_exceptions_only() {
    let workspace_root = test_workspace();

    run_rust_trait_audit(&workspace_root).expect("audit should pass");
}

#[test]
fn audit_rejects_unapproved_handwritten_test_double() {
    let workspace_root = test_workspace();
    write_file(
        &workspace_root,
        "crates/example/src/_tests_/lib_tests.rs",
        "impl ExampleTrait for ExampleStub {}\n",
    );

    let error = run_rust_trait_audit(&workspace_root).expect_err("audit should fail");

    assert!(error.to_string().contains(
        "crates/example/src/_tests_/lib_tests.rs:1: `impl ExampleTrait for ExampleStub`"
    ));
}

#[test]
fn audit_ignores_crate_root_integration_tests() {
    let workspace_root = test_workspace();
    write_file(
        &workspace_root,
        "crates/example/tests/lib_tests.rs",
        "impl ExampleTrait for ExampleStub {}\n",
    );

    run_rust_trait_audit(&workspace_root).expect("integration tests are outside the audit path");
}

#[test]
fn audit_rejects_generic_multiline_handwritten_test_double() {
    let workspace_root = test_workspace();
    write_file(
        &workspace_root,
        "crates/example/src/_tests_/lib_tests.rs",
        "impl<T>\n    ExampleTrait\n    for ExampleStub<T>\n{}\n",
    );

    let error = run_rust_trait_audit(&workspace_root).expect_err("audit should fail");

    assert!(error.to_string().contains(
        "crates/example/src/_tests_/lib_tests.rs:1: `impl ExampleTrait for ExampleStub`"
    ));
}

#[cfg(unix)]
#[test]
fn audit_skips_symlinked_test_trees() {
    let workspace_root = test_workspace();
    write_file(
        &workspace_root,
        "external-tests/leaked_tests.rs",
        "impl ExampleTrait for ExampleStub {}\n",
    );
    fs::create_dir_all(workspace_root.join("crates/example/src/_tests_")).expect("test root");
    std::os::unix::fs::symlink(
        workspace_root.join("external-tests"),
        workspace_root.join("crates/example/src/_tests_/linked-tests"),
    )
    .expect("test symlink");

    run_rust_trait_audit(&workspace_root).expect("symlinked trees are outside the audit path");
}

fn test_workspace() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let counter = TEST_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "xtask-rust-trait-audit-{}-{unique}-{counter}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("workspace root reset");
    }
    fs::create_dir_all(&root).expect("workspace root created");
    write_file(
        &root,
        TRAIT_GUIDANCE_PATH,
        "# Rust trait boundaries\n\nDocumented for tests.\n",
    );
    write_approved_test_double_fixtures(&root);

    root
}

fn write_approved_test_double_fixtures(workspace_root: &Path) {
    let mut fixtures = BTreeMap::<&str, String>::new();
    for approved in APPROVED_TEST_DOUBLES {
        fixtures
            .entry(approved.path)
            .or_default()
            .push_str(&format!(
                "impl {} for {} {{\n}}\n",
                approved.trait_name, approved.impl_name
            ));
    }

    for (path, contents) in fixtures {
        write_file(workspace_root, path, &contents);
    }
}

fn write_file(workspace_root: &Path, relative_path: &str, contents: &str) {
    let path = workspace_root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent created");
    }
    fs::write(path, contents).expect("file written");
}
