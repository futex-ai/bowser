//! Browser-native download result types.

use std::path::PathBuf;

/// Result of a browser-native file download.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DownloadResult {
    /// Destination path where the completed file was written.
    pub path: PathBuf,
    /// Final filename observed on disk after Chrome completed the download.
    pub filename: String,
    /// Completed file size in bytes.
    pub bytes: u64,
}
