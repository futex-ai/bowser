//! Page action helpers for interactive commands.

use std::sync::{Arc, Mutex};

use bowser::{BrowserEngine, PageEngine};

use crate::StructuredFormat;
use crate::commands::render_element;
use crate::commands::repl::HelperState;
use crate::error::{CliError, Result};
use crate::url::normalize_navigation_target;

use super::capture::capture_and_print;
use super::state::refresh_page_state;

pub(super) async fn navigate(
    page: &mut Box<dyn PageEngine>,
    url: &str,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    let url = normalize_navigation_target(url);
    page.navigate(&url).await?;
    capture_and_print(&**page, helper_state).await
}

pub(super) async fn go_back(
    page: &mut Box<dyn PageEngine>,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.back().await?;
    capture_and_print(&**page, helper_state).await
}

pub(super) async fn go_forward(
    page: &mut Box<dyn PageEngine>,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.forward().await?;
    capture_and_print(&**page, helper_state).await
}

pub(super) async fn reload(
    page: &mut Box<dyn PageEngine>,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.reload().await?;
    capture_and_print(&**page, helper_state).await
}

pub(super) async fn new_page(
    browser: &dyn BrowserEngine,
    page: &mut Box<dyn PageEngine>,
    config: &bowser::BrowserConfig,
    session_id: &str,
    url: Option<&str>,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    let url = url.map(normalize_navigation_target);
    *page = browser.new_page(url.as_deref()).await?;
    refresh_page_state(config, session_id, helper_state).await?;
    capture_and_print(&**page, helper_state).await
}

pub(super) async fn print_yaml(
    page: &dyn PageEngine,
    id: Option<u32>,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    if let Some(id) = id {
        println!(
            "{}",
            render_element(&page.capture_subtree(id).await?, StructuredFormat::Yaml)?
        );
        return Ok(());
    }
    capture_and_print(page, helper_state).await
}

pub(super) async fn click(
    page: &dyn PageEngine,
    id: u32,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.click(id).await?;
    capture_and_print(page, helper_state).await
}

pub(super) async fn press_keys(
    page: &dyn PageEngine,
    keys: &[String],
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.press_keys(keys).await?;
    capture_and_print(page, helper_state).await
}

pub(super) async fn type_text(
    page: &dyn PageEngine,
    id: u32,
    value: &str,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.type_text(id, value).await?;
    capture_and_print(page, helper_state).await
}

pub(super) async fn clear(
    page: &dyn PageEngine,
    id: u32,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.clear(id).await?;
    capture_and_print(page, helper_state).await
}

pub(super) async fn select_option(
    page: &dyn PageEngine,
    id: u32,
    value: &str,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.select_option(id, value).await?;
    capture_and_print(page, helper_state).await
}

pub(super) async fn submit(
    page: &dyn PageEngine,
    id: u32,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    page.submit(id).await?;
    capture_and_print(page, helper_state).await
}

pub(super) async fn screenshot(
    page: &dyn PageEngine,
    target: Option<u32>,
    path: Option<std::path::PathBuf>,
) -> Result<()> {
    let path = path.unwrap_or_else(crate::commands::repl::default_screenshot_path);
    let bytes = if let Some(id) = target {
        page.screenshot_element(id).await?
    } else {
        page.screenshot().await?
    };
    tokio::fs::write(&path, bytes)
        .await
        .map_err(|_| CliError::OutputWrite {
            path: path.display().to_string(),
        })?;
    println!("{}", path.display());
    Ok(())
}
