//! Session cleanup helpers.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use url::Url;

use crate::debug_port::{DYNAMIC_DEBUG_PORT_ARGUMENT, DebugPortReader, FileDebugPortReader};
use crate::error::Result;
use crate::model::SessionSummary;

use super::metadata::SessionMetadata;
use super::process::{ProcessControl, SystemProcessControl};
use super::profile::remove_owned_session_profile;
use super::store::SessionStore;

const GRACEFUL_EXIT_TIMEOUT: Duration = Duration::from_secs(2);
const FORCED_EXIT_TIMEOUT: Duration = Duration::from_secs(2);
const PROCESS_EXIT_POLL: Duration = Duration::from_millis(50);

/// Best-effort process termination for detached Chrome instances.
pub fn terminate_process(pid: u32) {
    SystemProcessControl.terminate(pid);
}

/// Best-effort process termination for a detached Bowser session.
pub fn terminate_session_processes(session: &SessionMetadata) {
    terminate_session_processes_with(
        &SystemProcessControl,
        read_active_debug_port(session),
        session,
    );
}

pub(crate) fn validate_session_process_identity(session: &SessionMetadata) -> Result<()> {
    validate_session_process_identity_with(
        &SystemProcessControl,
        read_active_debug_port(session),
        session,
    )
}

pub(crate) fn validate_session_process_identity_with(
    control: &dyn ProcessControl,
    active_debug_port: Option<u16>,
    session: &SessionMetadata,
) -> Result<()> {
    if control.command_line(session.pid).is_some_and(|cmdline| {
        chrome_process_identity_matches(&cmdline, active_debug_port, session)
    }) {
        return Ok(());
    }
    Err(crate::error::Error::SessionProcessIdentityMismatch {
        session_id: session.id.clone(),
    })
}

/// Terminates a detached session and waits until its Chrome identity disappears.
pub async fn terminate_session_processes_and_wait(session: &SessionMetadata) -> Result<()> {
    terminate_session_processes_and_wait_with(
        &SystemProcessControl,
        read_active_debug_port(session),
        session,
        GRACEFUL_EXIT_TIMEOUT,
        FORCED_EXIT_TIMEOUT,
    )
    .await
}

pub(crate) async fn terminate_session_processes_and_wait_with(
    control: &dyn ProcessControl,
    active_debug_port: Option<u16>,
    session: &SessionMetadata,
    graceful_timeout: Duration,
    forced_timeout: Duration,
) -> Result<()> {
    terminate_session_processes_with(control, active_debug_port, session);
    if wait_for_chrome_exit(control, active_debug_port, session, graceful_timeout).await {
        return Ok(());
    }
    if chrome_process_is_running(control, active_debug_port, session) {
        control.force_terminate(session.pid);
    }
    if wait_for_chrome_exit(control, active_debug_port, session, forced_timeout).await {
        return Ok(());
    }
    Err(crate::error::Error::SessionProcessStillRunning { pid: session.pid })
}

pub(crate) fn terminate_session_processes_with(
    control: &dyn ProcessControl,
    active_debug_port: Option<u16>,
    session: &SessionMetadata,
) {
    if control.command_line(session.pid).is_some_and(|cmdline| {
        chrome_process_identity_matches(&cmdline, active_debug_port, session)
    }) {
        control.terminate(session.pid);
    }
    if let Some(pid) = xvfb_pid_to_terminate_with(control, session) {
        control.terminate(pid);
    }
}

pub(crate) fn xvfb_pid_to_terminate_with(
    control: &dyn ProcessControl,
    session: &SessionMetadata,
) -> Option<u32> {
    let pid = session.xvfb_pid?;
    let display = session.xvfb_display.as_deref()?;
    if pid == session.pid
        || !control
            .command_line(pid)
            .is_some_and(|cmdline| xvfb_cmdline_matches(&cmdline, display))
    {
        return None;
    }
    Some(pid)
}

