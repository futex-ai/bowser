//! Declared favicon ranking and document identity checks.

use std::collections::HashSet;

use url::Url;

const MAX_DECLARED_CANDIDATES: usize = 3;
const MAX_CANDIDATE_URL_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
/// One favicon declaration captured from a live document.
pub(crate) struct FaviconDeclaration {
    pub(crate) href: String,
    pub(crate) sizes: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// The document identity and favicon declarations used for a race check.
pub(crate) struct FaviconDocument {
    pub(crate) loader_id: String,
    pub(crate) url: String,
    pub(crate) base_uri: String,
    pub(crate) declarations: Vec<FaviconDeclaration>,
}

pub(crate) fn candidate_urls(document: &FaviconDocument) -> Vec<String> {
    let Ok(page_url) = Url::parse(&document.url) else {
        return Vec::new();
    };
    if !matches!(page_url.scheme(), "http" | "https") {
        return Vec::new();
    }
    let base_url = Url::parse(&document.base_uri)
        .ok()
        .filter(|base| matches!(base.scheme(), "http" | "https"))
        .unwrap_or_else(|| page_url.clone());
    let mut ranked = document
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(index, declaration)| {
            let url = resolve_candidate(&base_url, &declaration.href)?;
            Some((candidate_rank(&declaration.sizes), index, url))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));

    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    for (_, _, candidate) in ranked {
        if seen.insert(candidate.clone()) {
            candidates.push(candidate);
        }
        if candidates.len() == MAX_DECLARED_CANDIDATES {
            break;
        }
    }
    let mut root = page_url;
    root.set_path("/favicon.ico");
    root.set_query(None);
    root.set_fragment(None);
    let root = root.to_string();
    if seen.insert(root.clone()) {
        candidates.push(root);
    }
    candidates
}

pub(crate) fn same_document_and_declarations(
    initial: &FaviconDocument,
    current: &FaviconDocument,
) -> bool {
    initial == current
}

fn resolve_candidate(base_url: &Url, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || href.len() > MAX_CANDIDATE_URL_BYTES {
        return None;
    }
    let url = base_url.join(href).ok()?;
    if !matches!(url.scheme(), "http" | "https" | "data") {
        return None;
    }
    let value = url.to_string();
    (value.len() <= MAX_CANDIDATE_URL_BYTES).then_some(value)
}

fn candidate_rank(sizes: &str) -> u8 {
    let mut any = false;
    for size in sizes.split_ascii_whitespace() {
        if size.eq_ignore_ascii_case("32x32") {
            return 0;
        }
        any |= size.eq_ignore_ascii_case("any");
    }
    if any { 1 } else { 2 }
}

#[cfg(test)]
#[path = "_tests_/candidate_tests.rs"]
mod candidate_tests;
