use std::time::{Duration, Instant};

use bowser::{
    BrowserConfig, Element, FileSessionStore, PageCapture, PageContent, SessionMetadata,
    SessionStore,
};
use tempfile::tempdir;

use super::super::repl::ReplCommand;
use super::execution::command_timeout;
#[cfg(unix)]
use super::interrupts::REPL_SIGINT_PENDING;
use super::interrupts::{
    CommandExecution, clear_pending_sigint, run_interruptible, should_exit_on_interrupt,
};
use super::preview::capture_with_stored_fallback;
#[cfg(unix)]
use std::sync::atomic::Ordering;

#[tokio::test]
async fn page_management_capture_uses_stored_preview_when_live_capture_times_out() {
    let session_dir = tempdir().expect("session dir");
    let store = FileSessionStore::new(session_dir.path().to_path_buf());
    let mut metadata = SessionMetadata::new(
        "http://localhost:9222".to_string(),
        "ws://localhost:9222/devtools/browser/test".to_string(),
        0,
        session_dir.path().join("profile"),
    );
    let page_id = {
        let page = metadata.ensure_page_for_target("target-1");
        page.preview_capture = Some(stored_capture());
        page.id.clone()
    };
    metadata.selected_page_id = Some(page_id);
    store.save(&metadata).await.expect("save metadata");

    let mut config = BrowserConfig::default();
    config.session.dir = Some(session_dir.path().to_path_buf());
    let timeout = Duration::from_millis(20);
    let started = Instant::now();

    let capture = capture_with_stored_fallback(
        Box::pin(async move {
            tokio::time::sleep(Duration::from_secs(60)).await;
            Ok(PageCapture {
                url: "https://live.example".to_string(),
                title: "Live".to_string(),
                body_id: None,
                obscured_body_id: None,
                content: PageContent::default(),
            })
        }),
        timeout,
        &config,
        &metadata.id,
    )
    .await
    .expect("stored fallback");

    assert_eq!(capture.title, "Stored Preview");
    assert!(started.elapsed() < Duration::from_secs(1));
}

fn stored_capture() -> PageCapture {
    PageCapture {
        url: "https://stored.example".to_string(),
        title: "Stored Preview".to_string(),
        body_id: None,
        obscured_body_id: None,
        content: PageContent::visible_only(vec![Element::Text {
            text: "cached body".to_string(),
        }]),
    }
}

#[test]
fn exits_only_after_second_interrupt() {
    let mut interrupted_once = false;
    assert!(!should_exit_on_interrupt(&mut interrupted_once));
    assert!(interrupted_once);
    assert!(should_exit_on_interrupt(&mut interrupted_once));
}

#[cfg(unix)]
#[tokio::test]
async fn in_flight_commands_can_be_interrupted() {
    clear_pending_sigint();
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        REPL_SIGINT_PENDING.store(true, Ordering::SeqCst);
    });
    let outcome = run_interruptible(
        async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            1_u8
        },
        Duration::from_secs(5),
    )
    .await;
    assert!(matches!(outcome, CommandExecution::Interrupted));
    clear_pending_sigint();
}

#[tokio::test]
async fn in_flight_commands_time_out() {
    let outcome = run_interruptible(
        async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            1_u8
        },
        Duration::from_millis(20),
    )
    .await;
    assert!(matches!(outcome, CommandExecution::TimedOut));
}

#[test]
fn state_changing_commands_get_capture_grace() {
    assert_eq!(
        command_timeout(
            Duration::from_secs(1),
            &ReplCommand::Goto("https://example.com".into())
        ),
        Duration::from_secs(12)
    );
}

#[test]
fn page_management_commands_get_longer_grace() {
    assert_eq!(
        command_timeout(Duration::from_secs(1), &ReplCommand::Page("pg_1".into())),
        Duration::from_secs(17)
    );
}

#[test]
fn wait_commands_keep_their_selector_budget() {
    assert_eq!(
        command_timeout(Duration::from_secs(1), &ReplCommand::Wait("#ready".into())),
        Duration::from_secs(12)
    );
}

#[test]
fn simple_commands_keep_the_base_timeout() {
    assert_eq!(
        command_timeout(Duration::from_secs(1), &ReplCommand::Help),
        Duration::from_secs(5)
    );
}
