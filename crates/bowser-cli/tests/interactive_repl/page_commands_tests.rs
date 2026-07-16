#![cfg(unix)]

use std::fs;
use std::time::Duration;

use expectrl::{Expect, Regex};
use tempfile::tempdir;

use super::support;

#[test]
fn repl_supports_multi_page_commands() {
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
        &["--timeout", "90", "-i", &server.url("/long")],
    );
    repl.set_expect_timeout(Some(Duration::from_secs(90)));
    repl.expect(Regex("---")).expect("initial separator");
    repl.expect(Regex("title: Fixture Long"))
        .expect("initial title");
    support::expect_prompt(&mut repl, "prompt");
    repl.send_line(format!("new page {}", server.url("/simple")))
        .expect("new page");
    repl.expect(Regex("---")).expect("separator after new page");
    repl.expect(Regex("title: Fixture Simple"))
        .expect("new page title");
    support::expect_prompt(&mut repl, "prompt after new page");
    repl.send_line("pages").expect("pages");
    repl.expect(Regex("pg_1.*?/long"))
        .expect("long page listed");
    repl.expect(Regex("pg_2.*?/simple"))
        .expect("simple page listed");
    support::expect_prompt(&mut repl, "prompt after pages");
    repl.send_line("page pg_1").expect("switch to long");
    repl.expect(Regex("---"))
        .expect("separator after page switch");
    repl.expect(Regex("title: Fixture Long"))
        .expect("long title after switch");
    support::expect_prompt(&mut repl, "prompt after page switch");
    repl.send_line("close page pg_2").expect("close simple");
    repl.expect(Regex("---")).expect("separator after close");
    repl.expect(Regex("title: Fixture Long"))
        .expect("long title after close");
    support::expect_prompt(&mut repl, "prompt after close");
    repl.send_line("pages").expect("pages after close");
    repl.expect(Regex("pg_1.*?/long"))
        .expect("remaining long page");
    support::expect_prompt(&mut repl, "prompt after second pages");
    repl.send_line("quit").expect("quit");
    support::expect_quit_and_close(repl, session_dir.path());

    let _ = fs::remove_dir_all(session_dir.path());
}
