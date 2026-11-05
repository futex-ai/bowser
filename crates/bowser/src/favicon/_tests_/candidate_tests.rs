//! Favicon declaration selection and ordering regressions.

use super::{FaviconDeclaration, FaviconDocument, candidate_urls, same_document_and_declarations};

#[test]
fn resolves_base_uri_and_ranks_declared_candidates() {
    let document = document(
        "https://site.test/account/page",
        "https://static.test/assets/",
        &[
            ("small.png", "16x16"),
            ("first-exact.png", "32x32"),
            ("vector.svg", "any"),
            ("last-exact.png", "16x16 32x32"),
        ],
    );

    assert_eq!(
        candidate_urls(&document),
        [
            "https://static.test/assets/last-exact.png",
            "https://static.test/assets/first-exact.png",
            "https://static.test/assets/vector.svg",
            "https://site.test/favicon.ico",
        ]
    );
}

#[test]
fn deduplicates_urls_limits_declarations_and_filters_schemes() {
    let document = document(
        "http://site.test/path",
        "http://site.test/base/",
        &[
            ("javascript:alert(1)", "32x32"),
            ("file:///tmp/icon.png", "32x32"),
            ("one.png", "16x16"),
            ("../base/one.png", "32x32"),
            ("two.png", "16x16"),
            ("three.png", "16x16"),
            ("four.png", "16x16"),
        ],
    );

    assert_eq!(
        candidate_urls(&document),
        [
            "http://site.test/base/one.png",
            "http://site.test/base/four.png",
            "http://site.test/base/three.png",
            "http://site.test/favicon.ico",
        ]
    );
}

#[test]
fn non_http_documents_have_no_candidates() {
    let document = document(
        "about:blank",
        "about:blank",
        &[("data:image/png;base64,AA==", "any")],
    );

    assert!(candidate_urls(&document).is_empty());
}

#[test]
fn identity_includes_loader_url_base_and_ranked_declarations() {
    let original = document(
        "https://site.test/page",
        "https://site.test/",
        &[("icon.png", "32x32")],
    );
    let mut changed_loader = original.clone();
    changed_loader.loader_id = "loader-2".to_string();
    let mut changed_declaration = original.clone();
    changed_declaration.declarations[0].href = "new.png".to_string();

    assert!(same_document_and_declarations(&original, &original));
    assert!(!same_document_and_declarations(&original, &changed_loader));
    assert!(!same_document_and_declarations(
        &original,
        &changed_declaration
    ));
}

fn document(url: &str, base_uri: &str, declarations: &[(&str, &str)]) -> FaviconDocument {
    FaviconDocument {
        loader_id: "loader-1".to_string(),
        url: url.to_string(),
        base_uri: base_uri.to_string(),
        declarations: declarations
            .iter()
            .map(|(href, sizes)| FaviconDeclaration {
                href: (*href).to_string(),
                sizes: (*sizes).to_string(),
            })
            .collect(),
    }
}
