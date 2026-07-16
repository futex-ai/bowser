//! Omitted-path reporting caps for local review prompts.

use crate::review_paths::GitPath;

/// Maximum omitted untracked paths included verbatim in the prompt.
pub(crate) const MAX_OMITTED_UNTRACKED_PATHS: usize = 128;

/// Capped omitted-path report for untracked review safeguards.
pub(crate) struct OmittedPathReport {
    paths: Vec<String>,
    hidden_count: usize,
    cap: usize,
}

impl OmittedPathReport {
    /// Create an omitted-path report with a maximum listed-path count.
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            paths: Vec::new(),
            hidden_count: 0,
            cap,
        }
    }

    /// Record an omitted path, hiding it once the reporting cap is reached.
    pub(crate) fn push(&mut self, path: String) {
        if self.paths.len() < self.cap {
            self.paths.push(path);
        } else {
            self.hidden_count += 1;
        }
    }

    /// Number of omitted paths intentionally not listed.
    pub(crate) fn hidden_count(&self) -> usize {
        self.hidden_count
    }

    /// Append a capped omitted-path summary to the synthetic diff list.
    pub(crate) fn append_summary(&self, diffs: &mut Vec<String>) {
        if self.paths.is_empty() && self.hidden_count == 0 {
            return;
        }

        let mut summary = format!(
            "### untracked files omitted\n[skipped or omitted by untracked review safeguards; reporting cap is {} paths]\n",
            self.cap
        );
        if !self.paths.is_empty() {
            summary.push_str(&self.paths.join("\n"));
            summary.push('\n');
        }
        if self.hidden_count > 0 {
            summary.push_str(&format!(
                "[{} additional omitted untracked paths not listed]\n",
                self.hidden_count
            ));
        }

        diffs.push(summary);
    }

    /// Return listed omitted paths.
    pub(crate) fn into_paths(self) -> Vec<String> {
        self.paths
    }
}

/// Record the current untracked path and every path after it as omitted.
pub(crate) fn omit_current_and_remaining(
    untracked_files: &[GitPath],
    current: &GitPath,
    report: &mut OmittedPathReport,
) {
    let Some(current_index) = untracked_files
        .iter()
        .position(|path| std::ptr::eq(path, current))
    else {
        return;
    };

    for path in &untracked_files[current_index..] {
        report.push(path.display().to_owned());
    }
}
