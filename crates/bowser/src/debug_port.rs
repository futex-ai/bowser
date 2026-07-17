//! Chrome dynamic-debug-port discovery and profile handshake validation.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::error::{Error, Result};

pub(crate) const DYNAMIC_DEBUG_PORT_ARGUMENT: &str = "--remote-debugging-port=0";
const ACTIVE_PORT_FILE: &str = "DevToolsActivePort";
const MAX_ACTIVE_PORT_BYTES: u64 = 1024;
const ENDPOINT_POLL_INTERVAL: Duration = Duration::from_millis(100);
const ENDPOINT_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, PartialEq)]
#[expect(
    unreachable_pub,
    reason = "unimock's generated public DebugPortReader mock exposes this type"
)]
/// Parsed identity from Chrome's profile-local dynamic debug-port handshake.
pub struct ActiveDebugPort {
    fingerprint: Vec<u8>,
    port: u16,
}

impl ActiveDebugPort {
    pub(crate) fn port(&self) -> u16 {
        self.port
    }

    fn is_fresh_for(&self, baseline: Option<&Self>) -> bool {
        baseline != Some(self)
    }
}

#[cfg_attr(test, unimock::unimock(api = DebugPortReaderMock))]
pub(crate) trait DebugPortReader: Send + Sync {
    fn read(&self, user_data_dir: &Path) -> Option<ActiveDebugPort>;
}

pub(crate) struct FileDebugPortReader;

impl DebugPortReader for FileDebugPortReader {
    fn read(&self, user_data_dir: &Path) -> Option<ActiveDebugPort> {
        let path = user_data_dir.join(ACTIVE_PORT_FILE);
        let metadata = fs::symlink_metadata(&path).ok()?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.len() > MAX_ACTIVE_PORT_BYTES
        {
            return None;
        }
        let file = fs::File::open(path).ok()?;
        let metadata = file.metadata().ok()?;
        if !metadata.is_file() || metadata.len() > MAX_ACTIVE_PORT_BYTES {
            return None;
        }
        let mut contents = Vec::with_capacity(metadata.len() as usize);
        file.take(MAX_ACTIVE_PORT_BYTES + 1)
            .read_to_end(&mut contents)
            .ok()?;
        if contents.len() as u64 > MAX_ACTIVE_PORT_BYTES {
            return None;
        }
        parse_active_debug_port(contents)
    }
}

pub(crate) async fn discover_debug_endpoint(
    reader: &dyn DebugPortReader,
    user_data_dir: &Path,
    baseline: Option<&ActiveDebugPort>,
    timeout: Duration,
) -> Result<(String, String)> {
    let client = reqwest::Client::new();
    let started = tokio::time::Instant::now();
    loop {
        let remaining = timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            break;
        }
        if let Some(active_port) = reader
            .read(user_data_dir)
            .filter(|active_port| active_port.is_fresh_for(baseline))
        {
            let http_url = format!("http://127.0.0.1:{}", active_port.port());
            let request_timeout = remaining.min(ENDPOINT_REQUEST_TIMEOUT);
            if let Ok(Some(websocket_url)) =
                tokio::time::timeout(request_timeout, fetch_browser_websocket(&client, &http_url))
                    .await
            {
                return Ok((http_url, websocket_url));
            }
        }
        tokio::time::sleep(ENDPOINT_POLL_INTERVAL.min(remaining)).await;
    }
    Err(Error::BrowserLaunch {
        reason: "timed out waiting for Chrome dynamic debug endpoint".to_string(),
    })
}

fn parse_active_debug_port(contents: Vec<u8>) -> Option<ActiveDebugPort> {
    let text = std::str::from_utf8(&contents).ok()?;
    let mut lines = text.lines();
    let port = lines.next()?.parse::<u16>().ok()?;
    let websocket_path = lines.next()?;
    if port == 0 || !websocket_path.starts_with("/devtools/browser/") {
        return None;
    }
    Some(ActiveDebugPort {
        fingerprint: contents,
        port,
    })
}

async fn fetch_browser_websocket(client: &reqwest::Client, http_url: &str) -> Option<String> {
    let response = client
        .get(format!("{http_url}/json/version"))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    response
        .json::<DebugVersion>()
        .await
        .ok()
        .map(|version| version.websocket_url)
}

#[derive(Deserialize)]
struct DebugVersion {
    #[serde(rename = "webSocketDebuggerUrl")]
    websocket_url: String,
}

#[cfg(test)]
#[path = "_tests_/debug_port_tests.rs"]
mod debug_port_tests;
