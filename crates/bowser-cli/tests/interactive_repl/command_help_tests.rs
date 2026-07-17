#![cfg(unix)]

use std::fs;

use super::support;
use expectrl::{Expect, Regex};

#[test]
fn invalid_command_keeps_repl_running() {
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
        &["-i", &server.url("/simple")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("clcik 2").expect("invalid command");
    repl.expect(Regex("invalid command: clcik 2"))
        .expect("invalid command error");
    support::expect_prompt(&mut repl, "prompt after invalid command");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}

#[test]
fn help_prints_command_descriptions() {
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
        &["-i", &server.url("/simple")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("help").expect("help");
    repl.expect(Regex("Available commands:"))
        .expect("help heading");
    repl.expect(Regex("goto <URL>")).expect("goto usage");
    repl.expect(Regex("Navigate to a new URL"))
        .expect("goto description");
    repl.expect(Regex("new page \\[URL\\]"))
        .expect("new page usage");
    repl.expect(Regex("click <ID>")).expect("click usage");
    repl.expect(Regex("Activate a clickable element"))
        .expect("click description");
    repl.expect(Regex("keypress <KEY\\.\\.\\.>"))
        .expect("keypress usage");
    repl.expect(Regex("space-separated keys together"))
        .expect("keypress description");
    repl.expect(Regex("describe <ID>")).expect("describe usage");
    repl.expect(Regex("AI description for an image"))
        .expect("describe description");
    repl.expect(Regex("html")).expect("html usage");
    repl.expect(Regex("full rendered HTML"))
        .expect("html description");
    repl.expect(Regex("input#12")).expect("id argument note");
    support::expect_prompt(&mut repl, "prompt after help");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}

#[test]
fn html_command_prints_the_rendered_document() {
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
        &["-i", &server.url("/dynamic")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Dynamic Fixture"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("html").expect("html");
    repl.expect(Regex("<html lang=\"en\">")).expect("html root");
    repl.expect(Regex("<p id=\"loaded\">Loaded later</p>"))
        .expect("rendered dynamic content");
    support::expect_prompt(&mut repl, "prompt after html");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}
