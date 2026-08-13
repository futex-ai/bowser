//! `bowser download`.

use std::path::PathBuf;

use bowser::{Browser, BrowserEngine, MetadataRecord, PageEngine};

use crate::DownloadArgs;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};
use crate::url::normalize_navigation_target;

/// Runs the browser-native download flow.
pub async fn run(
    config: bowser::BrowserConfig,
    args: DownloadArgs,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    let direct_request = direct_download_request(&config, &args)?;
    let browser = Browser::launch(config).await?;
    let session = browser.session_info().await?;
    context.update_session(&session);
    let page = browser.current_page().await?;
    let request = match direct_request {
        Some(request) => Ok(request),
        None => link_download_request(&args, page.as_ref()).await,
    };
    let result = match request {
        Ok((url, path)) => page.download(&url, &path).await,
        Err(err) => Err(err),
    };
    let detach = browser.detach().await;
    let result = result?;
    detach?;
    CommandOutput::result(serde_json::json!({
        "path": result.path,
        "bytes": result.bytes,
    }))
    .map(|output| {
        output
            .human_stdout(format!(
                "Downloaded: {} ({} bytes)\n",
                result.path.display(),
                result.bytes
            ))
            .human_stderr(format!("Session: {}\n", session.id))
    })
}

fn direct_download_request(
    config: &bowser::BrowserConfig,
    args: &DownloadArgs,
) -> bowser::Result<Option<(String, PathBuf)>> {
    if args.link.is_some() {
        if config.session.id.is_none() {
            return Err(bowser::Error::SessionNotFound {
                session_id: "<missing --session>".to_string(),
            });
        }
        if args.output.is_none() {
            return Err(bowser::Error::config(
                "download --link requires --output <PATH>",
            ));
        }
        return Ok(None);
    }
    let url = args.url.as_deref().ok_or_else(|| {
        bowser::Error::config(
            "download requires <URL> <PATH> or --link <ELEMENT_ID> --output <PATH>",
        )
    })?;
    let path = args
        .path
        .clone()
        .or_else(|| args.output.clone())
        .ok_or_else(|| {
            bowser::Error::config(
                "download requires <URL> <PATH> or --link <ELEMENT_ID> --output <PATH>",
            )
        })?;
    Ok(Some((normalize_navigation_target(url), path)))
}

async fn link_download_request(
    args: &DownloadArgs,
    page: &dyn PageEngine,
) -> bowser::Result<(String, PathBuf)> {
    let element_id = args.link.ok_or_else(|| {
        bowser::Error::config(
            "download requires <URL> <PATH> or --link <ELEMENT_ID> --output <PATH>",
        )
    })?;
    let path = args
        .output
        .clone()
        .ok_or_else(|| bowser::Error::config("download --link requires --output <PATH>"))?;
    match page.metadata(element_id).await? {
        MetadataRecord::Link { href, .. } => Ok((href, path)),
        _ => Err(bowser::Error::Download {
            reason: format!("element {element_id} is not a link"),
        }),
    }
}
