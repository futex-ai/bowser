//! Git path parsing helpers for local review context.

/// Path parsed from NUL-delimited Git output.
pub(crate) struct GitPath {
    display: String,
    readable: Option<String>,
}

impl GitPath {
    /// Build a prompt-safe path value from raw Git path bytes.
    pub(crate) fn from_bytes(path: &[u8]) -> Self {
        match std::str::from_utf8(path) {
            Ok(path) => Self {
                display: path.to_owned(),
                readable: Some(path.to_owned()),
            },
            Err(_) => Self {
                display: String::from_utf8_lossy(path).into_owned(),
                readable: None,
            },
        }
    }

    /// Path text for prompts and diagnostic summaries.
    pub(crate) fn display(&self) -> &str {
        &self.display
    }

    /// UTF-8 path text usable with normal Rust path APIs.
    pub(crate) fn readable(&self) -> Option<&str> {
        self.readable.as_deref()
    }
}

/// Parse `git -z` path output without trimming or splitting on newlines.
pub(crate) fn parse_git_paths(output: &[u8]) -> Vec<GitPath> {
    output
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(GitPath::from_bytes)
        .collect()
}
