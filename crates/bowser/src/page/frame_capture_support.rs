//! Shared frame-capture state and session-browser helpers.

use std::collections::{HashMap, HashSet};

use chromiumoxide::browser::Browser as ChromiumBrowser;
use chromiumoxide::cdp::browser_protocol::page::FrameId;
use futures::StreamExt;
use serde::Deserialize;

use crate::error::{Error, Result};

use super::types::LivePage;

pub(super) enum PendingTarget {
    Local {
        page: chromiumoxide::Page,
        target_id: String,
        required: bool,
    },
    Remote {
        target_id: String,
        websocket_url: String,
        required: bool,
    },
}

pub(super) struct IframeTargetEndpoint {
    pub(super) child_frame_id: FrameId,
    pub(super) websocket_url: String,
}

#[derive(Default)]
pub(super) struct CaptureRegistry {
    pub(super) captures: HashMap<FrameId, crate::capture::RawCaptureEnvelope>,
    pub(super) child_frames_by_owner: HashMap<crate::capture::FrameOwnerKey, FrameId>,
    pub(super) frame_pages: HashMap<FrameId, chromiumoxide::Page>,
    pub(super) remote_frame_websocket_urls: HashMap<FrameId, String>,
    pub(super) seen_targets: HashSet<String>,
}

#[derive(Deserialize)]
struct TargetPageDescriptor {
    id: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    websocket_debugger_url: Option<String>,
}

impl LivePage {
    pub(super) async fn iframe_targets_by_parent_frame(
        &self,
        browser: &mut ChromiumBrowser,
        http_url: &str,
    ) -> Result<HashMap<FrameId, Vec<IframeTargetEndpoint>>> {
        let targets = browser
            .fetch_targets()
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?;
        let pages = reqwest::get(format!("{http_url}/json/list"))
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?
            .json::<Vec<TargetPageDescriptor>>()
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?;
        let websocket_urls: HashMap<String, String> = pages
            .into_iter()
            .filter_map(|page| page.websocket_debugger_url.map(|url| (page.id, url)))
            .collect();
        let mut by_parent = HashMap::new();
        for target in targets {
            if target.r#type != "iframe" {
                continue;
            }
            let (Some(parent_frame_id), Some(websocket_url)) = (
                target.parent_frame_id.clone(),
                websocket_urls.get(target.target_id.as_ref()).cloned(),
            ) else {
                continue;
            };
            by_parent
                .entry(parent_frame_id)
                .or_insert_with(Vec::new)
                .push(IframeTargetEndpoint {
                    child_frame_id: FrameId::from(target.target_id.as_ref().to_string()),
                    websocket_url,
                });
        }
        Ok(by_parent)
    }

    pub(super) async fn connect_session_browser(
        &self,
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
}
