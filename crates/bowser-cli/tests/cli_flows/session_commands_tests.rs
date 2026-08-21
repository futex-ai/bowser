use std::time::{Duration, Instant};

use bowser::{Browser, BrowserEngine};

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supports_session_info_list_and_close_commands() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();

    let get_output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/long"),
        ])
        .output()
        .expect("run bowser get");
    assert!(get_output.status.success());
    let session_id =
        support::parse_session_id(&String::from_utf8(get_output.stderr).expect("stderr"));

    let info_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "session",
            "info",
            &session_id,
        ])
        .output()
        .expect("run bowser session info");
    assert!(info_output.status.success());
    let info_stdout = String::from_utf8(info_output.stdout).expect("info stdout");
    assert!(info_stdout.contains(&format!("id: {session_id}")));
    assert!(info_stdout.contains("title: Fixture Long"));

    let list_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "session",
            "list",
        ])
        .output()
        .expect("run bowser session list");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).expect("list stdout");
    assert!(list_stdout.contains(&session_id));

    let close_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "session",
            "close",
            &session_id,
        ])
        .output()
        .expect("run bowser session close");
    assert!(close_output.status.success());

    let list_after_close = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "session",
            "list",
        ])
        .output()
        .expect("run bowser session list after close");
    assert!(list_after_close.status.success());
    let list_after_close_stdout =
        String::from_utf8(list_after_close.stdout).expect("list stdout after close");
    assert!(!list_after_close_stdout.contains(&session_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_info_reads_live_navigation_and_content_opened_tabs() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let session_root = session_dir.path().to_str().expect("session dir");

    let browser = Browser::launch(support::test_config(session_dir.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("current page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("initial navigation");
    page.capture().await.expect("initial capture");
    let session_info = browser.session_info().await.expect("session info");
    let session_id = session_info.id;
    let selected_page_id = session_info.selected_page_id.expect("selected page id");
    let popup_url = serde_json::to_string(&server.url("/long")).expect("popup URL");
    let focused = page
        .evaluate_js(&format!(
            "(() => {{ const link = document.createElement('a'); link.href = {popup_url}; link.target = '_blank'; link.textContent = 'Open tab'; document.body.append(link); link.focus(); return document.activeElement === link; }})()"
        ))
        .await
        .expect("content-opened tab link");
    assert_eq!(focused, "true");
    page.press_keys(&["Enter".to_string()])
        .await
        .expect("open content tab");
    page.evaluate_js(
        "history.pushState({}, '', '/next'); document.title = 'Human Navigation'; true",
    )
    .await
    .expect("out-of-band navigation");
    tokio::time::sleep(std::time::Duration::from_millis(750)).await;

    let info_started = Instant::now();
    let info = support::run_bowser_with_retry(
        &[
            "--json-envelope",
            "--session-dir",
            session_root,
            "session",
            "info",
            &session_id,
        ],
        5,
    );
    assert!(
        info_started.elapsed() < Duration::from_secs(5),
        "live inventory took {:?}",
        info_started.elapsed()
    );
    browser.detach().await.expect("detach browser");
    assert!(
        info.status.success(),
        "session info failed: {}",
        String::from_utf8_lossy(&info.stdout)
    );
    let info_envelope: serde_json::Value =
        serde_json::from_slice(&info.stdout).expect("info envelope");
    let session = &info_envelope["result"]["session"];
    let pages = session["pages"].as_array().expect("live pages");
    assert_eq!(pages.len(), 2, "{pages:?}");
    assert!(pages.iter().any(|page| {
        page["id"] == selected_page_id
            && page["last_url"]
                .as_str()
                .is_some_and(|url| url.ends_with("/next"))
            && page["last_title"] == "Human Navigation"
    }));
    let content_opened_page = pages
        .iter()
        .find(|page| {
            page["last_url"]
                .as_str()
                .is_some_and(|url| url.ends_with("/long"))
                && page["last_title"] == "Fixture Long"
        })
        .unwrap_or_else(|| panic!("content-opened page missing: {pages:?}"));
    assert_ne!(content_opened_page["id"], selected_page_id);
    assert_eq!(session["selected_page_id"], content_opened_page["id"]);
    assert_eq!(info_envelope["page"], content_opened_page["id"]);
    assert_eq!(
        pages
            .iter()
            .filter(|page| page["id"] == session["selected_page_id"])
            .count(),
        1
    );
}
