use std::time::{Duration, Instant};

use super::{PageStateStringPolicy, bounded_optional_page_string};

#[tokio::test]
async fn page_state_string_reads_fall_back_after_timeout() {
    let started = Instant::now();

    let value = bounded_optional_page_string(
        Box::pin(async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            Some("late".to_string())
        }),
        Some("stored".to_string()),
        PageStateStringPolicy::AllowEmpty,
    )
    .await;

    assert_eq!(value, "stored");
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[tokio::test]
async fn title_state_string_reads_reject_empty_live_values() {
    let value = bounded_optional_page_string(
        Box::pin(async { Some(String::new()) }),
        Some("stored title".to_string()),
        PageStateStringPolicy::RequireNonEmpty,
    )
    .await;

    assert_eq!(value, "stored title");
}
