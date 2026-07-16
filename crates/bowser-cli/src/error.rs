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
