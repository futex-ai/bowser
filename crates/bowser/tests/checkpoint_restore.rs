mod support;

use bowser::{Browser, BrowserEngine, PageEngine};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_restores_cookies_storage_tabs_selection_and_usability() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let source_root = tempdir().expect("source session root");
    let mut source_config = support::test_config(source_root.path());
    let source = Browser::launch(source_config.clone())
        .await
        .expect("launch source browser");
    let first = source.current_page().await.expect("source first page");
    first
        .navigate(&server.url("/checkpoint-state"))
        .await
        .expect("navigate counter");
    first
        .evaluate_js(
            "document.cookie = 'visible=one; Path=/'; localStorage.setItem('checkpoint-key', 'checkpoint-value')",
        )
        .await
        .expect("seed page storage");
    let second = source
        .new_page(Some(&server.url("/simple")))
        .await
        .expect("open selected second page");
    second.capture().await.expect("capture selected page");

    let checkpoint = source.export_checkpoint().await.expect("export checkpoint");
    assert_eq!(checkpoint.pages.len(), 2);
    assert_eq!(checkpoint.selected_page, 1);
    assert!(
        checkpoint
            .cookies
            .iter()
            .any(|cookie| cookie.name == "visible")
    );
    assert!(checkpoint.cookies.iter().any(|cookie| {
        cookie.name == "checkpoint_http_only" && cookie.http_only && cookie.value == "secret"
    }));
    assert!(checkpoint.origins.iter().any(|origin| {
        origin
            .local_storage
            .iter()
            .any(|entry| entry.key == "checkpoint-key" && entry.value == "checkpoint-value")
    }));

    let source_info = source.session_info().await.expect("source session info");
    source.close().await.expect("close source browser");
    assert!(
        !source_root
            .path()
            .join(format!("{}.json", source_info.id))
            .exists()
    );
    assert!(
        std::fs::read_dir(source_root.path().join("profiles"))
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true)
    );

    let restored_root = tempdir().expect("restored session root");
    source_config.session.dir = Some(restored_root.path().to_path_buf());
    source_config.persistent_profile = true;
    let restored = Browser::restore(source_config, checkpoint)
        .await
        .expect("restore checkpoint");
    let restored_info = restored
        .session_info()
        .await
        .expect("restored session info");
    assert_ne!(restored_info.id, source_info.id);
    let pages = restored.list_pages().await.expect("restored pages");
    assert_eq!(pages.len(), 2);
    assert!(pages.iter().any(|page| {
        page.selected
            && page
                .url
                .as_deref()
                .is_some_and(|url| url.ends_with("/simple"))
    }));

    let counter_page_id = pages
        .iter()
        .find(|page| {
            page.url
                .as_deref()
                .is_some_and(|url| url.ends_with("/checkpoint-state"))
        })
        .map(|page| page.id.clone())
        .expect("restored counter page");
    let counter = restored
        .select_page(&counter_page_id)
        .await
        .expect("select restored counter");
    assert_eq!(
        evaluate_string(counter.as_ref(), "localStorage.getItem('checkpoint-key')").await,
        "checkpoint-value"
    );
    assert!(
        evaluate_string(counter.as_ref(), "document.cookie")
            .await
            .contains("visible=one")
    );
    let restored_checkpoint = restored
        .export_checkpoint()
        .await
        .expect("re-export restored state");
    assert!(restored_checkpoint.cookies.iter().any(|cookie| {
        cookie.name == "checkpoint_http_only" && cookie.http_only && cookie.value == "secret"
    }));

    let counter = restored
        .current_page()
        .await
        .expect("reacquire restored page after export");
    let capture = counter.capture().await.expect("capture restored counter");
    let button_id =
        find_button_id(&capture.content.visible, "Increment").expect("increment button");
    counter.click(button_id).await.expect("click restored page");
    assert_eq!(
        evaluate_string(
            counter.as_ref(),
            "document.getElementById('count').textContent"
        )
        .await,
        "Count: 1"
    );

    let restored_profile = restored_root
        .path()
        .join(format!("{}.json", restored_info.id));
    let metadata: bowser::SessionMetadata = serde_json::from_str(
        &std::fs::read_to_string(restored_profile).expect("read restored metadata"),
    )
    .expect("parse restored metadata");
    assert!(metadata.owns_user_data_dir);
    assert_ne!(
        metadata.user_data_dir,
        restored_root.path().join("profiles/default")
    );

    restored.close().await.expect("close restored browser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exporting_a_live_session_leaves_it_usable() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_root = tempdir().expect("session root");
    let browser = Browser::launch(support::test_config(session_root.path()))
        .await
        .expect("launch browser");
    let page = browser.current_page().await.expect("page");
    page.navigate(&server.url("/counter"))
        .await
        .expect("navigate counter");
    let first_capture = page.capture().await.expect("first capture");
    let button_id = find_button_id(&first_capture.content.visible, "Increment").expect("button");

    browser.export_checkpoint().await.expect("live export");
    let page = browser
        .current_page()
        .await
        .expect("reacquire after export");
    page.click(button_id).await.expect("click after export");
    let second_capture = page.capture().await.expect("capture after export");

    assert_eq!(second_capture.title, "Counter Fixture");
    assert_eq!(
        evaluate_string(
            page.as_ref(),
            "document.getElementById('count').textContent"
        )
        .await,
        "Count: 1"
    );
    browser.close().await.expect("close browser");
}

async fn evaluate_string(page: &dyn PageEngine, expression: &str) -> String {
    serde_json::from_str(
        &page
            .evaluate_js(expression)
            .await
            .expect("evaluate restored state"),
    )
    .expect("parse evaluated string")
}

fn find_button_id(elements: &[bowser::Element], text: &str) -> Option<u32> {
    for element in elements {
        if let bowser::Element::Button {
            id, text: label, ..
        } = element
            && label == text
        {
            return Some(*id);
        }
        if let Some(children) = element.children()
            && let Some(id) = find_button_id(children, text)
        {
            return Some(id);
        }
    }
    None
}
