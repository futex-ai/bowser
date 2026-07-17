#![cfg(unix)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::time::Duration;

use super::support;
use expectrl::{Eof, Expect, Regex, Session, session::OsSession};

#[test]
#[ignore = "live Google smoke test; run with BOWSER_GOOGLE_SMOKE=1"]
fn google_search_flow_avoids_captcha() {
    if env::var("BOWSER_GOOGLE_SMOKE").ok().as_deref() != Some("1") {
        eprintln!("set BOWSER_GOOGLE_SMOKE=1 to run the live Google smoke flow");
        return;
    }

    let _guard = support::browser_test_guard();
    let session_dir = support::test_session_dir();
    let trace_path = session_dir.path().join("cdp-trace.jsonl");
    let mut repl = google_repl(session_dir.path(), &trace_path);

    let accept_id = expect_element_id(
        &mut repl,
        r"button#([0-9]+)[^\r\n]*(Accept all|I agree|Accept)",
        "cookie accept button",
    );
    support::expect_prompt(&mut repl, "initial prompt");

    repl.send_line(format!("click {accept_id}"))
        .expect("accept cookies");
    let search_input_id = expect_element_id(&mut repl, r"input#([0-9]+):", "search input");
    support::expect_prompt(&mut repl, "prompt after accept");

    repl.send_line(format!("type {search_input_id} hello"))
        .expect("type query");
    let search_button_id = expect_element_id(
        &mut repl,
        r"button#([0-9]+)[^\r\n]*Google Search",
        "google search button",
    );
    support::expect_prompt(&mut repl, "prompt after typing");

    repl.send_line(format!("click {search_button_id}"))
        .expect("submit search");
    let after_search = expect_prompt_output(&mut repl, "prompt after search");
    eprintln!("google smoke capture after search:\n{after_search}");
    assert_no_google_sorry_markers(&after_search);

    std::thread::sleep(Duration::from_secs(3));
    let current_url = current_url(&mut repl);
    eprintln!("google smoke url after wait:\n{current_url}");
    assert_google_search_url(&current_url);

    if runtime_assertions_enabled() {
        log_page_state(&mut repl, "after search");
        assert_js(
            &mut repl,
            "!window.location.href.includes('/sorry/')",
            "true",
        );
        assert_js(
            &mut repl,
            "!document.body.innerText.includes('Our systems have detected unusual traffic')",
            "true",
        );
        assert_js(
            &mut repl,
            "!document.body.innerText.includes('reCAPTCHA')",
            "true",
        );
        assert_js(
            &mut repl,
            "window.location.href.includes('/search') && new URL(window.location.href).searchParams.get('q') === 'hello'",
            "true",
        );
    }
    assert_google_trace(&trace_path);

    repl.send_line("quit").expect("quit");
    repl.expect(Regex("Session: bsr_")).expect("session line");
    repl.expect(Eof).expect("repl eof");
    let session_id = support::only_session_id(session_dir.path());
    support::close_session(session_dir.path(), &session_id);
}

fn google_repl(session_dir: &Path, trace_path: &Path) -> OsSession {
    let mut command = ProcessCommand::new(support::binary_path());
    command
        .env("BOWSER_CDP_TRACE_PATH", trace_path)
        .arg("--no-ai")
        .arg("--headed");
    command
        .arg("--chrome-path")
        .arg(support::chrome_path())
        .arg("--session-dir")
        .arg(session_dir)
        .arg("--timeout")
        .arg("120")
        .arg("-i")
        .arg("https://www.google.com");
    if env::var("BOWSER_GOOGLE_SMOKE_PERSISTENT").ok().as_deref() == Some("1") {
        command.arg("--persistent-profile");
    }
    let mut session = Session::spawn(command).expect("spawn google repl");
    session.set_expect_timeout(Some(Duration::from_secs(120)));
    session
}

fn expect_element_id(session: &mut OsSession, pattern: &str, label: &str) -> u32 {
    let captures = session.expect(Regex(pattern)).expect(label);
    let id_index = if captures.get(1).is_some_and(|value| {
        std::str::from_utf8(value)
            .map(|text| text.chars().all(|ch| ch.is_ascii_digit()))
            .unwrap_or(false)
    }) {
        1
    } else {
        2
    };
    std::str::from_utf8(captures.get(id_index).expect("element id"))
        .expect("utf8 element id")
        .parse()
        .expect("numeric element id")
}

