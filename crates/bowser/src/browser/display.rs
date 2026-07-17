//! Headed browser display preparation.

#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::time::Duration;

use crate::config::BrowserConfig;
#[cfg(target_os = "linux")]
use crate::error::Error;
use crate::error::Result;

use super::lifecycle::effective_headless;

/// Xvfb process metadata owned by a Bowser session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct XvfbSession {
    pub(crate) pid: u32,
    pub(crate) display: String,
}

pub(super) struct PreparedDisplay {
    pub(super) display_env: Option<String>,
    pub(super) xvfb: Option<XvfbSession>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DisplayPlan {
    Headless,
    ExistingDisplay,
    ExistingWayland,
    SpawnXvfb,
    UnsupportedHeaded,
}

pub(super) fn display_plan(
    headless: bool,
    display: Option<&str>,
    wayland_display: Option<&str>,
    linux: bool,
) -> DisplayPlan {
    if headless {
        return DisplayPlan::Headless;
    }
    if display.is_some_and(|value| !value.is_empty()) {
        return DisplayPlan::ExistingDisplay;
    }
    if wayland_display.is_some_and(|value| !value.is_empty()) {
        return DisplayPlan::ExistingWayland;
    }
    if linux {
        DisplayPlan::SpawnXvfb
    } else {
        DisplayPlan::UnsupportedHeaded
    }
}

pub(super) fn prepare_headed_display(config: &BrowserConfig) -> Result<PreparedDisplay> {
    match display_plan(
        effective_headless(config),
        std::env::var("DISPLAY").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
        cfg!(target_os = "linux"),
    ) {
        DisplayPlan::Headless | DisplayPlan::ExistingDisplay | DisplayPlan::ExistingWayland => {
            Ok(PreparedDisplay {
                display_env: None,
                xvfb: None,
            })
        }
        DisplayPlan::SpawnXvfb => spawn_xvfb(config),
        DisplayPlan::UnsupportedHeaded => Ok(PreparedDisplay {
            display_env: None,
            xvfb: None,
        }),
    }
}

#[cfg(target_os = "linux")]
fn spawn_xvfb(config: &BrowserConfig) -> Result<PreparedDisplay> {
    let display = free_xvfb_display()?;
    let geometry = format!("{}x{}x24", config.viewport.width, config.viewport.height);
    let child = Command::new("Xvfb")
        .arg(&display)
        .arg("-screen")
        .arg("0")
        .arg(geometry)
        .arg("-nolisten")
        .arg("tcp")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| Error::BrowserLaunch {
            reason: format!("failed to start Xvfb for headed Chrome: {err}"),
        })?;
    std::thread::sleep(Duration::from_millis(250));
    Ok(PreparedDisplay {
        display_env: Some(display.clone()),
        xvfb: Some(XvfbSession {
            pid: child.id(),
            display,
        }),
    })
}

#[cfg(not(target_os = "linux"))]
fn spawn_xvfb(_config: &BrowserConfig) -> Result<PreparedDisplay> {
    Ok(PreparedDisplay {
        display_env: None,
        xvfb: None,
    })
}

#[cfg(target_os = "linux")]
fn free_xvfb_display() -> Result<String> {
    for display in 99..200 {
        let socket = format!("/tmp/.X11-unix/X{display}");
        if !std::path::Path::new(&socket).exists() {
            return Ok(format!(":{display}"));
        }
    }
    Err(Error::BrowserLaunch {
        reason: "could not find a free Xvfb display".to_string(),
    })
}

#[cfg(test)]
#[path = "_tests_/display_tests.rs"]
mod display_tests;
