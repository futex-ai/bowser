use crate::url::normalize_navigation_target;

#[test]
fn keeps_explicit_urls_and_relative_targets() {
    assert_eq!(
        normalize_navigation_target("https://example.com/path"),
        "https://example.com/path"
    );
    assert_eq!(normalize_navigation_target("about:blank"), "about:blank");
    assert_eq!(
        normalize_navigation_target("/settings/profile"),
        "/settings/profile"
    );
    assert_eq!(normalize_navigation_target("../login"), "../login");
}

#[test]
fn assumes_https_for_public_hostnames() {
    assert_eq!(
        normalize_navigation_target("slack.com"),
        "https://slack.com"
    );
    assert_eq!(
        normalize_navigation_target("slack.com/signin"),
        "https://slack.com/signin"
    );
    assert_eq!(
        normalize_navigation_target("example.com:8443/app"),
        "https://example.com:8443/app"
    );
}

#[test]
fn assumes_http_for_loopback_hosts() {
    assert_eq!(
        normalize_navigation_target("localhost:3000/dashboard"),
        "http://localhost:3000/dashboard"
    );
    assert_eq!(
        normalize_navigation_target("127.0.0.1:8080/api"),
        "http://127.0.0.1:8080/api"
    );
    assert_eq!(
        normalize_navigation_target("[::1]:9222/json/version"),
        "http://[::1]:9222/json/version"
    );
}
