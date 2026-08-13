//! `bowser get`.

use std::time::Duration;

use bowser::{Browser, BrowserEngine};

use crate::GetArgs;
use crate::commands::page_output;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};
use crate::url::normalize_navigation_target;

/// Runs the single-shot capture flow.
pub async fn run(
    config: bowser::BrowserConfig,
    args: GetArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    let browser = Browser::launch(config).await?;
    let session = browser.session_info().await?;
    context.update_session(&session);
    context.announce_session_on_failure = true;
    let operation_result = execute(&browser, args).await;
    context.update_session(&browser.session_info().await?);
    let detach_result = browser.detach().await;
    let output = operation_result?;
    detach_result?;
    Ok(output.human_stderr(format!("Session: {}\n", session.id)))
}

async fn execute(browser: &dyn BrowserEngine, args: GetArgs) -> Result<CommandOutput> {
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
    let screenshot = if let Some(path) = args.screenshot.as_ref() {
        let png = page.screenshot().await?;
        Some((path.clone(), png))
    } else {
        None
    };
    let mut output = page_output(page.as_ref(), args.format, args.output.as_deref()).await?;
    if let Some((path, png)) = screenshot {
        let mut result = output.result.as_object().cloned().unwrap_or_default();
        result.insert(
            "screenshot".to_string(),
            serde_json::json!({ "path": path, "bytes": png.len() }),
        );
        output.result = serde_json::Value::Object(result);
        output = output.binary_file(path, png);
    }
    Ok(output)
}
