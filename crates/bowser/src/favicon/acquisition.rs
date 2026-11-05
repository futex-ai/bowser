//! Fail-soft favicon acquisition over a resource-reader boundary.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::time::Instant;

use crate::favicon::candidate::{FaviconDocument, candidate_urls, same_document_and_declarations};
use crate::favicon::image::normalize_favicon;
use crate::model::BrowserFavicon;

#[cfg_attr(test, unimock::unimock(api = [SnapshotFaviconMock, ReadFaviconMock]))]
#[async_trait]
pub(crate) trait FaviconResourceReader: Send + Sync {
    async fn snapshot(&self) -> Option<FaviconDocument>;
    async fn read(&self, address: &str) -> Option<Vec<u8>>;
}

#[cfg_attr(test, unimock::unimock(api = [AcquireFaviconMock]))]
#[async_trait]
pub(crate) trait FaviconAcquirer: Send + Sync {
    async fn acquire(
        &self,
        reader: Arc<dyn FaviconResourceReader>,
        deadline: Instant,
    ) -> Option<BrowserFavicon>;
}

pub(crate) struct DefaultFaviconAcquirer;

#[async_trait]
impl FaviconAcquirer for DefaultFaviconAcquirer {
    async fn acquire(
        &self,
        reader: Arc<dyn FaviconResourceReader>,
        deadline: Instant,
    ) -> Option<BrowserFavicon> {
        acquire(reader.as_ref(), deadline).await
    }
}

async fn acquire(
    reader: &(dyn FaviconResourceReader + Send + Sync),
    deadline: Instant,
) -> Option<BrowserFavicon> {
    if Instant::now() >= deadline {
        return None;
    }
    let initial = reader.snapshot().await?;
    for address in candidate_urls(&initial) {
        if Instant::now() >= deadline {
            return None;
        }
        let Some(source) = reader.read(&address).await else {
            continue;
        };
        let favicon = tokio::time::timeout_at(
            deadline,
            tokio::task::spawn_blocking(move || normalize_favicon(&source)),
        )
        .await
        .ok()
        .and_then(Result::ok)
        .flatten();
        let Some(favicon) = favicon else {
            continue;
        };
        if Instant::now() >= deadline {
            return None;
        }
        let current = reader.snapshot().await?;
        return (Instant::now() < deadline && same_document_and_declarations(&initial, &current))
            .then_some(favicon);
    }
    None
}

#[cfg(test)]
#[path = "_tests_/acquisition_tests.rs"]
mod acquisition_tests;
