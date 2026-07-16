//! Interrupt handling for interactive mode.

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};

use rustyline::error::{ReadlineError, Signal};

use crate::error::{CliError, Result};

#[cfg(unix)]
pub(crate) static REPL_SIGINT_PENDING: AtomicBool = AtomicBool::new(false);

const COMMAND_INTERRUPT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

pub(crate) enum CommandExecution<T> {
    Completed(T),
    Interrupted,
    TimedOut,
}

pub(crate) async fn run_interruptible<F, T>(
    future: F,
    timeout: std::time::Duration,
) -> CommandExecution<T>
where
    F: Future<Output = T>,
{
    tokio::pin!(future);
    let timeout_future = tokio::time::sleep(timeout);
    tokio::pin!(timeout_future);
    #[cfg(unix)]
    {
        let interrupt_future = wait_for_command_interrupt();
        tokio::pin!(interrupt_future);
        tokio::select! {
            result = &mut future => CommandExecution::Completed(result),
            _ = &mut interrupt_future => CommandExecution::Interrupted,
            _ = &mut timeout_future => CommandExecution::TimedOut,
        }
    }
    #[cfg(not(unix))]
    {
        tokio::select! {
            result = &mut future => CommandExecution::Completed(result),
            _ = &mut timeout_future => CommandExecution::TimedOut,
        }
    }
}

pub(crate) fn should_exit_on_interrupt(interrupted_once: &mut bool) -> bool {
    if *interrupted_once {
        true
    } else {
        *interrupted_once = true;
        false
    }
}

pub(super) fn install_sigint_handler() -> Result<SigintHandlerGuard> {
    #[cfg(unix)]
    {
        REPL_SIGINT_PENDING.store(false, Ordering::SeqCst);
        let action = nix::sys::signal::SigAction::new(
            nix::sys::signal::SigHandler::Handler(sigint_handler),
            nix::sys::signal::SaFlags::empty(),
            nix::sys::signal::SigSet::empty(),
        );
        let previous =
            unsafe { nix::sys::signal::sigaction(nix::sys::signal::Signal::SIGINT, &action) }
                .map_err(|err| CliError::SignalHandler {
                    reason: err.to_string(),
                })?;
        Ok(SigintHandlerGuard { previous })
    }
    #[cfg(not(unix))]
    {
        Ok(SigintHandlerGuard)
    }
}

pub(super) fn is_interrupt(err: &ReadlineError) -> bool {
    match err {
        ReadlineError::Interrupted | ReadlineError::Signal(Signal::Interrupt) => true,
        ReadlineError::Io(io_err) if io_err.kind() == std::io::ErrorKind::Interrupted => true,
        #[cfg(unix)]
        ReadlineError::Errno(errno) if *errno == nix::errno::Errno::EINTR => true,
        _ => take_pending_sigint(),
    }
}

pub(crate) fn clear_pending_sigint() {
    #[cfg(unix)]
    REPL_SIGINT_PENDING.store(false, Ordering::SeqCst);
}

#[cfg(unix)]
async fn wait_for_command_interrupt() {
    loop {
        if take_pending_sigint() {
            return;
        }
        tokio::time::sleep(COMMAND_INTERRUPT_POLL_INTERVAL).await;
    }
}

#[cfg(not(unix))]
async fn wait_for_command_interrupt() {
    std::future::pending::<()>().await;
}

#[cfg(unix)]
fn take_pending_sigint() -> bool {
    REPL_SIGINT_PENDING.swap(false, Ordering::SeqCst)
}

#[cfg(not(unix))]
fn take_pending_sigint() -> bool {
    false
}

#[cfg(unix)]
extern "C" fn sigint_handler(_: i32) {
    REPL_SIGINT_PENDING.store(true, Ordering::SeqCst);
}

#[cfg(unix)]
pub(super) struct SigintHandlerGuard {
    previous: nix::sys::signal::SigAction,
}

#[cfg(not(unix))]
pub(super) struct SigintHandlerGuard;

#[cfg(unix)]
impl Drop for SigintHandlerGuard {
    fn drop(&mut self) {
        let _ = unsafe {
            nix::sys::signal::sigaction(nix::sys::signal::Signal::SIGINT, &self.previous)
        };
    }
}
