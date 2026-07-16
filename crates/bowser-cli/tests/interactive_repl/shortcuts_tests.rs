#![cfg(unix)]

use std::fs;

use expectrl::{Expect, Regex};
use tempfile::tempdir;

use super::support;

#[test]
fn repl_supports_keypress_shortcuts() {
    let _guard = support::browser_test_guard();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let server = runtime.block_on(support::spawn_server());
    let session_dir = tempdir().expect("session dir");
    let chrome_path = support::chrome_path();
    let binary = support::binary_path();

    let mut repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["-i", &server.url("/keyboard-shortcuts")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Keyboard Shortcut Fixture"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("click input#2").expect("focus textarea");
    repl.expect(Regex("---")).expect("separator after focus");
    repl.expect(Regex("focused: true"))
        .expect("focused textarea");
    support::expect_prompt(&mut repl, "prompt after focus");
    repl.send_line("keypress cmd enter")
        .expect("keypress combo");
    repl.expect(Regex("---")).expect("separator after keypress");
    repl.expect(Regex("Meta\\+Enter fired"))
        .expect("shortcut status");
    support::expect_prompt(&mut repl, "prompt after keypress");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}
