#![cfg(unix)]

use std::fs;
use std::process::Command as ProcessCommand;

use super::support;
use expectrl::session::Session;
use expectrl::{ControlCode, Expect, Regex};

#[test]
fn long_running_command_times_out_and_reprompts() {
    let _guard = support::browser_test_guard();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let server = runtime.block_on(support::spawn_server());
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let binary = support::binary_path();

    let mut repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["--timeout", "5", "-i", &server.url("/simple")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("js new Promise(() => {})")
        .expect("long-running js");
    repl.expect(Regex("command timed out after 5s"))
        .expect("timeout error");
    support::expect_prompt(&mut repl, "prompt after timeout");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}

#[test]
fn ctrl_c_interrupts_in_flight_command_and_returns_prompt() {
    let _guard = support::browser_test_guard();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let server = runtime.block_on(support::spawn_server());
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let binary = support::binary_path();

    let mut repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["--timeout", "30", "-i", &server.url("/simple")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("js new Promise(() => {})")
        .expect("long-running js");
    std::thread::sleep(std::time::Duration::from_millis(200));
    repl.send(ControlCode::EndOfText).expect("ctrl-c");
    repl.expect(Regex("command interrupted"))
        .expect("interrupt error");
    support::expect_prompt(&mut repl, "prompt after interrupt");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}

#[test]
fn goto_allows_post_navigation_capture_to_finish() {
    let _guard = support::browser_test_guard();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let server = runtime.block_on(support::spawn_server());
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let binary = support::binary_path();

    let mut repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["--timeout", "15", "-i", &server.url("/simple")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line(format!("goto {}", server.url("/slow-page")))
        .expect("goto slow page");
    repl.expect(Regex("---")).expect("slow-page separator");
    repl.expect(Regex("title: Slow Page"))
        .expect("slow-page title");
    support::expect_prompt(&mut repl, "prompt after goto");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}

#[test]
fn goto_normalizes_bare_loopback_hosts() {
    let _guard = support::browser_test_guard();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let server = runtime.block_on(support::spawn_server());
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let binary = support::binary_path();
    let bare_simple_url = server
        .url("/simple")
        .strip_prefix("http://")
        .expect("loopback server url")
        .to_string();

    let mut repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["-i", &server.url("/help")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Help"))
        .expect("initial help title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line(format!("goto {bare_simple_url}"))
        .expect("goto bare loopback host");
    repl.expect(Regex("---")).expect("simple separator");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("simple title");
    support::expect_prompt(&mut repl, "prompt after goto");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}

#[test]
fn click_recapture_does_not_wait_for_ai_image_descriptions() {
    let _guard = support::browser_test_guard();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let server = runtime.block_on(support::spawn_server());
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let binary = support::binary_path();

    let mut command = ProcessCommand::new(binary);
    command
        .env("BOWSER_HEADLESS", "true")
        .env("BOWSER_CHROME_ARGS", support::CONSTRAINED_SHM_CHROME_ARGS)
        .env(
            "BOWSER_INTERNAL_STEALTH_FEATURES",
            support::HEADLESS_STEALTH_FEATURES,
        )
        .env("BOWSER_AI_PROVIDER", "anthropic")
        .env("BOWSER_AI_MODEL", "test-model")
        .env("BOWSER_AI_API_KEY_ENV", "HOME")
        .env("BOWSER_AI_ENDPOINT", server.url("/anthropic-slow"))
        .arg("--timeout")
        .arg("5")
        .arg("--chrome-path")
        .arg(&chrome_path)
        .arg("--session-dir")
        .arg(session_dir.path())
        .arg("-i")
        .arg(server.url("/click-reveals-images"));
    let mut repl = Session::spawn(command).expect("spawn repl command");
    repl.set_expect_timeout(Some(support::REPL_EXPECT_TIMEOUT));

    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Click Reveals Images"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("click 1").expect("click button");
    repl.expect(Regex("---")).expect("click separator");
    repl.expect(Regex("text: clicked"))
        .expect("clicked status in capture");
    repl.expect(Regex("describable: true"))
        .expect("images present after click");
    support::expect_prompt(&mut repl, "prompt after click");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}
