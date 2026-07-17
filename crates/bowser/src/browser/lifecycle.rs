//! Chrome launch, attach, and configuration helpers.

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use chromiumoxide::browser::Browser as ChromiumBrowser;
use futures::StreamExt;

use crate::{
    browser_identity,
    config::BrowserConfig,
    debug_port::{
        DYNAMIC_DEBUG_PORT_ARGUMENT, DebugPortReader, FileDebugPortReader, discover_debug_endpoint,
    },
    error::{Error, Result},
    session::{LaunchedProcessGuard, SessionMetadata, terminate_process},
    stealth_features::StealthFeatures,
};

use super::display::{XvfbSession, prepare_headed_display};

pub(super) fn resolve_chrome_path(path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = path {
        return Ok(path.to_path_buf());
    }
    let candidates = [
        "google-chrome",
        "chromium",
        "chromium-browser",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
    ];
    for candidate in candidates {
        if let Ok(found) = which::which(candidate) {
            return Ok(found);
        }
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(Error::BrowserLaunch {
        reason: "could not locate Chrome or Chromium".to_string(),
    })
}

pub(super) fn resolve_user_data_dir(config: &BrowserConfig, session_root: &Path) -> PathBuf {
    if let Some(path) = config.user_data_dir.clone() {
        return path;
    }
    if config.persistent_profile {
        return session_root.join("profiles").join("default");
    }
    session_root
        .join("profiles")
        .join(uuid::Uuid::new_v4().to_string())
}

pub(super) fn owns_user_data_dir(config: &BrowserConfig) -> bool {
    config.user_data_dir.is_none() && !config.persistent_profile
}

pub(super) fn build_chrome_args(
    config: &BrowserConfig,
    user_data_dir: &Path,
    synthetic_display: bool,
) -> Vec<String> {
    let mut args = Vec::new();
    if effective_headless(config) {
        args.push("--headless=new".to_string());
    }
    args.extend(browser_identity::stealth_launch_args(config.stealth));
    args.push("--no-first-run".to_string());
    args.push("--no-default-browser-check".to_string());
    if force_window_size(config, synthetic_display) {
        args.push(format!(
            "--window-size={},{}",
            config.viewport.width, config.viewport.height
        ));
    }
    args.extend(config.chrome_args.iter().cloned());
    args.push(DYNAMIC_DEBUG_PORT_ARGUMENT.to_string());
    args.push(format!("--user-data-dir={}", user_data_dir.display()));
    args
}

pub(super) fn effective_headless(config: &BrowserConfig) -> bool {
    config.headless && !(config.stealth && StealthFeatures::from_config(config).launch_headed())
}

pub(super) fn force_window_size(config: &BrowserConfig, synthetic_display: bool) -> bool {
    let features = StealthFeatures::from_config(config);
    synthetic_display
        || effective_headless(config)
        || (config.headless
            && config.stealth
            && features.launch_headed()
            && !features.launch_native_window())
}

pub(super) async fn launch_chrome(
    config: &BrowserConfig,
    chrome_path: &Path,
    user_data_dir: &Path,
) -> Result<(
    u32,
    String,
    String,
    Option<XvfbSession>,
    LaunchedProcessGuard,
)> {
    let debug_port_reader = FileDebugPortReader;
    let previous_debug_port = debug_port_reader.read(user_data_dir);
    let display = prepare_headed_display(config)?;
    let mut command = Command::new(chrome_path);
    #[cfg(unix)]
    command.process_group(0);
    for arg in build_chrome_args(config, user_data_dir, display.xvfb.is_some()) {
        command.arg(arg);
    }
    if let Some(display_env) = display.display_env.as_deref() {
        command.env("DISPLAY", display_env);
    }
    command
        .arg("about:blank")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            if let Some(xvfb) = display.xvfb.as_ref() {
                terminate_process(xvfb.pid);
            }
            return Err(Error::BrowserLaunch {
                reason: err.to_string(),
            });
        }
    };
    let pid = child.id();
    let process_guard =
        LaunchedProcessGuard::system(pid, display.xvfb.as_ref().map(|xvfb| xvfb.pid));
    let (http_url, websocket_url) = discover_debug_endpoint(
        &debug_port_reader,
        user_data_dir,
        previous_debug_port.as_ref(),
        config.timeout,
    )
    .await?;
    Ok((pid, http_url, websocket_url, display.xvfb, process_guard))
}

pub(super) async fn connect_browser(
    http_url: &str,
) -> Result<(ChromiumBrowser, tokio::task::JoinHandle<()>)> {
    let (browser, mut handler) = ChromiumBrowser::connect(http_url.to_string())
        .await
        .map_err(|err| Error::cdp(format!("failed to connect to browser: {err}")))?;
    let task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if event.is_err() {
                break;
            }
        }
    });
    Ok((browser, task))
}

pub(super) fn validate_session_age(metadata: &SessionMetadata, ttl: Duration) -> Result<()> {
    let idle = chrono::Utc::now()
        .signed_duration_since(metadata.updated_at)
        .to_std()
        .unwrap_or_default();
    if idle > ttl {
        return Err(Error::SessionExpired {
            session_id: metadata.id.clone(),
        });
    }
    Ok(())
}
