//! Frame-aware capture orchestration.

use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::page::FrameId;

use crate::capture::{FinalizedCapture, finalize_capture};
use crate::error::{Error, Result};
use crate::model::PageCapture;

use super::frame_capture_support::{CaptureRegistry, PendingTarget};
use super::iframe_target::RemoteFrameTarget;
use super::types::{LiveElementContext, LivePage};

impl LivePage {
    pub(super) async fn capture_frame_tree(&self) -> Result<PageCapture> {
        let metadata = self.store.load(&self.session_id).await?;
        let (mut browser, handler_task) = self.connect_session_browser(&metadata.http_url).await?;
        let result = async {
            let iframe_targets = self
                .iframe_targets_by_parent_frame(&mut browser, &metadata.http_url)
                .await?;
            let mut registry = CaptureRegistry::default();
            let mut pending = vec![PendingTarget::Local {
                page: self.page.clone(),
                target_id: self.page.target_id().as_ref().to_string(),
                required: true,
            }];
            let mut root_frame_id = None;

            while let Some(target) = pending.pop() {
                match target {
                    PendingTarget::Local {
                        page,
                        target_id,
                        required,
                    } => {
                        if !registry.seen_targets.insert(target_id) {
                            continue;
                        }
                        let ordered_frames = match self.ordered_frames_for_page(&page).await {
                            Ok(ordered_frames) => ordered_frames,
                            Err(error) if required => return Err(error),
                            Err(_) => continue,
                        };
                        if root_frame_id.is_none() {
                            root_frame_id = ordered_frames.first().cloned();
                        }
                        self.capture_local_frames(&page, &ordered_frames, &mut registry, required)
                            .await?;
                        self.register_same_target_children(&page, &ordered_frames, &mut registry)
                            .await;
                        self.enqueue_local_iframe_targets(
                            &page,
                            &ordered_frames,
                            &iframe_targets,
                            &mut pending,
                            &mut registry,
                        )
                        .await;
                    }
                    PendingTarget::Remote {
                        target_id,
                        websocket_url,
                        required,
                    } => {
                        if !registry.seen_targets.insert(target_id) {
                            continue;
                        }
                        let mut remote = match RemoteFrameTarget::connect(&websocket_url).await {
                            Ok(remote) => remote,
                            Err(error) if required => return Err(error),
                            Err(_) => continue,
                        };
                        let snapshot = match remote.ordered_frames().await {
                            Ok(snapshot) => snapshot,
                            Err(error) if required => return Err(error),
                            Err(_) => continue,
                        };
                        if root_frame_id.is_none() {
                            root_frame_id = snapshot.ordered_frames.first().cloned();
                        }
                        for frame_id in &snapshot.ordered_frames {
                            registry
                                .remote_frame_websocket_urls
                                .insert(frame_id.clone(), websocket_url.clone());
                        }
                        self.capture_remote_frames(&mut remote, &snapshot, &mut registry, required)
                            .await?;
                        self.register_remote_children(&mut remote, &snapshot, &mut registry)
                            .await;
                        self.enqueue_remote_iframe_targets(
                            &mut remote,
                            &snapshot,
                            &iframe_targets,
                            &mut pending,
                            &mut registry,
                        )
                        .await;
                    }
                }
            }

            let finalized = finalize_capture(
                &root_frame_id.ok_or(Error::CaptureParse)?,
                &registry.captures,
                &registry.child_frames_by_owner,
            )?;
            self.rebuild_live_id_maps(&registry.frame_pages, &finalized)
                .await?;
            self.store_live_element_contexts(live_element_contexts(&registry, &finalized))
                .await;
            Ok(finalized.capture)
        }
        .await;
        handler_task.abort();
        result
    }
}

fn live_element_contexts(
    registry: &CaptureRegistry,
    finalized: &FinalizedCapture,
) -> HashMap<u32, LiveElementContext> {
    let mut contexts = HashMap::new();
    for (frame_id, mappings) in &finalized.id_mappings {
        let Some(websocket_url) = registry.remote_frame_websocket_urls.get(frame_id) else {
            continue;
        };
        for (temp_id, final_id) in mappings {
            contexts.insert(
                *final_id,
                LiveElementContext::Remote {
                    frame_id: frame_id.clone(),
                    websocket_url: websocket_url.clone(),
                    element_id: *temp_id,
                    owner_element_id: owner_element_id(registry, finalized, frame_id),
                },
            );
        }
    }
    contexts
}

fn owner_element_id(
    registry: &CaptureRegistry,
    finalized: &FinalizedCapture,
    child_frame_id: &FrameId,
) -> Option<u32> {
    for (owner, frame_id) in &registry.child_frames_by_owner {
        if frame_id != child_frame_id {
            continue;
        }
        return finalized
            .id_mappings
            .get(&owner.parent_frame_id)?
            .iter()
            .find_map(|(temp_id, final_id)| {
                (*temp_id == owner.owner_temp_id).then_some(*final_id)
            });
    }
    None
}
