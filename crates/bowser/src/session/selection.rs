//! Session page selection and mutation helpers.

use super::metadata::{SessionMetadata, SessionPageMetadata};

impl SessionMetadata {
    /// Returns the currently selected page metadata.
    pub fn selected_page(&self) -> Option<&SessionPageMetadata> {
        let selected = self.selected_page_id.as_deref()?;
        self.pages.iter().find(|page| page.id == selected)
    }

    /// Returns the currently selected page metadata mutably.
    pub fn selected_page_mut(&mut self) -> Option<&mut SessionPageMetadata> {
        let selected = self.selected_page_id.clone()?;
        self.pages.iter_mut().find(|page| page.id == selected)
    }

    /// Returns a page record by Bowser page ID.
    pub fn page_by_id(&self, page_id: &str) -> Option<&SessionPageMetadata> {
        self.pages.iter().find(|page| page.id == page_id)
    }

    /// Returns a mutable page record by Bowser page ID.
    pub fn page_by_id_mut(&mut self, page_id: &str) -> Option<&mut SessionPageMetadata> {
        self.pages.iter_mut().find(|page| page.id == page_id)
    }

    /// Returns a page record by live Chrome target ID.
    pub fn page_by_target_id(&self, target_id: &str) -> Option<&SessionPageMetadata> {
        self.pages.iter().find(|page| page.target_id == target_id)
    }

    /// Returns a mutable page record by live Chrome target ID.
    pub fn page_by_target_id_mut(&mut self, target_id: &str) -> Option<&mut SessionPageMetadata> {
        self.pages
            .iter_mut()
            .find(|page| page.target_id == target_id)
    }

    /// Ensures a page record exists for the given live Chrome target.
    pub fn ensure_page_for_target(&mut self, target_id: &str) -> &mut SessionPageMetadata {
        if let Some(index) = self
            .pages
            .iter()
            .position(|page| page.target_id == target_id)
        {
            return &mut self.pages[index];
        }
        let page_id = format!("pg_{}", self.next_page_ordinal.max(1));
        self.next_page_ordinal = self.next_page_ordinal.max(1) + 1;
        self.pages
            .push(SessionPageMetadata::new(page_id, target_id.to_string()));
        let last = self.pages.len() - 1;
        &mut self.pages[last]
    }

    /// Removes a page record by Bowser page ID.
    pub fn remove_page(&mut self, page_id: &str) -> Option<SessionPageMetadata> {
        let index = self.pages.iter().position(|page| page.id == page_id)?;
        Some(self.pages.remove(index))
    }

    /// Marks all known pages as requiring a fresh capture.
    pub fn mark_all_pages_stale(&mut self) {
        for page in &mut self.pages {
            page.requires_fresh_capture = true;
        }
    }
}