fn runtime_assertions_enabled() -> bool {
    env::var("BOWSER_GOOGLE_SMOKE_RUNTIME_ASSERTIONS")
        .ok()
        .as_deref()
        == Some("1")
}

fn current_url(session: &mut OsSession) -> String {
    let output = run_repl_command(session, "url", "prompt after url");
    output
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("https://") || line.starts_with("http://"))
        .expect("url command output")
        .to_string()
}

fn assert_google_search_url(url: &str) {
    assert_no_google_sorry_markers(url);
    assert!(
        url.contains("://www.google.") && url.contains("/search"),
        "expected Google search URL, got:\n{url}"
    );
    assert!(
        url.contains("q=hello"),
        "expected Google search query in URL, got:\n{url}"
    );
}

fn assert_no_google_sorry_markers(output: &str) {
    let normalized = output.to_ascii_lowercase();
    assert!(
        !normalized.contains("/sorry/"),
        "Google sorry URL detected in output:\n{output}"
    );
    assert!(
        !normalized.contains("our systems have detected unusual traffic"),
        "Google unusual-traffic text detected in output:\n{output}"
    );
    assert!(
        !normalized.contains("recaptcha"),
        "Google captcha text detected in output:\n{output}"
    );
}

fn assert_js(session: &mut OsSession, expression: &str, expected: &str) {
    let output = run_js(session, expression);
    let actual = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("js "))
        .find(|line| *line == "true" || *line == "false")
        .expect("js assertion output");
    assert_eq!(
        actual, expected,
        "js assertion failed for `{expression}` with output:\n{output}"
    );
}

fn log_page_state(session: &mut OsSession, label: &str) {
    let state = run_js(
        session,
        r#"JSON.stringify({href: window.location.href, title: document.title, text: document.body.innerText.slice(0, 500)})"#,
    );
    eprintln!("google smoke page state {label}:\n{state}");
}

fn run_js(session: &mut OsSession, expression: &str) -> String {
    run_repl_command(
        session,
        &format!("js {expression}"),
        "prompt after js expression",
    )
}

fn run_repl_command(session: &mut OsSession, command: &str, message: &str) -> String {
    session.send_line(command).expect("send repl command");
    expect_prompt_output(session, message)
}

fn expect_prompt_output(session: &mut OsSession, message: &str) -> String {
    let captures = session
        .expect(Regex(support::PROMPT_PATTERN))
        .expect(message);
    String::from_utf8_lossy(captures.before())
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn assert_google_trace(trace_path: &Path) {
    let trace = fs::read_to_string(trace_path).expect("read cdp trace");
    assert!(
        !trace.contains("\"method\":\"Runtime.enable\""),
        "Bowser-owned Google smoke path must not emit Runtime.enable"
    );
    let runtime_disable = trace.find("\"method\":\"Runtime.disable\"");
    let first_navigation = trace
        .find("\"method\":\"Page.locationAssign\"")
        .or_else(|| trace.find("\"method\":\"Page.navigate\""));
    if runtime_disable_expected() {
        assert!(
            runtime_disable
                .zip(first_navigation)
                .is_some_and(|(disable, navigate)| disable < navigate),
            "Runtime.disable should be recorded before navigation in the Google smoke path"
        );
    } else {
        assert!(
            runtime_disable.is_none(),
            "Runtime.disable should not be recorded when the runtime-disable feature is off"
        );
    }
}

fn runtime_disable_expected() -> bool {
    if env::var("BOWSER_GOOGLE_SMOKE_RUNTIME_EVENTS")
        .ok()
        .as_deref()
        == Some("1")
    {
        return false;
    }
    let runtime_env = env::var("BOWSER_DISABLE_RUNTIME_EVENTS").unwrap_or_default();
    if matches!(runtime_env.as_str(), "true" | "on" | "enabled") {
        return true;
    }
    let overrides = env::var("BOWSER_INTERNAL_STEALTH_FEATURES").unwrap_or_default();
    overrides
        .split(',')
        .map(str::trim)
        .find_map(runtime_disable_override)
        .unwrap_or(false)
}

fn runtime_disable_override(value: &str) -> Option<bool> {
    match value {
        "+runtime-disable"
        | "runtime-disable=on"
        | "runtime-disable=true"
        | "runtime-disable=enabled" => Some(true),
        "-runtime-disable"
        | "runtime-disable=off"
        | "runtime-disable=false"
        | "runtime-disable=disabled" => Some(false),
        _ => None,
    }
}
