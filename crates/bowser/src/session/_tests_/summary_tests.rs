//! Session summary tests.

use crate::{PageCapture, PageContent};

use super::SessionPageMetadata;

#[test]
fn live_page_state_takes_precedence_over_cached_capture_metadata() {
    let mut page = SessionPageMetadata::new("pg_1".to_string(), "target".to_string());
    page.preview_capture = Some(PageCapture {
        url: "https://example.test/cached".to_string(),
        title: "Cached".to_string(),
        body_id: None,
        obscured_body_id: None,
        content: PageContent::default(),
    });
    page.update_live_state("https://example.test/live".to_string(), "Live".to_string());

    assert_eq!(page.url().as_deref(), Some("https://example.test/live"));
    assert_eq!(page.title().as_deref(), Some("Live"));
}
