use tempfile::tempdir;

use super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn download_command_saves_browser_native_file() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let downloads_dir = tempdir().expect("downloads dir");
    let chrome_path = support::chrome_path();
    let destination = downloads_dir.path().join("report.csv");

    let output = support::bowser_command()
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "download",
            &server.url("/download/redirect"),
            destination.to_str().expect("destination"),
        ])
        .output()
        .expect("run bowser download");

    assert!(output.status.success());
    assert_eq!(
        tokio::fs::read_to_string(&destination)
            .await
            .expect("download body"),
        "name,value\nalpha,1\n"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("Downloaded:"));
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(stderr.contains("Session: "));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn download_command_saves_current_page_link_target() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let downloads_dir = tempdir().expect("downloads dir");
    let chrome_path = support::chrome_path();

    let get_output = support::bowser_command()
        .args([
            "--no-ai",
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/download/cookie-page"),
        ])
        .output()
        .expect("run bowser get");
    assert!(get_output.status.success());
    let stdout = String::from_utf8(get_output.stdout).expect("stdout");
    let stderr = String::from_utf8(get_output.stderr).expect("stderr");
    let session_id = support::parse_session_id(&stderr);
    let link_id = parse_link_id(&stdout);
    let destination = downloads_dir.path().join("cookie.csv");

    let output = support::bowser_command()
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &session_id,
            "download",
            "--link",
            &link_id,
            "--output",
            destination.to_str().expect("destination"),
        ])
        .output()
        .expect("run bowser download link");

    assert!(output.status.success());
    assert_eq!(
        tokio::fs::read_to_string(&destination)
            .await
            .expect("download body"),
        "name,value\ncookie,1\n"
    );
}

fn parse_link_id(output: &str) -> String {
    output
        .lines()
        .find_map(|line| {
            let line = line.trim_start();
            let line = line.strip_prefix("- ").unwrap_or(line);
            if !line.contains("download") {
                return None;
            }
            line.strip_prefix("link#")
                .and_then(|rest| rest.split(':').next())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| panic!("download link id in output:\n{output}"))
}
