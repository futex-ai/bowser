//! `bowser get`.

use std::path::PathBuf;
use std::time::Duration;

use bowser::{Browser, BrowserEngine};

use crate::GetArgs;
use crate::commands::{render_capture, write_output};
use crate::error::Result;
use crate::url::normalize_navigation_target;
use crate::{PageFormat, StructuredFormat};

/// Runs the single-shot capture flow.
pub async fn run(config: bowser::BrowserConfig, args: GetArgs) -> Result<()> {
    let browser = Browser::launch(config).await?;
    let session = browser.session_info().await?;
    let operation_result = execute(&browser, args).await;
    let detach_result = browser.detach().await;
    eprintln!("Session: {}", session.id);
    operation_result?;
    detach_result?;
    Ok(())
}

async fn execute(browser: &dyn BrowserEngine, args: GetArgs) -> Result<()> {
    let page = browser.current_page().await?;
    let url = normalize_navigation_target(&args.url);
    page.navigate(&url).await?;
    if let Some(selector) = args.wait.as_deref() {
        page.wait_for_selector(selector, Duration::from_secs(args.wait_timeout))
            .await?;
    }
    page.wait_for_stable(Duration::from_secs(10)).await.ok();
    if args.delay > 0 {
        tokio::time::sleep(Duration::from_millis(args.delay)).await;
    }
    let capture = page.capture().await?;
    if let Some(path) = args.screenshot.as_ref() {
        let png = page.screenshot().await?;
        save_binary(path, &png).await?;
    }
    let output = match args.format {
        PageFormat::Html => page.rendered_html().await?,
        PageFormat::Yaml => render_capture(&capture, StructuredFormat::Yaml)?,
        PageFormat::Json => render_capture(&capture, StructuredFormat::Json)?,
    };
    write_output(args.output.as_deref(), &output).await?;
    Ok(())
}

async fn save_binary(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    tokio::fs::write(path, bytes)
        .await
        .map_err(|_| crate::error::CliError::OutputWrite {
            path: path.display().to_string(),
        })
}
