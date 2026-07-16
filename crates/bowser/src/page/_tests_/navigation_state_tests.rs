use super::{
    NavigationSample, changed_document_ready, loader_correlated_redirect_ready,
    redirected_document_ready, requested_document_ready,
};

fn sample(url: &str, ready: bool, document_key: &str) -> NavigationSample {
    NavigationSample {
        url: url.to_string(),
        ready,
        document_key: document_key.to_string(),
    }
}

#[test]
fn requested_url_requires_new_document_for_same_url_navigation() {
    let current = sample("https://example.test/page", true, "doc-1");

    assert!(!requested_document_ready(
        &current,
        "https://example.test/page",
        Some("https://example.test/page"),
        Some("doc-1"),
    ));

    let reloaded = sample("https://example.test/page", true, "doc-2");
    assert!(requested_document_ready(
        &reloaded,
        "https://example.test/page",
        Some("https://example.test/page"),
        Some("doc-1"),
    ));
}

#[test]
fn about_blank_is_valid_when_requested() {
    let blank = sample("about:blank", true, "doc-2");

    assert!(requested_document_ready(
        &blank,
        "about:blank",
        Some("https://example.test/page"),
        Some("doc-1"),
    ));
    assert!(!requested_document_ready(
        &blank,
        "https://example.test/page",
        Some("https://example.test/start"),
        Some("doc-1"),
    ));
}

#[test]
fn redirected_documents_require_prior_state_change() {
    let redirected = sample("https://example.test/landing", true, "doc-2");

    assert!(!requested_document_ready(
        &redirected,
        "https://example.test/start",
        Some("https://example.test/home"),
        Some("doc-1"),
    ));
    assert!(!redirected_document_ready(
        &redirected,
        "https://example.test/start",
        None,
        None,
    ));
    assert!(redirected_document_ready(
        &redirected,
        "https://example.test/start",
        Some("https://example.test/home"),
        Some("doc-1"),
    ));
    assert!(!redirected_document_ready(
        &redirected,
        "https://example.test/landing",
        Some("https://example.test/home"),
        Some("doc-1"),
    ));
}

#[test]
fn loader_correlated_redirect_does_not_require_prior_state_change() {
    let redirected = sample("https://example.test/landing", true, "doc-2");

    assert!(loader_correlated_redirect_ready(
        &redirected,
        "https://example.test/start",
    ));
    assert!(!loader_correlated_redirect_ready(
        &redirected,
        "https://example.test/landing",
    ));
    assert!(!loader_correlated_redirect_ready(
        &sample("about:blank", true, "doc-2"),
        "https://example.test/start",
    ));
    assert!(!loader_correlated_redirect_ready(
        &sample("https://example.test/landing", false, "doc-2"),
        "https://example.test/start",
    ));
}

#[test]
fn changed_document_accepts_history_url_changes() {
    let history_change = sample("https://example.test/previous", true, "doc-1");

    assert!(changed_document_ready(
        &history_change,
        Some("https://example.test/current"),
        Some("doc-1"),
    ));
}

#[test]
fn changed_document_rejects_ready_sample_without_previous_sample() {
    let ready = sample("https://example.test/current", true, "doc-1");
    assert!(!changed_document_ready(&ready, None, None));

    let loading = sample("https://example.test/current", false, "doc-1");
    assert!(!changed_document_ready(&loading, None, None));
}

#[test]
fn changed_document_accepts_about_blank_reload_samples() {
    let blank = sample("about:blank", true, "doc-1");

    assert!(changed_document_ready(
        &blank,
        Some("about:blank"),
        Some("doc-1"),
    ));
    assert!(!changed_document_ready(
        &blank,
        Some("https://example.test/current"),
        Some("doc-1"),
    ));
}
