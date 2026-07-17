//! Bounded live page state reads.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

const PAGE_STATE_READ_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
enum PageStateStringPolicy {
    AllowEmpty,
    RequireNonEmpty,
}

impl PageStateStringPolicy {
    fn accepts(self, value: &str) -> bool {
        match self {
            Self::AllowEmpty => true,
            Self::RequireNonEmpty => !value.is_empty(),
        }
    }
}

async fn bounded_optional_page_string(
    future: Pin<Box<dyn Future<Output = Option<String>> + Send + '_>>,
    fallback: Option<String>,
    policy: PageStateStringPolicy,
) -> String {
    match tokio::time::timeout(PAGE_STATE_READ_TIMEOUT, future).await {
        Ok(Some(value)) if policy.accepts(&value) => value,
        Ok(Some(_)) | Ok(None) | Err(_) => fallback.unwrap_or_default(),
    }
}

pub(super) async fn live_page_url(page: &chromiumoxide::Page, fallback: Option<String>) -> String {
    bounded_optional_page_string(
        Box::pin(async { page.url().await.ok().flatten() }),
        fallback,
        PageStateStringPolicy::AllowEmpty,
    )
    .await
}

pub(super) async fn live_page_title(
    page: &chromiumoxide::Page,
    fallback: Option<String>,
) -> String {
    bounded_optional_page_string(
        Box::pin(async { page.get_title().await.ok().flatten() }),
        fallback,
        PageStateStringPolicy::RequireNonEmpty,
    )
    .await
}

#[cfg(test)]
#[path = "_tests_/page_state_tests.rs"]
mod page_state_tests;
