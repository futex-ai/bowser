//! Browser-backed kiosk launch and page-management coverage.

#![cfg(target_os = "linux")]

mod support;

#[path = "kiosk_mode/support.rs"]
mod kiosk_support;

use std::fs;

use bowser::{Browser, BrowserEngine, FileSessionStore, SessionStore, Viewport};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KioskMetrics {
    inner_width: u32,
    inner_height: u32,
    outer_width: u32,
    outer_height: u32,
    screen_width: u32,
    screen_height: u32,
    webdriver: bool,
}

#[test]
fn kiosk_fills_display_and_keeps_page_workflows_operational() {
    const TEST_NAME: &str = "kiosk_fills_display_and_keeps_page_workflows_operational";
    const CHILD_MARKER: &str = "BOWSER_KIOSK_TEST_CHILD";
    if !kiosk_support::is_child(CHILD_MARKER) {
        kiosk_support::run_on_managed_display(TEST_NAME, CHILD_MARKER, 1024, 768);
        return;
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("kiosk test runtime");
    runtime.block_on(kiosk_flow());
}

async fn kiosk_flow() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.headless = false;
    config.stealth = true;
    config.viewport = Viewport {
        width: 1024,
        height: 768,
    };
    config.chrome_args = vec![
        "--webrtc-ip-handling-policy=disable_non_proxied_udp".to_string(),
        "--enforce-webrtc-ip-permission-check".to_string(),
        "--kiosk".to_string(),
        "--disable-blink-features=AutomationControlled,InterestCohortAPI".to_string(),
        "--disable-gpu".to_string(),
    ];

    let browser = Browser::launch(config).await.expect("launch kiosk browser");
    let session = browser.session_info().await.expect("session info");
    let store = FileSessionStore::new(session_dir.path().to_path_buf());
    let metadata = store.load(&session.id).await.expect("session metadata");
    assert!(metadata.xvfb_display.is_none());
    assert_real_process_arguments(metadata.pid);

    let first_page = browser.current_page().await.expect("first page");
    first_page
        .navigate(&server.url("/simple"))
        .await
        .expect("navigate first page");
    let first_capture = first_page.capture().await.expect("capture first page");
    assert_eq!(first_capture.title, "Fixture Simple");
    let first_page_id = browser
        .session_info()
        .await
        .expect("selected page")
        .selected_page_id
        .expect("first page id");

    let metrics: KioskMetrics = serde_json::from_str(
        &first_page
            .evaluate_js(
                "({ innerWidth, innerHeight, outerWidth, outerHeight, screenWidth: screen.width, screenHeight: screen.height, webdriver: navigator.webdriver })",
            )
            .await
            .expect("kiosk metrics"),
    )
    .expect("parse kiosk metrics");
    assert_eq!(metrics.screen_width, 1024, "{metrics:?}");
    assert_eq!(metrics.screen_height, 768, "{metrics:?}");
    assert_eq!(metrics.inner_width, metrics.screen_width, "{metrics:?}");
    assert_eq!(metrics.inner_height, metrics.screen_height, "{metrics:?}");
    assert_eq!(metrics.outer_width, metrics.inner_width, "{metrics:?}");
    assert_eq!(metrics.outer_height, metrics.inner_height, "{metrics:?}");
    assert!(!metrics.webdriver);
    assert_png(&first_page.screenshot().await.expect("kiosk screenshot"));

    let second_page = browser
        .new_page(Some(&server.url("/long")))
        .await
        .expect("new kiosk page");
    assert_eq!(
        second_page.capture().await.expect("second capture").title,
        "Fixture Long"
    );
    let pages = browser.list_pages().await.expect("two kiosk pages");
    assert_eq!(pages.len(), 2, "unexpected kiosk pages: {pages:#?}");
    let second_page_id = pages
        .iter()
        .find(|page| {
            page.url
                .as_deref()
                .is_some_and(|url| url.ends_with("/long"))
        })
        .map(|page| page.id.clone())
        .expect("second page id");

    let selected_first = browser
        .select_page(&first_page_id)
        .await
        .expect("select first kiosk page");
    assert_eq!(
        selected_first
            .capture()
            .await
            .expect("selected capture")
            .title,
        "Fixture Simple"
    );
    let remaining = browser
        .close_page(Some(&first_page_id))
        .await
        .expect("close kiosk page");
    assert_eq!(
        remaining.capture().await.expect("remaining capture").title,
        "Fixture Long"
    );

    let popup_url = serde_json::to_string(&server.url("/next")).expect("popup URL");
    remaining
        .evaluate_js(&format!(
            "(() => {{ const link = document.createElement('a'); link.href = {popup_url}; link.target = '_blank'; link.textContent = 'Open tab'; document.body.append(link); link.focus(); return true; }})()"
        ))
        .await
        .expect("create content link");
    remaining
        .press_keys(&["Enter".to_string()])
        .await
        .expect("open content tab");
    let pages = wait_for_page_count(&browser, 2).await;
    assert_eq!(pages.iter().filter(|page| page.selected).count(), 1);
    let opened_page_id = pages
        .iter()
        .find(|page| {
            page.url
                .as_deref()
                .is_some_and(|url| url.ends_with("/next"))
        })
        .map(|page| page.id.clone())
        .expect("content-opened page");
    assert_ne!(opened_page_id, second_page_id);
    assert!(
        pages
            .iter()
            .find(|page| page.id == opened_page_id)
            .expect("selected content-opened page")
            .selected
    );
    let opened = browser
        .select_page(&opened_page_id)
        .await
        .expect("select content-opened page");
    assert_eq!(
        opened.capture().await.expect("content page capture").title,
        "Next"
    );

    browser.close().await.expect("close kiosk browser");
}

fn assert_real_process_arguments(pid: u32) {
    let bytes = fs::read(format!("/proc/{pid}/cmdline")).expect("Chrome command line");
    let args = bytes
        .split(|byte| *byte == 0)
        .filter(|arg| !arg.is_empty())
        .map(|arg| std::str::from_utf8(arg).expect("UTF-8 Chrome argument"))
        .collect::<Vec<_>>();
    for required in [
        "--webrtc-ip-handling-policy=disable_non_proxied_udp",
        "--enforce-webrtc-ip-permission-check",
        "--kiosk",
    ] {
        assert!(args.contains(&required), "missing {required}: {args:?}");
    }
    let bowser_default = args
        .iter()
        .position(|arg| *arg == "--disable-blink-features=AutomationControlled")
        .expect("Bowser stealth argument");
    let caller_override = args
        .iter()
        .position(|arg| *arg == "--disable-blink-features=AutomationControlled,InterestCohortAPI")
        .expect("caller stealth override");
    assert!(caller_override > bowser_default);
    for conflict in [
        "--start-maximized",
        "--window-size",
        "--window-position",
        "--start-fullscreen",
        "--app",
    ] {
        assert!(!args.iter().any(|arg| arg.starts_with(conflict)));
    }
}

async fn wait_for_page_count(
    browser: &dyn BrowserEngine,
    expected: usize,
) -> Vec<bowser::SessionPageSummary> {
    for _ in 0..10 {
        let pages = browser.list_pages().await.expect("live kiosk inventory");
        if pages.len() == expected {
            return pages;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    browser.list_pages().await.expect("final kiosk inventory")
}

fn assert_png(bytes: &[u8]) {
    assert!(bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
}
