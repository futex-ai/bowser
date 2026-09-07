//! Inventory-wide favicon concurrency and time bounds.

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::time::Instant;

use crate::model::BrowserFavicon;

pub(crate) type FaviconFuture =
    Pin<Box<dyn Future<Output = Option<BrowserFavicon>> + Send + 'static>>;

#[cfg_attr(test, unimock::unimock(api = FaviconSchedulerMock))]
#[async_trait]
pub(crate) trait FaviconScheduler: Send + Sync {
    async fn collect(
        &self,
        futures: Vec<FaviconFuture>,
        deadline: Instant,
        concurrency: usize,
    ) -> Vec<Option<BrowserFavicon>>;
}

type IndexedFaviconFuture = Pin<Box<dyn Future<Output = (usize, Option<BrowserFavicon>)> + Send>>;

pub(crate) struct DeadlineFaviconScheduler;

#[async_trait]
impl FaviconScheduler for DeadlineFaviconScheduler {
    async fn collect(
        &self,
        futures: Vec<FaviconFuture>,
        deadline: Instant,
        concurrency: usize,
    ) -> Vec<Option<BrowserFavicon>> {
        collect_until(futures, deadline, concurrency).await
    }
}

async fn collect_until(
    futures: Vec<FaviconFuture>,
    deadline: Instant,
    concurrency: usize,
) -> Vec<Option<BrowserFavicon>> {
    let count = futures.len();
    let mut results = (0..count).map(|_| None).collect::<Vec<_>>();
    let mut pending = futures.into_iter().enumerate();
    let mut active: FuturesUnordered<IndexedFaviconFuture> = FuturesUnordered::new();
    let concurrency = concurrency.max(1);
    for _ in 0..concurrency {
        if let Some((index, future)) = pending.next() {
            active.push(Box::pin(async move { (index, future.await) }));
        }
    }
    loop {
        if active.is_empty() {
            break;
        }
        tokio::select! {
            biased;
            _ = tokio::time::sleep_until(deadline) => break,
            completed = active.next() => {
                let Some((index, favicon)) = completed else {
                    break;
                };
                results[index] = favicon;
                if let Some((next_index, future)) = pending.next() {
                    active.push(Box::pin(async move { (next_index, future.await) }));
                }
            }
        }
    }
    results
}

#[cfg(test)]
#[path = "_tests_/scheduler_tests.rs"]
mod scheduler_tests;
