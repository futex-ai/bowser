use bowser::{Browser, BrowserEngine, BrowserIdentitySnapshot};
use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_identity_defaults_are_consistent() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.stealth = true;
    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/automation-globals"))
        .await
        .expect("navigate automation globals");
    page.wait_for_selector("p", std::time::Duration::from_secs(5))
        .await
        .expect("wait for automation globals");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let snapshot: BrowserIdentitySnapshot = serde_json::from_str(
        &page
            .evaluate_js(bowser::browser_identity_diagnostic_script())
            .await
            .expect("browser identity"),
    )
    .expect("parse browser identity");

    assert!(!snapshot.webdriver_is_true);
    assert!(!snapshot.user_agent.contains("Headless"));
    let user_agent_data = snapshot
        .user_agent_data
        .as_ref()
        .expect("user agent client hints");
    assert!(!user_agent_data.brands.is_empty());
    assert!(!user_agent_data.platform.is_empty());
    assert!(snapshot.automation_globals.is_empty());
    assert_eq!(snapshot.notification_permission.as_deref(), Some("prompt"));
    assert_eq!(snapshot.mime_types.object_tag, "[object MimeTypeArray]");
    assert!(snapshot.viewport.screen_width > 0.0);
    assert!(snapshot.viewport.screen_height > 0.0);
    assert!(snapshot.viewport.avail_width > 0.0);
    assert!(snapshot.viewport.avail_height > 0.0);
    assert!(snapshot.viewport.inner_width > 0.0);
    assert!(snapshot.viewport.inner_height > 0.0);
    assert!(!uses_synthetic_plugin_profile(&snapshot));
    assert!(!uses_removed_webgl_profile(&snapshot));
    assert!(!uses_synthetic_empty_runtime(&snapshot));

    browser.close().await.expect("close browser");
}

fn uses_synthetic_plugin_profile(snapshot: &BrowserIdentitySnapshot) -> bool {
    let names = snapshot
        .plugins
        .iter()
        .map(|plugin| plugin.name.as_str())
        .collect::<Vec<_>>();
    names
        == [
            "Chrome PDF Viewer",
            "Chromium PDF Viewer",
            "WebKit built-in PDF",
        ]
}

fn uses_removed_webgl_profile(snapshot: &BrowserIdentitySnapshot) -> bool {
    snapshot
        .webgl
        .as_ref()
        .map(|webgl| webgl.vendor == "Intel Inc." && webgl.renderer == "Intel Iris OpenGL Engine")
        .unwrap_or(false)
}

fn uses_synthetic_empty_runtime(snapshot: &BrowserIdentitySnapshot) -> bool {
    snapshot.chrome.runtime_present
        && snapshot.chrome.runtime_type == "object"
        && snapshot.chrome.runtime_key_count == 0
}
