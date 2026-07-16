//! Process ownership and cleanup regression tests.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use super::cleanup::{
    chrome_cmdline_matches, terminate_session_processes_with, xvfb_pid_to_terminate_with,
};
use super::metadata::SessionMetadata;
use super::process::{LaunchedProcessGuard, ProcessControl, ProcessControlMock};

#[test]
fn cleanup_skips_a_reused_chrome_pid() {
    let session = metadata(3333);
    let control = Unimock::new(
        ProcessControlMock::command_line
            .next_call(matching!(3333))
            .returns(Some(vec!["/usr/bin/sleep".to_string(), "100".to_string()])),
    );

    terminate_session_processes_with(&control, &session);
}

#[test]
fn cleanup_terminates_chrome_only_when_its_launch_identity_matches() {
    let session = metadata(3333);
    let control = Unimock::new((
        ProcessControlMock::command_line
            .next_call(matching!(3333))
            .returns(Some(vec![
                "/usr/bin/google-chrome".to_string(),
                "--remote-debugging-port=9222".to_string(),
                "--user-data-dir=/tmp/bowser-profile".to_string(),
            ])),
        ProcessControlMock::terminate
            .next_call(matching!(3333))
            .returns(()),
    ));

    terminate_session_processes_with(&control, &session);
}

#[test]
fn xvfb_cleanup_skips_ambiguous_process_metadata() {
    let control = Unimock::new(());
    let mut session = metadata(3333);
    session.xvfb_pid = Some(3333);
    session.xvfb_display = Some(":99".to_string());

    assert_eq!(xvfb_pid_to_terminate_with(&control, &session), None);

    session.xvfb_pid = Some(4444);
    session.xvfb_display = None;

    assert_eq!(xvfb_pid_to_terminate_with(&control, &session), None);
}

#[test]
fn chrome_identity_requires_the_expected_port_and_profile() {
    let session = metadata(3333);
    let wrong_port = vec![
        "/usr/bin/google-chrome".to_string(),
        "--remote-debugging-port=9333".to_string(),
        "--user-data-dir=/tmp/bowser-profile".to_string(),
    ];
    let wrong_profile = vec![
        "/usr/bin/google-chrome".to_string(),
        "--remote-debugging-port=9222".to_string(),
        "--user-data-dir=/tmp/other-profile".to_string(),
    ];

    assert!(!chrome_cmdline_matches(&wrong_port, &session));
    assert!(!chrome_cmdline_matches(&wrong_profile, &session));
}

#[test]
fn chrome_identity_accepts_quoted_arguments_from_process_listing() {
    let session = metadata(3333);
    let combined = vec![
        r#""C:\Program Files\Chrome\chrome.exe" "--remote-debugging-port=9222" "--user-data-dir=/tmp/bowser-profile""#
            .to_string(),
    ];

    assert!(chrome_cmdline_matches(&combined, &session));
}

#[test]
fn armed_launch_guard_terminates_every_fresh_process() {
    let terminated = Arc::new(Mutex::new(Vec::new()));
    let control: Arc<dyn ProcessControl> = Arc::new(Unimock::new(
        ProcessControlMock::terminate
            .each_call(matching!(_))
            .answers_arc({
                let terminated = terminated.clone();
                Arc::new(move |_, pid| {
                    terminated.lock().expect("terminated pids").push(pid);
                })
            }),
    ));

    {
        let _guard = LaunchedProcessGuard::new(control, 3333, Some(4444));
    }

    assert_eq!(
        terminated.lock().expect("terminated pids").as_slice(),
        [3333, 4444]
    );
}

#[test]
fn disarmed_launch_guard_leaves_the_live_processes_running() {
    let control: Arc<dyn ProcessControl> = Arc::new(Unimock::new(()));
    let mut guard = LaunchedProcessGuard::new(control, 3333, Some(4444));

    guard.disarm();
}

fn metadata(pid: u32) -> SessionMetadata {
    let mut metadata = SessionMetadata::new(
        "http://127.0.0.1:9222".to_string(),
        "ws://127.0.0.1:9222/devtools/browser/test".to_string(),
        pid,
        PathBuf::from("/tmp/bowser-profile"),
    );
    metadata.id = "bsr_process_test".to_string();
    metadata
}
