use std::path::PathBuf;
use std::process::ExitStatus;

use thiserror::Error as ThisError;

use crate::check::CheckCommand;

#[expect(
    unreachable_pub,
    reason = "unimock's generated public mock traits expose this result alias"
)]
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
#[expect(
    unreachable_pub,
    reason = "unimock's generated public mock traits expose this error type"
)]
pub enum Error {
    #[error("[xtask/check] command failed: {command} ({status})")]
    CommandFailed {
        command: CheckCommand,
        status: ExitStatus,
    },
    #[error("[xtask/check] failed to start command: {command}: {source}")]
    CommandStart {
        command: CheckCommand,
        source: std::io::Error,
    },
    #[error("[xtask/review] failed to start {command}: {source}")]
    ReviewGitCommandStart {
        command: String,
        source: std::io::Error,
    },
    #[error("[xtask/review] {command} failed ({status}){stderr}")]
    ReviewGitCommandFailed {
        command: String,
        status: ExitStatus,
        stderr: String,
    },
    #[error("[xtask/review] failed to create isolated review directory {path:?}: {source}")]
    ReviewSandboxCreate {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("[xtask/review] failed to canonicalize review path {path:?}: {source}")]
    ReviewSandboxCanonicalize {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(
        "[xtask/review] no neutral temp directory found outside workspace {workspace_root:?}; checked {checked_roots:?}"
    )]
    ReviewSandboxRootInsideWorkspace {
        workspace_root: PathBuf,
        checked_roots: Vec<PathBuf>,
    },
    #[error("[xtask/review] failed to start review command `{command}`: {source}")]
    ReviewCodexStart {
        command: String,
        source: std::io::Error,
    },
    #[error("[xtask/review] failed to write prompt to `{command}`: {source}")]
    ReviewCodexPromptWrite {
        command: String,
        source: std::io::Error,
    },
    #[error("[xtask/review] failed to wait for `{command}`: {source}")]
    ReviewCodexWait {
        command: String,
        source: std::io::Error,
    },
    #[error("[xtask/review] review command `{command}` failed ({status})")]
    ReviewCodexFailed { command: String, status: ExitStatus },
    #[error(
        "[xtask/review] omitted untracked path report exceeded cap {cap}: {hidden_count} additional path(s) hidden"
    )]
    ReviewOmittedPathReportExceeded { cap: usize, hidden_count: usize },
    #[error("[xtask/check] xtask manifest directory has no parent: {manifest_dir:?}")]
    MissingWorkspaceRoot { manifest_dir: PathBuf },
    #[error("[xtask/rust_trait_audit] path is not UTF-8: {path:?}")]
    RustTraitAuditNonUtf8Path { path: PathBuf },
    #[error(
        "[xtask/rust_trait_audit] failed to compute relative path for {path:?} from {workspace_root:?}: {source}"
    )]
    RustTraitAuditRelativePath {
        workspace_root: PathBuf,
        path: PathBuf,
        source: std::path::StripPrefixError,
    },
    #[error("[xtask/rust_trait_audit] failed to read directory {path:?}: {source}")]
    RustTraitAuditReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("[xtask/rust_trait_audit] failed to read file {path:?}: {source}")]
    RustTraitAuditReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("[xtask/rust_trait_audit] required guidance doc missing: {path:?}")]
    RustTraitAuditMissingDoc { path: PathBuf },
    #[error("[xtask/rust_trait_audit] {details}")]
    RustTraitAuditViolations { details: String },
    #[error(
        "[xtask/check] source-adjacent test directory must be named `_tests_`: {path}. hint: rename it to `{suggested_path}`. crate-root integration tests may stay in `tests/`."
    )]
    SourceAdjacentTestsDirectory {
        path: PathBuf,
        suggested_path: PathBuf,
    },
    #[error(
        "[xtask/check] inline Rust test attribute is not allowed outside `_tests_`: {path}:{line}"
    )]
    InlineRustTestAttribute { path: PathBuf, line: usize },
    #[error(
        "[xtask/check] `#[cfg(test)]` in source files must be followed by an external `_tests_` module declaration: {path}:{line}"
    )]
    InvalidCfgTestUsage { path: PathBuf, line: usize },
    #[error("[xtask/check] failed to read source file `{path}`: {source}")]
    ReadSourceFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("[xtask/check] failed to read directory `{path}`: {source}")]
    ReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("[xtask/rust_source_audit] failed to start `cargo metadata`: {source}")]
    RustSourceAuditMetadataCommandStart { source: std::io::Error },
    #[error("[xtask/rust_source_audit] `cargo metadata` failed ({status}){stderr}")]
    RustSourceAuditMetadataCommandFailed { status: ExitStatus, stderr: String },
    #[error("[xtask/rust_source_audit] failed to parse `cargo metadata` output: {source}")]
    RustSourceAuditMetadataParse { source: serde_json::Error },
    #[error("[xtask/rust_source_audit] failed to locate package root for {manifest_path:?}")]
    RustSourceAuditPackageRoot { manifest_path: PathBuf },
    #[error("[xtask/rust_source_audit] failed to read directory `{path}`: {source}")]
    RustSourceAuditReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("[xtask/rust_source_audit] failed to read Rust file `{path}`: {source}")]
    RustSourceAuditReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("[xtask/rust_source_audit] failed to parse Rust file `{path}`: {source}")]
    RustSourceAuditParseFile { path: PathBuf, source: syn::Error },
    #[error("[xtask/rust_source_audit] found {count} orphan Rust file(s):\n{details}")]
    RustSourceAuditViolations { count: usize, details: String },
    #[error("[xtask/rust_file_length_lint] failed to start git {command}: {source}")]
    RustFileLengthGitCommandStart {
        command: String,
        source: std::io::Error,
    },
    #[error("[xtask/rust_file_length_lint] git {command} failed ({status}){stderr}")]
    RustFileLengthGitCommandFailed {
        command: String,
        status: ExitStatus,
        stderr: String,
    },
    #[error("[xtask/rust_file_length_lint] failed to read Rust file `{path}`: {source}")]
    RustFileLengthReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(
        "[xtask/rust_file_length_lint] Rust file exceeds max lines: {path} has {line_count} lines; allowed max is {max_lines}"
    )]
    RustFileTooLong {
        path: PathBuf,
        line_count: usize,
        max_lines: usize,
    },
    #[error("[xtask/rust_file_length_lint] found {count} violation(s):\n{details}")]
    RustFileLengthViolations { count: usize, details: String },
}
