//! Live checkpoint capture and fresh-session restore.

mod cookies;
mod export;
mod restore;

pub(super) fn http_origin(value: &str) -> Option<String> {
    let url = url::Url::parse(value).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    Some(url.origin().ascii_serialization())
}
