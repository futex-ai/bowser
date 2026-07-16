//! Operating-system process inspection and termination boundary.

use std::process::{Command, Stdio};
use std::sync::Arc;

#[cfg_attr(test, unimock::unimock(api = ProcessControlMock))]
/// Inspects and terminates operating-system processes at an injectable boundary.
pub(crate) trait ProcessControl: Send + Sync {
    /// Returns the process command line when it can be inspected safely.
    fn command_line(&self, pid: u32) -> Option<Vec<String>>;
    /// Sends the platform's best-effort termination request.
    fn terminate(&self, pid: u32);
}

/// Process control backed by platform commands and process metadata.
pub(crate) struct SystemProcessControl;

impl ProcessControl for SystemProcessControl {
    fn command_line(&self, pid: u32) -> Option<Vec<String>> {
        process_command_line(pid)
    }

    fn terminate(&self, pid: u32) {
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
}

/// Terminates a newly owned Chrome/Xvfb pair unless launch finalization succeeds.
pub(crate) struct LaunchedProcessGuard {
    control: Arc<dyn ProcessControl>,
    chrome_pid: u32,
    xvfb_pid: Option<u32>,
    armed: bool,
}

impl LaunchedProcessGuard {
    /// Creates a guard using platform process control.
    pub(crate) fn system(chrome_pid: u32, xvfb_pid: Option<u32>) -> Self {
        Self::new(Arc::new(SystemProcessControl), chrome_pid, xvfb_pid)
    }

    /// Creates a guard with injected process control.
    pub(crate) fn new(
        control: Arc<dyn ProcessControl>,
        chrome_pid: u32,
        xvfb_pid: Option<u32>,
    ) -> Self {
        Self {
            control,
            chrome_pid,
            xvfb_pid,
            armed: true,
        }
    }

    /// Transfers process ownership to the completed browser session.
    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for LaunchedProcessGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        self.control.terminate(self.chrome_pid);
        if let Some(xvfb_pid) = self.xvfb_pid.filter(|pid| *pid != self.chrome_pid) {
            self.control.terminate(xvfb_pid);
        }
    }
}

#[cfg(target_os = "linux")]
fn process_command_line(pid: u32) -> Option<Vec<String>> {
    let contents = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    Some(
        contents
            .split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .filter_map(|part| std::str::from_utf8(part).ok())
            .map(ToString::to_string)
            .collect(),
    )
}

#[cfg(all(unix, not(target_os = "linux")))]
fn process_command_line(pid: u32) -> Option<Vec<String>> {
    let output = Command::new("ps")
        .args(["-ww", "-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let command_line = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!command_line.is_empty()).then(|| vec![command_line])
}

#[cfg(windows)]
fn process_command_line(pid: u32) -> Option<Vec<String>> {
    let script =
        format!("(Get-CimInstance Win32_Process -Filter \"ProcessId = {pid}\").CommandLine");
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let command_line = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!command_line.is_empty()).then(|| vec![command_line])
}
