//! Session cleanup helpers.

#[cfg(target_os = "linux")]
use std::fs;
use std::process::{Command, Stdio};
use std::sync::Arc;

use chrono::Utc;

use super::metadata::SessionMetadata;
use super::store::SessionStore;
use crate::error::Result;
use crate::model::SessionSummary;

/// Best-effort process termination for detached Chrome instances.
pub fn terminate_process(pid: u32) {
    #[cfg(unix)]
    let _ = Command::new("kill")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Best-effort process termination for a detached Bowser session.
pub fn terminate_session_processes(session: &SessionMetadata) {
    terminate_process(session.pid);
    if let Some(pid) = xvfb_pid_to_terminate(session) {
        terminate_process(pid);
    }
}

pub(crate) fn xvfb_pid_to_terminate(session: &SessionMetadata) -> Option<u32> {
    let pid = session.xvfb_pid?;
    let display = session.xvfb_display.as_deref()?;
    if pid == session.pid || !process_matches_xvfb(pid, display) {
        return None;
    }
    Some(pid)
}

#[cfg(target_os = "linux")]
fn process_matches_xvfb(pid: u32, display: &str) -> bool {
    let path = format!("/proc/{pid}/cmdline");
    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(_) => return false,
    };
    let cmdline = contents
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .filter_map(|part| std::str::from_utf8(part).ok())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    xvfb_cmdline_matches(&cmdline, display)
}

#[cfg(not(target_os = "linux"))]
fn process_matches_xvfb(_pid: u32, _display: &str) -> bool {
    false
}

#[cfg(any(target_os = "linux", test))]
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
