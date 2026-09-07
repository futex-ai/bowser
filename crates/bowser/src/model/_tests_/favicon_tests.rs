//! Favicon inventory diagnostic privacy regressions.

use super::{BrowserFavicon, LiveSessionPage};

#[test]
fn debug_output_redacts_favicon_bytes_including_from_live_pages() {
    let secret = "private-png-base64";
    let favicon = BrowserFavicon {
        png_base64: secret.to_string(),
    };
    let page = LiveSessionPage {
        page_id: "page-1".to_string(),
        url: "https://example.test/".to_string(),
        title: Some("Example".to_string()),
        selected: true,
        favicon: Some(favicon.clone()),
    };

    assert_eq!(
        format!("{favicon:?}"),
        "BrowserFavicon { png_base64: \"[redacted]\" }"
    );
    let page_debug = format!("{page:?}");
    assert!(page_debug.contains("[redacted]"));
    assert!(!page_debug.contains(secret));
}
