//! Headed Bowser demo driver for the pointer telemetry page.

use std::net::SocketAddr;
use std::time::Duration;

use bowser::{Browser, BrowserEngine, Element, InputType, PageCapture, PageEngine};

use crate::error::{CliError, Result};

/// Launches headed Chrome and repeatedly clicks randomized pointer-log targets.
pub(super) async fn run(
    addr: SocketAddr,
    mut config: bowser::BrowserConfig,
    max_clicks: Option<u32>,
) -> Result<()> {
    config.headless = false;
    config.session.id = None;

    let browser = Browser::launch(config).await?;
    let page = browser.current_page().await?;
    page.navigate(&format!("http://{addr}/?demo=1")).await?;
    page.wait_for_selector("[data-pointer-target]", Duration::from_secs(10))
        .await?;
    eprintln!("Pointer log demo: driving headed Chrome; press Ctrl-C to stop");

    let result = drive_page(page.as_ref(), max_clicks).await;
    if let Err(err) = result {
        let _ = flush_events(page.as_ref()).await;
        let _ = browser.close().await;
        return Err(err);
    }
    flush_events(page.as_ref()).await?;
    browser.close().await?;
    Ok(())
}

async fn drive_page(page: &dyn PageEngine, max_clicks: Option<u32>) -> Result<()> {
    let mut clicks = 0_u32;
    loop {
        if max_clicks.is_some_and(|max| clicks >= max) {
            return Ok(());
        }
        let click = click_next_target(page);
        tokio::pin!(click);
        tokio::select! {
            result = &mut click => {
                result?;
                clicks += 1;
                eprintln!("Pointer log demo clicks: {clicks}");
            }
            signal = tokio::signal::ctrl_c() => {
                return match signal {
                    Ok(()) => Ok(()),
                    Err(source) => Err(CliError::PointerLogSignal { source }),
                };
            }
        }
    }
}

async fn click_next_target(page: &dyn PageEngine) -> Result<()> {
    page.wait_for_selector("[data-pointer-target]", Duration::from_secs(10))
        .await?;
    let capture = page.capture().await?;
    let element_id = target_element_id(&capture).ok_or(CliError::PointerLogDemoTargetMissing)?;
    page.click(element_id).await?;
    tokio::time::sleep(Duration::from_millis(220)).await;
    Ok(())
}

async fn flush_events(page: &dyn PageEngine) -> Result<()> {
    let _ = page
        .evaluate_js(
            r#"
(async () => {
  if (typeof window.__pointerLogFlush !== 'function') return false;
  await window.__pointerLogFlush();
  return true;
})()
"#,
        )
        .await?;
    Ok(())
}

/// Returns the current randomized target element ID from a pointer-log capture.
pub(super) fn target_element_id(capture: &PageCapture) -> Option<u32> {
    capture
        .content
        .buckets()
        .into_iter()
        .flat_map(|children| children.iter())
        .find_map(target_id_for_element)
}

fn target_id_for_element(element: &Element) -> Option<u32> {
    if is_demo_target(element) {
        return element.id();
    }
    match element {
        Element::Table { headers, rows, .. } => headers
            .iter()
            .flat_map(|cell| cell.children.iter())
            .chain(
                rows.iter()
                    .flat_map(|row| row.cells.iter())
                    .flat_map(|cell| cell.children.iter()),
            )
            .find_map(target_id_for_element),
        Element::List { items, .. } => items
            .iter()
            .flat_map(|item| item.children.iter())
            .find_map(target_id_for_element),
        Element::Nav { children, .. }
        | Element::Form { children, .. }
        | Element::Section { children, .. }
        | Element::Iframe { children, .. } => children.iter().find_map(target_id_for_element),
        Element::Heading { .. }
        | Element::Text { .. }
        | Element::Link { .. }
        | Element::Button { .. }
        | Element::Input { .. }
        | Element::Image { .. } => None,
    }
}

fn is_demo_target(element: &Element) -> bool {
    match element {
        Element::Button { text, .. } => matches!(text.as_str(), "Button" | "Element"),
        Element::Link { text, .. } => text == "Link",
        Element::Input {
            id: Some(_),
            input_type,
            value,
            label,
            ..
        } => matches!(
            (input_type, value.as_str(), label.as_deref()),
            (InputType::Text, "Input", _)
                | (InputType::Textarea, "Textarea", _)
                | (InputType::Select, "Select", _)
                | (InputType::Checkbox, _, Some("checkbox"))
        ),
        Element::Heading { .. }
        | Element::Text { .. }
        | Element::Input { id: None, .. }
        | Element::Image { .. }
        | Element::Table { .. }
        | Element::List { .. }
        | Element::Nav { .. }
        | Element::Form { .. }
        | Element::Section { .. }
        | Element::Iframe { .. } => false,
    }
}
