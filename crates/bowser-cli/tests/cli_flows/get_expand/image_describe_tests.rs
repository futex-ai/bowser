use tempfile::tempdir;

use super::super::support;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn image_get_meta_and_describe_use_filename_and_ai_capability() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let chrome_path = support::chrome_path();

    let get_without_key = support::bowser_command()
        .env("BOWSER_AI_PROVIDER", "anthropic")
        .env("BOWSER_AI_MODEL", "test-model")
        .env("BOWSER_AI_API_KEY_ENV", "BOWSER_TEST_MISSING_KEY")
        .env("BOWSER_AI_ENDPOINT", server.url("/anthropic"))
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/image-page"),
        ])
        .output()
        .expect("run bowser get without ai key");
    assert!(get_without_key.status.success());

    let get_without_key_stdout = String::from_utf8(get_without_key.stdout).expect("stdout");
    let get_without_key_stderr = String::from_utf8(get_without_key.stderr).expect("stderr");
    let no_key_session_id = support::parse_session_id(&get_without_key_stderr);
    let no_key_image_id = parse_image_id(&get_without_key_stdout);
    assert!(get_without_key_stdout.contains("Checkerboard (checkerboard.png)"));
    assert!(!get_without_key_stdout.contains("describable: true"));

    let meta_without_key = support::bowser_command()
        .env("BOWSER_AI_PROVIDER", "anthropic")
        .env("BOWSER_AI_MODEL", "test-model")
        .env("BOWSER_AI_API_KEY_ENV", "BOWSER_TEST_MISSING_KEY")
        .env("BOWSER_AI_ENDPOINT", server.url("/anthropic"))
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &no_key_session_id,
            "meta",
            &no_key_image_id.to_string(),
        ])
        .output()
        .expect("run bowser meta without ai key");
    assert!(meta_without_key.status.success());
    let meta_without_key_stdout = String::from_utf8(meta_without_key.stdout).expect("meta stdout");
    assert!(!meta_without_key_stdout.contains("describable: true"));
    assert!(!meta_without_key_stdout.contains("description: anthropic mock"));

    let get_with_key = support::bowser_command()
        .env("BOWSER_AI_PROVIDER", "anthropic")
        .env("BOWSER_AI_MODEL", "test-model")
        .env("BOWSER_AI_API_KEY_ENV", "HOME")
        .env("BOWSER_AI_ENDPOINT", server.url("/anthropic"))
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "get",
            &server.url("/image-page"),
        ])
        .output()
        .expect("run bowser get with ai key");
    assert!(get_with_key.status.success());
    let get_with_key_stdout = String::from_utf8(get_with_key.stdout).expect("stdout");
    let get_with_key_stderr = String::from_utf8(get_with_key.stderr).expect("stderr");
    let ai_session_id = support::parse_session_id(&get_with_key_stderr);
    let ai_image_id = parse_image_id(&get_with_key_stdout);
    assert!(get_with_key_stdout.contains("Checkerboard (checkerboard.png)"));
    assert!(get_with_key_stdout.contains("describable: true"));

    let meta_before_describe = support::bowser_command()
        .env("BOWSER_AI_PROVIDER", "anthropic")
        .env("BOWSER_AI_MODEL", "test-model")
        .env("BOWSER_AI_API_KEY_ENV", "HOME")
        .env("BOWSER_AI_ENDPOINT", server.url("/anthropic"))
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &ai_session_id,
            "meta",
            &ai_image_id.to_string(),
        ])
        .output()
        .expect("run bowser meta before describe");
    assert!(meta_before_describe.status.success());
    let meta_before_describe_stdout =
        String::from_utf8(meta_before_describe.stdout).expect("meta stdout");
    assert!(meta_before_describe_stdout.contains("describable: true"));
    assert!(!meta_before_describe_stdout.contains("description: anthropic mock"));

    let describe_output = support::bowser_command()
        .env("BOWSER_AI_PROVIDER", "anthropic")
        .env("BOWSER_AI_MODEL", "test-model")
        .env("BOWSER_AI_API_KEY_ENV", "HOME")
        .env("BOWSER_AI_ENDPOINT", server.url("/anthropic"))
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &ai_session_id,
            "describe",
            &ai_image_id.to_string(),
        ])
        .output()
        .expect("run bowser describe");
    assert!(describe_output.status.success());
    let describe_stdout = String::from_utf8(describe_output.stdout).expect("describe stdout");
    assert!(describe_stdout.contains("alt: Checkerboard"));
    assert!(describe_stdout.contains("filename: checkerboard.png"));
    assert!(describe_stdout.contains("description: anthropic mock"));

    let meta_after_describe = support::bowser_command()
        .env("BOWSER_AI_PROVIDER", "anthropic")
        .env("BOWSER_AI_MODEL", "test-model")
        .env("BOWSER_AI_API_KEY_ENV", "HOME")
        .env("BOWSER_AI_ENDPOINT", server.url("/anthropic"))
        .args([
            "--chrome-path",
            chrome_path.to_str().expect("chrome path"),
            "--session-dir",
            session_dir.path().to_str().expect("session dir"),
            "--session",
            &ai_session_id,
            "meta",
            &ai_image_id.to_string(),
        ])
        .output()
        .expect("run bowser meta after describe");
    assert!(meta_after_describe.status.success());
    let meta_after_describe_stdout =
        String::from_utf8(meta_after_describe.stdout).expect("meta stdout");
    assert!(meta_after_describe_stdout.contains("description: anthropic mock"));
}

fn parse_image_id(output: &str) -> u32 {
    output
        .lines()
        .find_map(|line| {
            let (_, suffix) = line.split_once("image#")?;
            suffix.split_once(':')?.0.parse().ok()
        })
        .expect("image id")
}
