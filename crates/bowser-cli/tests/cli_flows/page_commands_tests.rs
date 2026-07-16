use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supports_page_list_select_new_and_close_commands() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let chrome_path = support::chrome_path();

    let get_output = support::run_bowser_with_retry(
        &[
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/long"),
        ],
        5,
    );
    assert!(get_output.status.success());
    let session_id =
        support::parse_session_id(&String::from_utf8(get_output.stderr).expect("stderr"));

    let new_output = support::run_bowser_with_retry(
        &[
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "page",
            "new",
            &server.url("/simple"),
        ],
        5,
    );
    assert!(new_output.status.success());
    let new_stdout = String::from_utf8(new_output.stdout).expect("new stdout");
    assert!(new_stdout.contains("title: Fixture Simple"));

    let list_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "page",
            "list",
        ])
        .output()
        .expect("run bowser page list");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).expect("list stdout");
    assert!(list_stdout.contains("/long"));
    assert!(list_stdout.contains("/simple"));
    let simple_page_id = support::parse_page_id(&list_stdout, "/simple");
    let long_page_id = support::parse_page_id(&list_stdout, "/long");

    let select_output = support::run_bowser_with_retry(
        &[
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "page",
            "select",
            &long_page_id,
        ],
        5,
    );
    assert!(select_output.status.success());
    let select_stdout = String::from_utf8(select_output.stdout).expect("select stdout");
    assert!(select_stdout.contains("title: Fixture Long"));

    let close_output = support::run_bowser_with_retry(
        &[
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "page",
            "close",
            &simple_page_id,
        ],
        5,
    );
    assert!(close_output.status.success());
    let close_stdout = String::from_utf8(close_output.stdout).expect("close stdout");
    assert!(close_stdout.contains("title: Fixture Long"));

    let list_after_close = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "page",
            "list",
        ])
        .output()
        .expect("run bowser page list after close");
    assert!(list_after_close.status.success());
    let list_after_close_stdout =
        String::from_utf8(list_after_close.stdout).expect("list stdout after close");
    assert!(list_after_close_stdout.contains(&long_page_id));
    assert!(!list_after_close_stdout.contains(&simple_page_id));
}
