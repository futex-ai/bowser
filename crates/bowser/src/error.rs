//! Error types for the Bowser library.

/// Error result alias for the Bowser library.
pub type Result<T> = std::result::Result<T, Error>;

const TARGET_SESSION_MISSING_REASON: &str = "Session with given id not found";
const TARGET_RECEIVER_GONE_REASON: &str = "send failed because receiver is gone";

/// Library error variants.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("[bowser/browser] failed to launch Chrome: {reason}")]
    BrowserLaunch { reason: String },

    #[error("[bowser/browser] Chrome connection lost")]
    BrowserDisconnected,

    #[error("[bowser/browser] page target session invalid: {reason}")]
    PageTargetSessionInvalid { reason: String },

    #[error("[bowser/browser] failed to close page {page_id}: {reason}")]
    PageClose { page_id: String, reason: String },

    #[error("[bowser/browser] timed out closing page {page_id} after {seconds}s")]
    PageCloseTimeout { page_id: String, seconds: u64 },

    #[error("[bowser/session] session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("[bowser/session] session expired: {session_id}")]
    SessionExpired { session_id: String },

    /// The supplied identifier cannot name a file-backed Bowser session.
    #[error("[bowser/session] invalid session identifier: {session_id}")]
    InvalidSessionId {
        /// Rejected identifier.
        session_id: String,
    },

    /// A stored metadata document claims a different identifier than its filename.
    #[error(
        "[bowser/session] metadata identifier mismatch: expected {expected_session_id}, found {actual_session_id}"
    )]
    SessionIdMismatch {
        /// Identifier derived from the requested metadata filename.
        expected_session_id: String,
        /// Identifier contained in the metadata document.
        actual_session_id: String,
    },

    /// Metadata marked a profile as owned, but its path is outside the generated profile root.
    #[error("[bowser/session] invalid owned profile path: {path}")]
    InvalidOwnedProfilePath {
        /// Rejected profile path.
        path: std::path::PathBuf,
    },

    /// Bowser could not remove one of its generated profile paths.
    #[error("[bowser/session] failed to remove owned profile {path}: {source}")]
    SessionProfileRemove {
        /// Profile path Bowser attempted to remove.
        path: std::path::PathBuf,
        /// Filesystem failure returned by the operating system.
        source: std::io::Error,
    },

    /// The blocking owned-profile cleanup worker could not be joined.
    #[error("[bowser/session] owned profile cleanup worker failed: {source}")]
    SessionProfileTask {
        /// Runtime join failure for the cleanup worker.
        source: tokio::task::JoinError,
    },

    /// Chrome retained the exact persisted session identity after forced termination.
    #[error("[bowser/session] Chrome process {pid} remained active after forced termination")]
    SessionProcessStillRunning {
        /// Persisted Chrome process identifier that remained active.
        pid: u32,
    },

    /// The persisted process no longer has the Chrome port and profile identity for the session.
    #[error("[bowser/session] session process identity no longer matches: {session_id}")]
    SessionProcessIdentityMismatch {
        /// Session that cannot be resumed safely.
        session_id: String,
    },

    #[error("[bowser/session] page not found: {page_id}")]
    SessionPageNotFound { page_id: String },

    #[error("[bowser/session] page is no longer live: {page_id}")]
    SessionPageNotLive { page_id: String },

    #[error("[bowser/session] failed to persist session metadata: {reason}")]
    SessionPersist { reason: String },

    #[error("[bowser/session] failed to detach session: {reason}")]
    SessionDetach { reason: String },

    #[error("[bowser/expand] element not expandable: {element_id}")]
    ExpandNotFound { element_id: u32 },

    #[error("[bowser/meta] metadata unavailable for element: {element_id}")]
    MetadataNotFound { element_id: u32 },

    #[error("[bowser/describe] element is not an image: {element_id}")]
    DescribeNotImage { element_id: u32 },

    #[error("[bowser/describe] image description unavailable for element: {element_id}")]
    DescribeUnavailable { element_id: u32 },

    #[error("[bowser/page] navigation failed: {url}")]
    Navigation { url: String },

    #[error("[bowser/page] page load timed out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("[bowser/page] element not found: {element_id}")]
    ElementNotFound { element_id: u32 },

    #[error("[bowser/page] element {element_id} is not interactable")]
    ElementNotInteractable { element_id: u32 },

    #[error("[bowser/page] key press failed: {reason}")]
    KeyPress { reason: String },

    #[error("[bowser/capture] DOM capture script failed: {reason}")]
    CaptureScript { reason: String },

    #[error("[bowser/capture] failed to parse capture result")]
    CaptureParse,

    #[error("[bowser/stealth] failed to inject stealth patches: {reason}")]
    StealthInjection { reason: String },

    #[error("[bowser/ai] image summarization failed: {reason}")]
    AiSummarization { reason: String },

    #[error("[bowser/config] invalid configuration: {reason}")]
    Config { reason: String },

    #[error("[bowser/screenshot] screenshot failed: {reason}")]
    Screenshot { reason: String },

    #[error("[bowser/download] invalid destination {path}: {reason}")]
    DownloadDestination { path: String, reason: String },

    #[error("[bowser/download] download failed: {reason}")]
    Download { reason: String },

    /// Chrome rejected the navigation used to initiate a browser-native download.
    #[error("[bowser/download] failed to start browser download: {source}")]
    DownloadStart {
        /// Chrome DevTools Protocol failure returned by the navigation assignment.
        #[source]
        source: chromiumoxide::error::CdpError,
    },

    #[error("[bowser/download] download timed out after {seconds}s")]
    DownloadTimeout { seconds: u64 },

    #[error("[bowser/js] JavaScript evaluation failed: {reason}")]
    JsEvaluation { reason: String },

    #[error("[bowser/io] {operation}: {source}")]
    Io {
        operation: String,
        source: std::io::Error,
    },

    #[error("[bowser/chrome] CDP protocol error: {reason}")]
    Cdp { reason: String },
}

impl Error {
    /// Creates a config error.
    pub fn config(reason: impl Into<String>) -> Self {
        Self::Config {
            reason: reason.into(),
        }
    }

    /// Creates a session persistence error.
    pub fn session(reason: impl Into<String>) -> Self {
        Self::SessionPersist {
            reason: reason.into(),
        }
    }

    /// Creates an I/O error.
    pub fn io(operation: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            operation: operation.into(),
            source,
        }
    }

    /// Creates a CDP error.
    pub fn cdp(reason: impl Into<String>) -> Self {
        Self::Cdp {
            reason: reason.into(),
        }
    }
}

pub(crate) fn is_target_session_missing(reason: &str) -> bool {
    reason.contains(TARGET_SESSION_MISSING_REASON) || reason.contains(TARGET_RECEIVER_GONE_REASON)
}

#[cfg(test)]
#[path = "_tests_/error_tests.rs"]
mod error_tests;
