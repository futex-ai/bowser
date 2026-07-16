use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn supports_session_info_list_and_close_commands() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
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
