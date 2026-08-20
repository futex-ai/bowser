//! CLI error types.

use std::net::SocketAddr;

/// CLI result alias.
pub type Result<T> = std::result::Result<T, CliError>;

/// CLI errors.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("[bowser-cli] {0}")]
    Bowser(#[from] bowser::Error),

    #[error("[bowser-cli/io] failed to write output: {path}")]
    OutputWrite { path: String },

    #[error("[bowser-cli/output] failed to serialize command output")]
    OutputSerialize { source: serde_json::Error },

    #[error("[bowser-cli/arguments] {message}")]
    Arguments { message: String },

    /// Bowser could not load or validate the requested configuration.
    #[error("[bowser-cli/config] failed to load config {path}: {source}")]
    ConfigLoad {
        /// Configuration path used for the load attempt.
        path: String,
        /// Typed library failure that explains the load error.
        #[source]
        source: bowser::Error,
    },

    #[error("[bowser-cli/repl] invalid command: {input}")]
    InvalidCommand { input: String },

    #[error("[bowser-cli/repl] readline failed: {reason}")]
    Readline { reason: String },

    #[error("[bowser-cli/repl] failed to install signal handler: {reason}")]
    SignalHandler { reason: String },

    #[error("[bowser-cli/pointer-log] bind address must be loopback: {addr}")]
    PointerLogNonLoopback { addr: SocketAddr },

    #[error("[bowser-cli/pointer-log] failed to bind server at {addr}")]
    PointerLogBind {
        addr: SocketAddr,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to read bound server address")]
    PointerLogLocalAddr { source: std::io::Error },

    #[error("[bowser-cli/pointer-log] server failed at {addr}")]
    PointerLogServe {
        addr: SocketAddr,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to create log directory: {path}")]
    PointerLogCreateDir {
        path: String,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to open log file: {path}")]
    PointerLogOpen {
        path: String,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to lock log file: {path}")]
    PointerLogLock { path: String },

    #[error("[bowser-cli/pointer-log] failed to serialize event for log file: {path}")]
    PointerLogSerialize {
        path: String,
        source: serde_json::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to append log file: {path}")]
    PointerLogAppend {
        path: String,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] demo target was not found in the current capture")]
    PointerLogDemoTargetMissing,

    #[error("[bowser-cli/pointer-log] demo browser config was not loaded")]
    PointerLogDemoConfigMissing,

    #[error("[bowser-cli/pointer-log] failed to listen for Ctrl-C")]
    PointerLogSignal { source: std::io::Error },
}

impl CliError {
    pub(crate) fn machine_code_and_detail(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::Bowser(error) => bowser_error_code_and_detail(error),
            Self::OutputWrite { path } => ("output_write", serde_json::json!({ "path": path })),
            Self::OutputSerialize { .. } => ("internal", serde_json::json!({})),
            Self::Arguments { .. } => ("invalid_arguments", serde_json::json!({})),
            Self::ConfigLoad { path, .. } => ("configuration", serde_json::json!({ "path": path })),
            Self::InvalidCommand { input } => {
                ("invalid_command", serde_json::json!({ "input": input }))
            }
            Self::Readline { .. }
            | Self::SignalHandler { .. }
            | Self::PointerLogNonLoopback { .. }
            | Self::PointerLogBind { .. }
            | Self::PointerLogLocalAddr { .. }
            | Self::PointerLogServe { .. }
            | Self::PointerLogCreateDir { .. }
            | Self::PointerLogOpen { .. }
            | Self::PointerLogLock { .. }
            | Self::PointerLogSerialize { .. }
            | Self::PointerLogAppend { .. }
            | Self::PointerLogDemoTargetMissing
            | Self::PointerLogDemoConfigMissing
            | Self::PointerLogSignal { .. } => ("internal", serde_json::json!({})),
        }
    }
}

fn bowser_error_code_and_detail(error: &bowser::Error) -> (&'static str, serde_json::Value) {
    use bowser::Error;
    match error {
        Error::BrowserLaunch { .. } => ("browser_launch", serde_json::json!({})),
        Error::BrowserDisconnected => ("browser_disconnected", serde_json::json!({})),
        Error::PageTargetSessionInvalid { .. } => ("page_target_invalid", serde_json::json!({})),
        Error::PageClose { page_id, .. } => {
            ("page_close", serde_json::json!({ "page_id": page_id }))
        }
        Error::PageCloseTimeout { page_id, seconds } => (
            "page_close_timeout",
            serde_json::json!({ "page_id": page_id, "seconds": seconds }),
        ),
        Error::SessionNotFound { session_id } => (
            "session_not_found",
            serde_json::json!({ "session_id": session_id }),
        ),
        Error::SessionExpired { session_id } => (
            "session_expired",
            serde_json::json!({ "session_id": session_id }),
        ),
        Error::InvalidSessionId { session_id } => (
            "invalid_session_id",
            serde_json::json!({ "session_id": session_id }),
        ),
        Error::SessionIdMismatch {
            expected_session_id,
            actual_session_id,
        } => (
            "session_identity_mismatch",
            serde_json::json!({ "expected_session_id": expected_session_id, "actual_session_id": actual_session_id }),
        ),
        Error::InvalidOwnedProfilePath { path } => (
            "session_identity_mismatch",
            serde_json::json!({ "path": path }),
        ),
        Error::SessionProfileRemove { path, .. } => ("io", serde_json::json!({ "path": path })),
        Error::SessionProfileTask { .. } => ("internal", serde_json::json!({})),
        Error::SessionProcessStillRunning { pid } => (
            "session_identity_mismatch",
            serde_json::json!({ "pid": pid }),
        ),
        Error::SessionProcessIdentityMismatch { session_id } => (
            "session_identity_mismatch",
            serde_json::json!({ "session_id": session_id }),
        ),
        Error::SessionPageNotFound { page_id } => (
            "session_page_not_found",
            serde_json::json!({ "page_id": page_id }),
        ),
        Error::SessionPageNotLive { page_id } => (
            "session_page_not_live",
            serde_json::json!({ "page_id": page_id }),
        ),
        Error::SessionPersist { .. } => ("session_persist", serde_json::json!({})),
        Error::SessionDetach { .. } => ("session_detach", serde_json::json!({})),
        Error::CheckpointRead { path, .. } => {
            ("checkpoint_read", serde_json::json!({ "path": path }))
        }
        Error::CheckpointWrite { path, .. } => {
            ("checkpoint_write", serde_json::json!({ "path": path }))
        }
        Error::CheckpointInvalid { .. } => ("checkpoint_invalid", serde_json::json!({})),
        Error::CheckpointUnsupported { version } => (
            "checkpoint_unsupported",
            serde_json::json!({ "version": version }),
        ),
        Error::CheckpointCapture { .. } => ("checkpoint_capture", serde_json::json!({})),
        Error::CheckpointRestore { session_id, .. } => (
            "checkpoint_restore",
            serde_json::json!({ "session_id": session_id }),
        ),
        Error::ExpandNotFound { element_id } => (
            "expand_not_found",
            serde_json::json!({ "element_id": element_id }),
        ),
        Error::MetadataNotFound { element_id } => (
            "metadata_not_found",
            serde_json::json!({ "element_id": element_id }),
        ),
        Error::DescribeNotImage { element_id } => (
            "describe_not_image",
            serde_json::json!({ "element_id": element_id }),
        ),
        Error::DescribeUnavailable { element_id } => (
            "describe_unavailable",
            serde_json::json!({ "element_id": element_id }),
        ),
        Error::Navigation { url } => ("navigation", serde_json::json!({ "url": url })),
        Error::HistoryRead { .. } | Error::HistoryNavigation { .. } => {
            ("cdp", serde_json::json!({}))
        }
        Error::HistoryBackExhausted => (
            "history_exhausted",
            serde_json::json!({ "direction": "back" }),
        ),
        Error::HistoryForwardExhausted => (
            "history_exhausted",
            serde_json::json!({ "direction": "forward" }),
        ),
        Error::Timeout { seconds } => ("timeout", serde_json::json!({ "seconds": seconds })),
        Error::ElementNotFound { element_id } => (
            "element_not_found",
            serde_json::json!({ "element_id": element_id }),
        ),
        Error::ElementNotInteractable { element_id } => (
            "element_not_interactable",
            serde_json::json!({ "element_id": element_id }),
        ),
        Error::KeyPress { .. } => ("key_press", serde_json::json!({})),
        Error::CaptureScript { .. } | Error::CaptureParse => ("capture", serde_json::json!({})),
        Error::StealthInjection { .. } => ("stealth", serde_json::json!({})),
        Error::AiSummarization { .. } | Error::AiSummarizationTimeout { .. } => {
            ("ai", serde_json::json!({}))
        }
        Error::ConfigFileNotFound { path } => {
            ("configuration", serde_json::json!({ "path": path }))
        }
        Error::Config { .. } => ("configuration", serde_json::json!({})),
        Error::Screenshot { .. } => ("screenshot", serde_json::json!({})),
        Error::DownloadDestination { path, .. } => {
            ("download", serde_json::json!({ "path": path }))
        }
        Error::Download { .. } | Error::DownloadStart { .. } | Error::DownloadTimeout { .. } => {
            ("download", serde_json::json!({}))
        }
        Error::JsEvaluation { .. } => ("javascript", serde_json::json!({})),
        Error::Io { operation, .. } => ("io", serde_json::json!({ "operation": operation })),
        Error::Cdp { .. } => ("cdp", serde_json::json!({})),
    }
}
