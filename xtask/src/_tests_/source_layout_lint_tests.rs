use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

static TEMP_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn rejects_source_adjacent_tests_directory_named_tests() {
    let workspace = temp_workspace();
    fs::create_dir_all(workspace.path().join("crates/demo/src/tests")).unwrap();

    let error = verify_source_adjacent_test_layout(workspace.path()).unwrap_err();

    assert!(matches!(error, Error::SourceAdjacentTestsDirectory { .. }));
    assert_eq!(
        error.to_string(),
        "[xtask/check] source-adjacent test directory must be named `_tests_`: crates/demo/src/tests. hint: rename it to `crates/demo/src/_tests_`. crate-root integration tests may stay in `tests/`."
    );
}

#[test]
fn rejects_cfg_test_without_external_tests_module() {
    let workspace = temp_workspace();
    let source = workspace.path().join("crates/demo/src/lib.rs");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(
        &source,
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn x() {}\n}\n",
    )
    .unwrap();

    let error = verify_source_adjacent_test_layout(workspace.path()).unwrap_err();

    assert!(matches!(error, Error::InvalidCfgTestUsage { .. }));
}

#[test]
fn accepts_external_tests_module_declaration() {
    let workspace = temp_workspace();
    let source = workspace.path().join("crates/demo/src/lib.rs");
    let test_file = workspace
        .path()
        .join("crates/demo/src/_tests_/lib_tests.rs");
    fs::create_dir_all(test_file.parent().unwrap()).unwrap();
    fs::write(
        &source,
        "#[cfg(test)]\n#[path = \"_tests_/lib_tests.rs\"]\nmod lib_tests;\n",
    )
    .unwrap();
    fs::write(&test_file, "#[test]\nfn x() {}\n").unwrap();

    verify_source_adjacent_test_layout(workspace.path()).unwrap();
}

#[test]
fn accepts_external_test_support_module_with_visibility() {
    let workspace = temp_workspace();
    let source = workspace.path().join("crates/demo/src/client.rs");
    let test_file = workspace
        .path()
        .join("crates/demo/src/_tests_/client_test_support.rs");
    fs::create_dir_all(test_file.parent().unwrap()).unwrap();
    fs::write(
        &source,
        "#[cfg(test)]\n#[path = \"_tests_/client_test_support.rs\"]\npub(crate) mod client_test_support;\n",
    )
    .unwrap();
    fs::write(&test_file, "pub(crate) fn helper() {}\n").unwrap();

    verify_source_adjacent_test_layout(workspace.path()).unwrap();
}

#[test]
fn rejects_inline_test_attribute_in_source_file() {
    let workspace = temp_workspace();
    let source = workspace.path().join("crates/demo/src/lib.rs");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "#[test]\nfn x() {}\n").unwrap();

    let error = verify_source_adjacent_test_layout(workspace.path()).unwrap_err();

    assert!(matches!(error, Error::InlineRustTestAttribute { .. }));
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
        "bowser-xtask-source-layout-{}-{nanos}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    TempWorkspace { path }
}
