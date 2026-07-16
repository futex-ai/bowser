mod support;

use std::time::{Duration, Instant};

use bowser::{Browser, BrowserEngine, Element, Error, PageEngine};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_navigation_marks_cached_capture_stale() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());
    config.timeout = Duration::from_secs(15);

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    let simple = page.capture().await.expect("capture simple");
    let next_link_id = find_link_id(&simple.content.visible, "next").expect("next link id");

    let token = "cancelled-navigation";
    let tracked_url = server.url(&format!("/tracked-slow-page/{token}"));
    let mut navigation = Box::pin(page.navigate(&tracked_url));
    let started_at = Instant::now();
    while started_at.elapsed() < Duration::from_secs(5) {
        tokio::select! {
            result = &mut navigation => {
                panic!("navigation finished before it could be cancelled: {result:?}");
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                if tracked_slow_page_requested(&server, token).await {
                    break;
                }
            }
        }
    }
    assert!(tracked_slow_page_requested(&server, token).await);
    drop(navigation);

    wait_for_title(&*page, "Slow Page", Duration::from_secs(10)).await;
    match page.capture_subtree(next_link_id).await {
        Ok(Element::Link { text, .. }) if text == "next" => {
            panic!("cancelled navigation reused the stale simple-page capture")
        }
        Ok(_) | Err(Error::ElementNotFound { .. }) => {}
        Err(err) => panic!("unexpected capture error after cancelled navigation: {err}"),
    }

    browser.close().await.expect("close browser");
}

async fn tracked_slow_page_requested(server: &support::TestServer, token: &str) -> bool {
    reqwest::get(server.url(&format!("/tracked-slow-page-hit/{token}")))
        .await
        .expect("tracked slow page hit response")
        .text()
        .await
        .expect("tracked slow page hit body")
        == "true"
}

async fn wait_for_title(page: &dyn PageEngine, expected_title: &str, timeout: Duration) {
    let started_at = Instant::now();
    while started_at.elapsed() < timeout {
        if page
            .title()
            .await
            .as_deref()
            .is_ok_and(|title| title == expected_title)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("timed out waiting for title {expected_title}");
}

fn find_link_id(elements: &[Element], expected_text: &str) -> Option<u32> {
    elements.iter().find_map(|element| match element {
        Element::Link { id, text, .. } if text == expected_text => Some(*id),
        element => element
            .children()
            .and_then(|children| find_link_id(children, expected_text)),
    })
}
