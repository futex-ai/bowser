//! Navigation-target normalization helpers.

use std::net::IpAddr;
use std::str::FromStr;

const PASSTHROUGH_SCHEMES: &[&str] = &[
    "about:",
    "chrome:",
    "chrome-extension:",
    "data:",
    "devtools:",
    "file:",
    "http:",
    "https:",
    "javascript:",
    "mailto:",
    "ws:",
    "wss:",
];

pub(crate) fn normalize_navigation_target(target: &str) -> String {
    let trimmed = target.trim();
    if trimmed.is_empty() || trimmed.contains("://") || has_passthrough_scheme(trimmed) {
        return trimmed.to_string();
    }
    if is_relative_reference(trimmed) {
        return trimmed.to_string();
    }
    let authority = trimmed.split(['/', '?', '#']).next().unwrap_or(trimmed);
    if is_localhost_authority(authority) || is_ip_authority(authority) {
        return format!("http://{trimmed}");
    }
    if authority.contains('.') {
        return format!("https://{trimmed}");
    }
    trimmed.to_string()
}

fn has_passthrough_scheme(target: &str) -> bool {
    PASSTHROUGH_SCHEMES
        .iter()
        .any(|scheme| target.starts_with(scheme))
}

fn is_relative_reference(target: &str) -> bool {
    target.starts_with('/')
        || target.starts_with("./")
        || target.starts_with("../")
        || target.starts_with('?')
        || target.starts_with('#')
}

fn is_localhost_authority(authority: &str) -> bool {
    host_from_authority(authority).eq_ignore_ascii_case("localhost")
}

fn is_ip_authority(authority: &str) -> bool {
    IpAddr::from_str(host_from_authority(authority)).is_ok()
}

fn host_from_authority(authority: &str) -> &str {
    let host_and_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    if let Some(rest) = host_and_port.strip_prefix('[') {
        return rest.split_once(']').map_or(host_and_port, |(host, _)| host);
    }
    host_and_port
        .rsplit_once(':')
        .filter(|(_, port)| !port.is_empty() && port.chars().all(|ch| ch.is_ascii_digit()))
        .map_or(host_and_port, |(host, _)| host)
}

#[cfg(test)]
#[path = "_tests_/url_tests.rs"]
mod url_tests;
