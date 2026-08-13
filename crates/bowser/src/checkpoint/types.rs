//! Portable checkpoint data types.

use chromiumoxide::cdp::browser_protocol::network::{
    CookiePartitionKey, CookiePriority, CookieSameSite, CookieSourceScheme,
};

/// Current portable checkpoint format version.
pub const CHECKPOINT_VERSION: u32 = 1;

/// Portable browser-session checkpoint.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionCheckpoint {
    /// Checkpoint schema version.
    pub checkpoint: u32,
    /// Timestamp at which the live state was captured.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// All cookies visible to the browser context, including HttpOnly cookies.
    pub cookies: Vec<CheckpointCookie>,
    /// localStorage entries for HTTP(S) origins represented by open tabs.
    pub origins: Vec<CheckpointOrigin>,
    /// Open top-level tabs in their Bowser page order.
    pub pages: Vec<CheckpointPage>,
    /// Zero-based selected-page index in `pages`.
    pub selected_page: usize,
}

impl SessionCheckpoint {
    /// Validates the versioned checkpoint contract before restore.
    pub fn validate(&self) -> crate::Result<()> {
        if self.checkpoint != CHECKPOINT_VERSION {
            return Err(crate::Error::CheckpointUnsupported {
                version: self.checkpoint,
            });
        }
        if self.pages.is_empty() {
            return Err(crate::Error::CheckpointInvalid {
                reason: "checkpoint must contain at least one page".to_string(),
            });
        }
        if self.selected_page >= self.pages.len() {
            return Err(crate::Error::CheckpointInvalid {
                reason: "selected_page must index an existing page".to_string(),
            });
        }
        if self.pages.iter().any(|page| page.url.is_empty()) {
            return Err(crate::Error::CheckpointInvalid {
                reason: "page URLs must not be empty".to_string(),
            });
        }
        for origin in &self.origins {
            let parsed =
                url::Url::parse(&origin.origin).map_err(|_| crate::Error::CheckpointInvalid {
                    reason: format!("invalid localStorage origin: {}", origin.origin),
                })?;
            if !matches!(parsed.scheme(), "http" | "https")
                || parsed.origin().ascii_serialization() != origin.origin
            {
                return Err(crate::Error::CheckpointInvalid {
                    reason: format!(
                        "localStorage key is not an HTTP(S) origin: {}",
                        origin.origin
                    ),
                });
            }
        }
        Ok(())
    }
}

/// One open top-level tab in a checkpoint.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CheckpointPage {
    /// URL reopened during restore.
    pub url: String,
}

/// localStorage state for one security origin.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CheckpointOrigin {
    /// Canonical HTTP(S) origin.
    pub origin: String,
    /// Complete localStorage key/value entries for the origin.
    pub local_storage: Vec<CheckpointStorageEntry>,
}

/// One localStorage key/value pair.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CheckpointStorageEntry {
    /// localStorage key.
    pub key: String,
    /// localStorage value.
    pub value: String,
}

/// Portable subset of a Chrome cookie record needed to recreate the cookie.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CheckpointCookie {
    /// Cookie name.
    pub name: String,
    /// Cookie value.
    pub value: String,
    /// Cookie domain.
    pub domain: String,
    /// Cookie path.
    pub path: String,
    /// Expiry time in Unix seconds; absent for session cookies.
    pub expires: Option<f64>,
    /// Whether the cookie requires secure transport.
    pub secure: bool,
    /// Whether JavaScript must not observe the cookie.
    pub http_only: bool,
    /// SameSite policy reported by Chrome.
    pub same_site: Option<CookieSameSite>,
    /// Cookie priority reported by Chrome.
    pub priority: CookiePriority,
    /// Source scheme reported by Chrome.
    pub source_scheme: CookieSourceScheme,
    /// Source port reported by Chrome.
    pub source_port: i64,
    /// Optional partition key for partitioned cookies.
    pub partition_key: Option<CookiePartitionKey>,
}

/// Counts included in checkpoint CLI results.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CheckpointSummary {
    /// Checkpoint format version.
    pub version: u32,
    /// Number of cookies.
    pub cookies: usize,
    /// Number of localStorage origins.
    pub origins: usize,
    /// Number of open tabs.
    pub pages: usize,
}

impl From<&SessionCheckpoint> for CheckpointSummary {
    fn from(checkpoint: &SessionCheckpoint) -> Self {
        Self {
            version: checkpoint.checkpoint,
            cookies: checkpoint.cookies.len(),
            origins: checkpoint.origins.len(),
            pages: checkpoint.pages.len(),
        }
    }
}
