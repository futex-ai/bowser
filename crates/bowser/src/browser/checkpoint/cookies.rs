//! Portable checkpoint cookie conversions.

use chromiumoxide::cdp::browser_protocol::network::{Cookie, CookieParam, TimeSinceEpoch};

use crate::checkpoint::CheckpointCookie;

impl From<Cookie> for CheckpointCookie {
    fn from(cookie: Cookie) -> Self {
        Self {
            name: cookie.name,
            value: cookie.value,
            domain: cookie.domain,
            path: cookie.path,
            expires: (!cookie.session && cookie.expires >= 0.0).then_some(cookie.expires),
            secure: cookie.secure,
            http_only: cookie.http_only,
            same_site: cookie.same_site,
            priority: cookie.priority,
            source_scheme: cookie.source_scheme,
            source_port: cookie.source_port,
            partition_key: cookie.partition_key,
        }
    }
}

impl From<CheckpointCookie> for CookieParam {
    fn from(cookie: CheckpointCookie) -> Self {
        Self {
            name: cookie.name,
            value: cookie.value,
            url: None,
            domain: Some(cookie.domain),
            path: Some(cookie.path),
            secure: Some(cookie.secure),
            http_only: Some(cookie.http_only),
            same_site: cookie.same_site,
            expires: cookie.expires.map(TimeSinceEpoch::new),
            priority: None,
            same_party: None,
            source_scheme: None,
            source_port: None,
            partition_key: cookie.partition_key,
        }
    }
}
