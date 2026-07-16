use tempfile::tempdir;

use super::super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supports_get_expand_meta_and_all_mode_commands() {
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

    let stdout = String::from_utf8(get_output.stdout).expect("stdout");
    let stderr = String::from_utf8(get_output.stderr).expect("stderr");
    let session_id = support::parse_session_id(&stderr);

    assert!(stdout.contains("title: Fixture Long"));
    assert!(stdout.contains("visible:"));
    assert!(stdout.contains("obscured: []"));
    assert!(stdout.contains("ul#1:"));
    assert!(!stdout.contains("focused: false"));
    assert!(stdout.contains("shown: 20"));
    assert!(stdout.contains("total: 22"));

    let expand_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "expand",
            "1",
        ])
        .output()
        .expect("run bowser expand");
    assert!(expand_output.status.success());
    let expand_stdout = String::from_utf8(expand_output.stdout).expect("expand stdout");
    assert!(expand_stdout.contains("Item 21"));
    assert!(expand_stdout.contains("Item 22"));

    let meta_output = support::bowser_command()
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "meta",
            "2",
        ])
        .output()
        .expect("run bowser meta");
    assert!(meta_output.status.success());
    let meta_stdout = String::from_utf8(meta_output.stdout).expect("meta stdout");
    assert!(meta_stdout.contains("href:"));
    assert!(meta_stdout.contains("/item/1"));
    assert!(meta_stdout.contains("focused: false"));
    assert!(meta_stdout.contains("visible: true"));
    assert!(meta_stdout.contains("clickable: true"));
    assert!(meta_stdout.contains("bounds:"));
    assert!(meta_stdout.contains("top_left:"));
    assert!(meta_stdout.contains("bottom_right:"));

    let list_meta_output = support::bowser_command()
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "meta",
            "1",
        ])
        .output()
        .expect("run bowser meta on list");
    assert!(list_meta_output.status.success());
    let list_meta_stdout = String::from_utf8(list_meta_output.stdout).expect("list meta stdout");
    assert!(list_meta_stdout.contains("kind: list"));
    assert!(list_meta_stdout.contains("items: 22"));
    assert!(list_meta_stdout.contains("focused: false"));
    assert!(list_meta_stdout.contains("clickable: true"));
    assert!(list_meta_stdout.contains("top_left:"));
    assert!(list_meta_stdout.contains("bottom_right:"));

    let all_simple_output = support::run_bowser_with_retry(
        &[
            "--no-ai",
            "--all",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/simple"),
        ],
        5,
    );
    assert!(all_simple_output.status.success());
    let all_simple_stdout = String::from_utf8(all_simple_output.stdout).expect("all simple stdout");
    assert!(all_simple_stdout.contains("visible:"));
    assert!(all_simple_stdout.contains("obscured:"));
    assert!(all_simple_stdout.contains("Secret link"));
    assert!(all_simple_stdout.contains("Hidden action"));

    let all_long_output = support::run_bowser_with_retry(
        &[
            "--no-ai",
            "--all",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/long"),
        ],
        5,
    );
    assert!(all_long_output.status.success());
    let all_long_stdout = String::from_utf8(all_long_output.stdout).expect("all long stdout");
    assert!(all_long_stdout.contains("Item 21"));
    assert!(all_long_stdout.contains("Item 22"));
    assert!(!all_long_stdout.contains("shown: 20"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_accepts_bare_loopback_hosts() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let chrome_path = support::chrome_path();
    let bare_simple_url = server
        .url("/simple")
        .strip_prefix("http://")
        .expect("loopback server url")
        .to_string();

    let output = support::run_bowser_with_retry(
        &[
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &bare_simple_url,
        ],
        5,
    );
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("title: Fixture Simple"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_supports_full_rendered_html_output() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let chrome_path = support::chrome_path();

    let output = support::run_bowser_with_retry(
        &[
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            "--format",
            "html",
            &server.url("/dynamic"),
        ],
        5,
    );
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("<title>Dynamic Fixture</title>"));
    assert!(stdout.contains(r#"<p id="loaded">Loaded later</p>"#));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expand_session_preserves_target_backed_iframe_content() {
    let _guard = support::browser_test_guard();
    let parent_server = support::spawn_server().await;
    let child_server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let chrome_path = support::chrome_path();

    let child_url = child_server
        .url("/iframe-child")
        .replace("127.0.0.1", "localhost");
    let parent_url = format!(
        "{}/iframe-parent-external?src={child_url}",
        parent_server.url("")
    );

    let get_output = support::run_bowser_with_retry(
        &[
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &parent_url,
        ],
        5,
    );
    assert!(get_output.status.success());

    let stdout = String::from_utf8(get_output.stdout).expect("stdout");
    let stderr = String::from_utf8(get_output.stderr).expect("stderr");
    let session_id = support::parse_session_id(&stderr);

    assert!(stdout.contains("button#1: Outer"));
    assert!(stdout.contains("iframe#2:"));
    assert!(stdout.contains("article#3:"));
    assert!(stdout.contains("h2: Frame Child"));
    assert!(stdout.contains("link#4: Inside"));
    assert!(stdout.contains("link#5: After"));

    let expand_output = support::bowser_command()
        .args([
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "expand",
            "2",
        ])
        .output()
        .expect("run bowser expand");
    assert!(expand_output.status.success());

    let expand_stdout = String::from_utf8(expand_output.stdout).expect("expand stdout");
    assert!(expand_stdout.contains("h2: Frame Child"));
    assert!(expand_stdout.contains("link#4: Inside"));
}
