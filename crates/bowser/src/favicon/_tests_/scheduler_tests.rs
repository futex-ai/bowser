//! Favicon inventory deadline and concurrency regressions.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use tokio::time::Instant as TokioInstant;

use super::{DeadlineFaviconScheduler, FaviconFuture, FaviconScheduler};
use crate::model::BrowserFavicon;

#[tokio::test]
async fn caps_concurrency_and_preserves_input_order() {
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let futures = (0..8)
        .map(|index| tracked_future(index, active.clone(), peak.clone()))
        .collect();

    let results = DeadlineFaviconScheduler
        .collect(futures, TokioInstant::now() + Duration::from_secs(1), 4)
        .await;

    assert_eq!(peak.load(Ordering::SeqCst), 4);
    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert_eq!(
        results
            .into_iter()
            .map(|favicon| favicon.expect("favicon").png_base64)
            .collect::<Vec<_>>(),
        (0..8).map(|index| index.to_string()).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn returns_completed_items_and_cancels_unfinished_work_at_budget() {
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let immediate: FaviconFuture = Box::pin(async {
        Some(BrowserFavicon {
            png_base64: "ready".to_string(),
        })
    });
    let slow = tracked_future_with_delay(1, active.clone(), peak, Duration::from_secs(60));
    let started = Instant::now();

    let results = DeadlineFaviconScheduler
        .collect(
            vec![immediate, slow],
            TokioInstant::now() + Duration::from_millis(30),
            4,
        )
        .await;

    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(
        results[0].as_ref().map(|icon| icon.png_base64.as_str()),
        Some("ready")
    );
    assert!(results[1].is_none());
    assert_eq!(active.load(Ordering::SeqCst), 0);
}

fn tracked_future(index: usize, active: Arc<AtomicUsize>, peak: Arc<AtomicUsize>) -> FaviconFuture {
    tracked_future_with_delay(index, active, peak, Duration::from_millis(20))
}

fn tracked_future_with_delay(
    index: usize,
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
    delay: Duration,
) -> FaviconFuture {
    Box::pin(async move {
        let _guard = ActiveGuard::new(active, peak);
        tokio::time::sleep(delay).await;
        Some(BrowserFavicon {
            png_base64: index.to_string(),
        })
    })
}

struct ActiveGuard {
    active: Arc<AtomicUsize>,
}

impl ActiveGuard {
    fn new(active: Arc<AtomicUsize>, peak: Arc<AtomicUsize>) -> Self {
        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
        peak.fetch_max(current, Ordering::SeqCst);
        Self { active }
    }
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}
