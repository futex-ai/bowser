mod support;

use bowser::{Browser, BrowserEngine};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fresh_sessions_do_not_enumerate_the_chrome_startup_blank_page() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    page.capture().await.expect("capture simple");

    let pages = browser.list_pages().await.expect("page list");
    let live_pages = pages.iter().filter(|page| page.live).collect::<Vec<_>>();

    assert_eq!(live_pages.len(), 1);
    assert!(
        live_pages[0]
            .url
            .as_deref()
            .is_some_and(|url| url.ends_with("/simple"))
    );

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sessions_can_list_switch_create_and_close_pages() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    page.capture().await.expect("capture simple");

    let long_page = browser
        .new_page(Some(&server.url("/long")))
        .await
        .expect("new page");
    let long_capture = long_page.capture().await.expect("capture long page");
    assert_eq!(long_capture.title, "Fixture Long");

    let pages = browser.list_pages().await.expect("page list");
    let simple_page_id = pages
        .iter()
        .find(|page| {
            page.url
                .as_deref()
                .is_some_and(|url| url.ends_with("/simple"))
                && page.live
        })
        .map(|page| page.id.clone())
        .expect("simple page id");
    let long_page_id = pages
        .iter()
        .find(|page| {
            page.url
                .as_deref()
                .is_some_and(|url| url.ends_with("/long"))
                && page.live
        })
        .map(|page| page.id.clone())
        .expect("long page id");
    assert_ne!(simple_page_id, long_page_id);
    assert!(
        pages
            .iter()
            .find(|page| page.id == long_page_id)
            .expect("long page")
            .selected
    );

    let simple_page = browser
        .select_page(&simple_page_id)
        .await
        .expect("select simple page");
    assert_eq!(
        simple_page.title().await.expect("simple title"),
        "Fixture Simple"
    );

    let long_page = browser
        .select_page(&long_page_id)
        .await
        .expect("select long page");
    assert_eq!(long_page.title().await.expect("long title"), "Fixture Long");

    let remaining_page = browser
        .close_page(Some(&simple_page_id))
        .await
        .expect("close non-selected simple page");
    assert_eq!(
        remaining_page.title().await.expect("remaining title"),
        "Fixture Long"
    );
    let remaining_pages = browser.list_pages().await.expect("remaining pages");
    assert!(
        !remaining_pages
            .iter()
            .any(|page| page.id == simple_page_id && page.live)
    );
    assert!(!remaining_pages.iter().any(|page| {
        page.url
            .as_deref()
            .is_some_and(|url| url.ends_with("/simple"))
    }));
    assert!(
        remaining_pages
            .iter()
            .any(|page| page.id == long_page_id && page.live)
    );

    browser.close().await.expect("close browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resumed_sessions_can_select_the_saved_page() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let mut config = support::test_config(session_dir.path());

    let browser = Browser::launch(config.clone())
        .await
        .expect("launch browser");
    let session = browser.session_info().await.expect("session info");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/simple"))
        .await
        .expect("navigate simple");
    page.capture().await.expect("capture simple");
    let saved_page_id = browser
        .session_info()
        .await
        .expect("selected page info")
        .selected_page_id
        .expect("selected page id");
    browser.detach().await.expect("detach browser");

    config.session.id = Some(session.id);
    let resumed = Browser::launch(config).await.expect("resume browser");
    let resumed_page = resumed
        .select_page(&saved_page_id)
        .await
        .expect("select resumed page");
    assert_eq!(
        resumed_page.title().await.expect("resumed title"),
        "Fixture Simple"
    );

    resumed.close().await.expect("close resumed browser");
}
