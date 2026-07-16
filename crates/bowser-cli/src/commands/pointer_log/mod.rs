//! Local pointer telemetry capture server.

mod demo;
mod event;
mod html;
mod log;
mod server;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;

use crate::PointerLogArgs;
use crate::error::{CliError, Result};

use self::log::{FilePointerEventLog, NoopPointerEventLog, PointerEventLog};

/// Runs the local pointer telemetry capture server.
pub async fn run(args: PointerLogArgs, demo_config: Option<bowser::BrowserConfig>) -> Result<()> {
    validate_loopback_bind(args.bind)?;
    let output = effective_output(args.demo, args.output.clone());
    let log = pointer_event_log(output.clone())?;
    let listener = match TcpListener::bind(args.bind).await {
        Ok(listener) => listener,
        Err(source) => {
            return Err(CliError::PointerLogBind {
                addr: args.bind,
                source,
            });
        }
    };
    let addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(source) => return Err(CliError::PointerLogLocalAddr { source }),
    };
    eprintln!("Pointer log server: http://{addr}");
    match output.as_ref() {
        Some(path) => eprintln!("Pointer log file: {}", path.display()),
        None => eprintln!("Pointer log file: disabled"),
    }
    if args.demo {
        let config = demo_config.ok_or(CliError::PointerLogDemoConfigMissing)?;
        let server = server::serve(listener, addr, log);
        tokio::pin!(server);
        let demo = demo::run(addr, config, args.demo_clicks);
        tokio::pin!(demo);
        tokio::select! {
            result = &mut demo => result,
            result = &mut server => result,
        }
    } else {
        server::serve(listener, addr, log).await
    }
}

pub(super) fn validate_loopback_bind(addr: SocketAddr) -> Result<()> {
    if addr.ip().is_loopback() {
        Ok(())
    } else {
        Err(CliError::PointerLogNonLoopback { addr })
    }
}

/// Resolves the pointer-log output policy for manual and demo modes.
pub(super) fn effective_output(demo: bool, output: Option<PathBuf>) -> Option<PathBuf> {
    if demo {
        output
    } else {
        Some(output.unwrap_or_else(|| PathBuf::from("browser-log")))
    }
}

fn pointer_event_log(output: Option<PathBuf>) -> Result<Arc<dyn PointerEventLog + Send + Sync>> {
    match output {
        Some(path) => Ok(Arc::new(FilePointerEventLog::open(path)?)),
        None => Ok(Arc::new(NoopPointerEventLog)),
    }
}

#[cfg(test)]
#[path = "_tests_/pointer_log_tests.rs"]
mod pointer_log_tests;