pub(crate) fn chrome_process_identity_matches(
    cmdline: &[String],
    active_debug_port: Option<u16>,
    session: &SessionMetadata,
) -> bool {
    match chrome_debug_port_mode(cmdline, session) {
        Some(DebugPortMode::Fixed) => true,
        Some(DebugPortMode::Dynamic) => active_debug_port == session_debug_port(session),
        None => false,
    }
}

fn chrome_debug_port_mode(cmdline: &[String], session: &SessionMetadata) -> Option<DebugPortMode> {
    let port = session_debug_port(session)?;
    let profile_argument = format!("--user-data-dir={}", session.user_data_dir.display());
    if !has_argument(cmdline, &profile_argument) {
        return None;
    }
    let fixed_port_argument = format!("--remote-debugging-port={port}");
    if has_argument(cmdline, &fixed_port_argument) {
        return Some(DebugPortMode::Fixed);
    }
    has_argument(cmdline, DYNAMIC_DEBUG_PORT_ARGUMENT).then_some(DebugPortMode::Dynamic)
}

fn session_debug_port(session: &SessionMetadata) -> Option<u16> {
    Url::parse(&session.http_url).ok()?.port()
}

fn read_active_debug_port(session: &SessionMetadata) -> Option<u16> {
    FileDebugPortReader
        .read(&session.user_data_dir)
        .map(|active_port| active_port.port())
}

#[derive(Clone, Copy)]
enum DebugPortMode {
    Fixed,
    Dynamic,
}

async fn wait_for_chrome_exit(
    control: &dyn ProcessControl,
    active_debug_port: Option<u16>,
    session: &SessionMetadata,
    timeout: Duration,
) -> bool {
    let started = tokio::time::Instant::now();
    loop {
        if !chrome_process_is_running(control, active_debug_port, session) {
            return true;
        }
        if started.elapsed() >= timeout {
            return false;
        }
        tokio::time::sleep(PROCESS_EXIT_POLL.min(timeout.saturating_sub(started.elapsed()))).await;
    }
}

fn chrome_process_is_running(
    control: &dyn ProcessControl,
    active_debug_port: Option<u16>,
    session: &SessionMetadata,
) -> bool {
    control.command_line(session.pid).is_some_and(|cmdline| {
        chrome_process_identity_matches(&cmdline, active_debug_port, session)
    })
}

fn has_argument(cmdline: &[String], expected: &str) -> bool {
    cmdline.iter().any(|argument| argument == expected)
        || (cmdline.len() == 1 && combined_command_has_argument(&cmdline[0], expected))
}

fn combined_command_has_argument(command_line: &str, expected: &str) -> bool {
    command_line.match_indices(expected).any(|(index, _)| {
        let before = command_line[..index].chars().next_back();
        let after = command_line[index + expected.len()..].chars().next();
        before.is_none_or(is_argument_boundary) && after.is_none_or(is_argument_boundary)
    })
}

fn is_argument_boundary(character: char) -> bool {
    character.is_whitespace() || matches!(character, '\'' | '"')
}

pub(crate) fn xvfb_cmdline_matches(cmdline: &[String], display: &str) -> bool {
    let Some(program) = cmdline.first() else {
        return false;
    };
    let is_xvfb = program
        .rsplit(std::path::MAIN_SEPARATOR)
        .next()
        .is_some_and(|name| name == "Xvfb");
    is_xvfb && cmdline.iter().any(|arg| arg == display)
}

/// Removes expired sessions and returns the remaining session summaries.
pub async fn cleanup_expired_sessions(
    store: Arc<dyn SessionStore>,
    session_root: &Path,
    ttl: std::time::Duration,
) -> Result<Vec<SessionSummary>> {
    let now = Utc::now();
    let mut summaries = Vec::new();
    for session in store.list().await? {
        let idle = now
            .signed_duration_since(session.updated_at)
            .to_std()
            .unwrap_or_default();
        if idle > ttl {
            terminate_session_processes_and_wait(&session).await?;
            remove_owned_session_profile(session_root, &session).await?;
            store.remove(&session.id).await?;
        } else {
            summaries.push(session.summary());
        }
    }
    Ok(summaries)
}
