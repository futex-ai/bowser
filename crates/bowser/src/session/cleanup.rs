//! Session cleanup helpers.

use std::sync::Arc;

use chrono::Utc;
use url::Url;

use crate::error::Result;
use crate::model::SessionSummary;

use super::metadata::SessionMetadata;
use super::process::{ProcessControl, SystemProcessControl};
use super::store::SessionStore;

/// Best-effort process termination for detached Chrome instances.
pub fn terminate_process(pid: u32) {
    SystemProcessControl.terminate(pid);
}

/// Best-effort process termination for a detached Bowser session.
pub fn terminate_session_processes(session: &SessionMetadata) {
    terminate_session_processes_with(&SystemProcessControl, session);
}

pub(crate) fn terminate_session_processes_with(
    control: &dyn ProcessControl,
    session: &SessionMetadata,
) {
    if control
        .command_line(session.pid)
        .is_some_and(|cmdline| chrome_cmdline_matches(&cmdline, session))
    {
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

pub(crate) fn chrome_cmdline_matches(cmdline: &[String], session: &SessionMetadata) -> bool {
    let Some(port) = Url::parse(&session.http_url)
        .ok()
        .and_then(|url| url.port())
    else {
        return false;
    };
    let port_argument = format!("--remote-debugging-port={port}");
    let profile_argument = format!("--user-data-dir={}", session.user_data_dir.display());
    has_argument(cmdline, &port_argument) && has_argument(cmdline, &profile_argument)
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
            terminate_session_processes(&session);
            store.remove(&session.id).await?;
        } else {
            summaries.push(session.summary());
        }
    }
    Ok(summaries)
}
