//! Session summary and display helpers.

use super::metadata::{SessionMetadata, SessionPageMetadata};
use crate::model::{SessionPageSummary, SessionSummary};

impl SessionPageMetadata {
    /// Returns the best available page URL.
    pub fn url(&self) -> Option<String> {
        self.last_url.clone().or_else(|| {
            self.preview_capture
                .as_ref()
                .map(|capture| capture.url.clone())
                .or_else(|| {
                    self.full_capture
                        .as_ref()
                        .map(|capture| capture.url.clone())
                })
        })
    }

    /// Returns the best available page title.
    pub fn title(&self) -> Option<String> {
        self.last_title.clone().or_else(|| {
            self.preview_capture
                .as_ref()
                .map(|capture| capture.title.clone())
                .or_else(|| {
                    self.full_capture
                        .as_ref()
                        .map(|capture| capture.title.clone())
                })
        })
    }

    /// Returns true when the stored page url is non-empty and not blank.
    pub fn has_non_blank_url(&self) -> bool {
        self.url()
            .is_some_and(|url| !url.is_empty() && url != "about:blank")
    }

    /// Updates the cached url/title from live browser state.
    pub fn update_live_state(&mut self, url: String, title: String) {
        self.last_url = Some(url);
        self.last_title = Some(title);
    }
}

impl SessionMetadata {
    /// Creates a summary view.
    pub fn summary(&self) -> SessionSummary {
        let page = self.summary_page();
        SessionSummary {
            id: self.id.clone(),
            url: page.and_then(SessionPageMetadata::url),
            title: page.and_then(SessionPageMetadata::title),
            updated_at: self.updated_at,
            pid: self.pid,
            user_data_dir: self.user_data_dir.clone(),
        }
    }

    /// Returns a selected-or-best page for summary output.
    pub fn summary_page(&self) -> Option<&SessionPageMetadata> {
        self.selected_page()
            .filter(|page| page.has_non_blank_url())
            .or_else(|| self.pages.iter().find(|page| page.has_non_blank_url()))
            .or_else(|| self.selected_page())
            .or_else(|| self.pages.first())
    }

    /// Returns stored page summaries annotated with live/selected state.
    pub fn page_summaries<I>(&self, live_page_ids: I) -> Vec<SessionPageSummary>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let live_page_ids: Vec<String> = live_page_ids
            .into_iter()
            .map(|value| value.as_ref().to_string())
            .collect();
        self.pages
            .iter()
            .map(|page| SessionPageSummary {
                id: page.id.clone(),
                page_type: page.page_type,
                url: page.url(),
                title: page.title(),
                selected: self.selected_page_id.as_deref() == Some(page.id.as_str()),
                live: live_page_ids.iter().any(|value| value == &page.id),
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "_tests_/summary_tests.rs"]
mod summary_tests;
