#![cfg(unix)]

use std::fs;

use super::support;
use expectrl::{Expect, Regex};

#[test]
fn repl_supports_expand_meta_separator_and_resume() {
    let _guard = support::browser_test_guard();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let server = runtime.block_on(support::spawn_server());
    let session_dir = support::test_session_dir();
    let chrome_path = support::chrome_path();
    let binary = support::binary_path();

    let mut long_repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["-i", &server.url("/long")],
    );
    long_repl.expect(Regex("---")).expect("initial separator");
    long_repl
        .expect(Regex("title: Fixture Long"))
        .expect("initial title");
    support::expect_prompt(&mut long_repl, "prompt");
    long_repl.send_line("expand 1").expect("expand command");
    long_repl
        .expect(Regex("Item 21"))
        .expect("expanded item 21");
    long_repl
        .expect(Regex("Item 22"))
        .expect("expanded item 22");
    support::expect_prompt(&mut long_repl, "prompt after expand");
    long_repl.send_line("meta 1").expect("list meta command");
    long_repl
        .expect(Regex("kind: list"))
        .expect("list meta kind");
    long_repl
        .expect(Regex("items: 22"))
        .expect("list meta items");
    long_repl
        .expect(Regex("clickable: true"))
        .expect("list meta clickable");
    long_repl
        .expect(Regex("top_left:"))
        .expect("list meta top-left");
    long_repl
        .expect(Regex("bottom_right:"))
        .expect("list meta bottom-right");
    support::expect_prompt(&mut long_repl, "prompt after list meta");
    long_repl.send_line("meta 2").expect("meta command");
    long_repl.expect(Regex("href:")).expect("meta href");
    long_repl.expect(Regex("/item/1")).expect("meta item path");
    long_repl
        .expect(Regex("visible: true"))
        .expect("meta visible");
    long_repl
        .expect(Regex("top_left:"))
        .expect("link meta top-left");
    long_repl
        .expect(Regex("bottom_right:"))
        .expect("link meta bottom-right");
    support::expect_prompt(&mut long_repl, "prompt after meta");
    long_repl.send_line("quit").expect("quit");
    long_repl
        .expect(Regex("Session: bsr_"))
        .expect("session line");
    long_repl.expect(expectrl::Eof).expect("long repl eof");
    let long_session_id = support::only_session_id(session_dir.path());
    support::close_session(session_dir.path(), &long_session_id);

    let mut counter_repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["-i", &server.url("/counter")],
    );
    counter_repl
        .expect(Regex("---"))
        .expect("counter separator");
    counter_repl
        .expect(Regex("title: Counter Fixture"))
        .expect("counter title");
    support::expect_prompt(&mut counter_repl, "counter prompt");
    counter_repl.send_line("click 2").expect("click");
    counter_repl
        .expect(Regex("---"))
        .expect("separator after click");
    counter_repl
        .expect(Regex("Count: 1"))
        .expect("counter updated");
    support::expect_prompt(&mut counter_repl, "prompt after click");
    counter_repl
        .send_line("click input#3")
        .expect("click input");
    counter_repl
        .expect(Regex("---"))
        .expect("separator after input click");
    counter_repl
        .expect(Regex("focused: true"))
        .expect("focused input in yaml");
    support::expect_prompt(&mut counter_repl, "prompt after input click");
    counter_repl.send_line("quit").expect("quit counter repl");
    counter_repl
        .expect(Regex("Session: bsr_"))
        .expect("session line after quit");
    counter_repl
        .expect(expectrl::Eof)
        .expect("counter repl eof");

    let session_id = support::only_session_id(session_dir.path());
    let mut resumed_repl = support::repl_command(
        &binary,
        &chrome_path,
        session_dir.path(),
        &["--session", &session_id, "-i"],
    );
    resumed_repl
        .expect(Regex("---"))
        .expect("resumed separator");
    resumed_repl
        .expect(Regex("title: Counter Fixture"))
        .expect("resumed title");
    resumed_repl
        .expect(Regex("Count: 1"))
        .expect("resumed count");
    support::expect_prompt(&mut resumed_repl, "resumed prompt");
    resumed_repl.send_line("meta 3").expect("resumed meta");
    resumed_repl
        .expect(Regex("present: true"))
        .expect("resumed meta present");
    resumed_repl
        .expect(Regex("top_left:"))
        .expect("resumed meta top-left");
    resumed_repl
        .expect(Regex("bottom_right:"))
        .expect("resumed meta bottom-right");
    support::expect_prompt(&mut resumed_repl, "prompt after resumed meta");
    resumed_repl.send_line("quit").expect("quit resumed repl");
    resumed_repl
        .expect(Regex("Session: bsr_"))
        .expect("resumed session line");
    resumed_repl.expect(expectrl::Eof).expect("resumed eof");

    support::close_session(session_dir.path(), &session_id);
    let _ = fs::remove_dir_all(session_dir.path());
}

#[test]
fn all_mode_repl_shows_hidden_elements() {
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
        &["--all", "-i", &server.url("/simple")],
    );
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("initial title");
    repl.expect(Regex("visible:")).expect("visible section");
    repl.expect(Regex("obscured:")).expect("obscured section");
    repl.expect(Regex("Secret link"))
        .expect("hidden link visible in all mode");
    repl.expect(Regex("Hidden action"))
        .expect("hidden button visible in all mode");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}
