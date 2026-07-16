use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

static TEMP_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn rejects_orphan_source_file() {
    let workspace = temp_workspace();
    write_workspace_file(workspace.path(), "crates/demo/src/lib.rs", "mod used;\n");
    write_workspace_file(
        workspace.path(),
        "crates/demo/src/used.rs",
        "pub(crate) fn used() {}\n",
    );
    write_workspace_file(
        workspace.path(),
        "crates/demo/src/orphan.rs",
        "pub(crate) fn orphan() {}\n",
    );

    let error = run_rust_source_audit(workspace.path()).unwrap_err();

    assert!(matches!(
        error,
        Error::RustSourceAuditViolations { count: 1, .. }
    ));
    assert!(
        error.to_string().contains("crates/demo/src/orphan.rs"),
        "{error}"
    );
}

#[test]
fn accepts_inline_and_path_modules() {
    let workspace = temp_workspace();
    write_workspace_file(
        workspace.path(),
        "crates/demo/src/lib.rs",
        "mod outer {\n    mod child;\n}\n#[cfg(test)]\n#[path = \"_tests_/lib_tests.rs\"]\nmod lib_tests;\n",
    );
    write_workspace_file(
        workspace.path(),
        "crates/demo/src/outer/child.rs",
        "pub(crate) fn child() {}\n",
    );
    write_workspace_file(
        workspace.path(),
        "crates/demo/src/_tests_/lib_tests.rs",
        "mod helper;\n#[test]\nfn reaches_test_module() {\n    helper::ready();\n}\n",
    );
    write_workspace_file(
        workspace.path(),
        "crates/demo/src/_tests_/helper.rs",
        "pub(crate) fn ready() {}\n",
    );

    run_rust_source_audit(workspace.path()).unwrap();
}

#[test]
fn accepts_integration_test_support_modules() {
    let workspace = temp_workspace();
    write_workspace_file(
        workspace.path(),
        "crates/demo/src/lib.rs",
        "pub fn demo() {}\n",
    );
    write_workspace_file(
        workspace.path(),
        "crates/demo/tests/cli_surface.rs",
        "mod support;\n#[test]\nfn smoke() {\n    support::helper();\n}\n",
    );
    write_workspace_file(
        workspace.path(),
        "crates/demo/tests/support.rs",
        "pub(crate) fn helper() {}\n",
    );

    run_rust_source_audit(workspace.path()).unwrap();
}

struct TempWorkspace {
    path: std::path::PathBuf,
}

impl TempWorkspace {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn temp_workspace() -> TempWorkspace {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique = TEMP_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "xtask-rust-source-audit-{}-{nanos}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    write_workspace_file(
        &path,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/demo\"]\nresolver = \"2\"\n",
    );
    write_workspace_file(
        &path,
        "crates/demo/Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    TempWorkspace { path }
}

fn write_workspace_file(workspace_root: &Path, relative_path: &str, contents: &str) {
    let path = workspace_root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}
