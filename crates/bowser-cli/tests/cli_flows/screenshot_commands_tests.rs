use std::fs;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_command_writes_screenshot_file() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let screenshot = session_dir.path().join("page.png");

    let output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/long"),
            "--screenshot",
            screenshot.to_str().expect("screenshot path"),
        ])
        .output()
        .expect("run bowser get");
    assert!(output.status.success());
    assert!(screenshot.exists());
    assert!(
        fs::metadata(&screenshot)
            .expect("screenshot metadata")
            .len()
            > 0
    );
}
