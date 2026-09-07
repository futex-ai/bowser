//! Favicon machine-output opt-in and persistence privacy regressions.

use base64::{Engine, engine::general_purpose::STANDARD};
use bowser::{Browser, BrowserEngine};

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn opt_in_info_is_flat_ephemeral_and_regular_info_stays_icon_free() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let mut config = support::test_config(session_dir.path());
    let browser = Browser::launch(config.clone())
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate fixture");
    let session_id = browser.session_info().await.expect("session info").id;
    browser.detach().await.expect("detach browser");
    config.session.id = Some(session_id.clone());

    let opt_in = support::bowser_command()
        .args([
            "--json-envelope",
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "session",
            "info",
            &session_id,
            "--include-favicons",
        ])
        .output()
        .expect("run favicon inventory");
    assert!(
        opt_in.status.success(),
        "{}",
        String::from_utf8_lossy(&opt_in.stdout)
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&opt_in.stdout).expect("favicon envelope");
    let pages = envelope["result"]["session"]["pages"]
        .as_array()
        .expect("flat live pages");
    let page = pages.first().expect("live page");
    assert_eq!(page["page_id"], envelope["page"]);
    assert!(
        page["url"]
            .as_str()
            .is_some_and(|url| url.ends_with("/simple"))
    );
    assert_eq!(page["title"], "Fixture Simple");
    assert_eq!(page["selected"], true);
    let png_base64 = page["favicon"]["png_base64"]
        .as_str()
        .expect("favicon base64");
    let png = STANDARD.decode(png_base64).expect("canonical favicon PNG");
    assert!(png_base64.len() <= 6 * 1024);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&png[16..24], &[0, 0, 0, 32, 0, 0, 0, 32]);

    let regular = support::bowser_command()
        .args([
            "--json-envelope",
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "session",
            "info",
            &session_id,
        ])
        .output()
        .expect("run regular inventory");
    assert!(regular.status.success());
    let regular: serde_json::Value =
        serde_json::from_slice(&regular.stdout).expect("regular envelope");
    let regular_page = &regular["result"]["session"]["pages"][0];
    assert!(regular_page.get("id").is_some());
    assert!(regular_page.get("favicon").is_none());
    let stored = std::fs::read_to_string(session_dir.path().join(format!("{session_id}.json")))
        .expect("stored session metadata");
    assert!(!stored.contains("favicon"));

    let resumed = Browser::launch(config).await.expect("resume browser");
    resumed.close().await.expect("close browser");
}
