//! Raw-DevTools iframe-target capture helpers.

use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::page::FrameId;

use crate::capture::{FrameOwnerKey, build_capture_script};
use crate::error::Result;

use super::frame_capture_support::{CaptureRegistry, IframeTargetEndpoint, PendingTarget};
use super::iframe_target::{FrameTreeSnapshot, RemoteFrameTarget};
use super::types::LivePage;

impl LivePage {
    pub(super) async fn capture_remote_frames(
        &self,
        remote: &mut RemoteFrameTarget,
        snapshot: &FrameTreeSnapshot,
        registry: &mut CaptureRegistry,
        required: bool,
    ) -> Result<()> {
        for (index, frame_id) in snapshot.ordered_frames.iter().enumerate() {
            match remote
                .evaluate_frame_value(
                    frame_id,
                    build_capture_script(self.config.output.include_hidden),
                )
                .await
            {
                Ok(capture) => {
                    registry.captures.insert(frame_id.clone(), capture);
                }
                Err(error) if required && index == 0 => return Err(error),
                Err(_) => {}
            }
        }
        Ok(())
    }

    pub(super) async fn register_remote_children(
        &self,
        remote: &mut RemoteFrameTarget,
        snapshot: &FrameTreeSnapshot,
        registry: &mut CaptureRegistry,
    ) {
        for frame_id in snapshot.ordered_frames.iter().skip(1) {
            let Some(parent_frame_id) = snapshot.parents.get(frame_id) else {
                continue;
            };
            if let Ok(Some(owner_temp_id)) = remote.owner_temp_id(parent_frame_id, frame_id).await {
                registry.child_frames_by_owner.insert(
                    FrameOwnerKey {
                        parent_frame_id: parent_frame_id.clone(),
                        owner_temp_id,
                    },
                    frame_id.clone(),
                );
            }
        }
    }

    pub(super) async fn enqueue_remote_iframe_targets(
        &self,
        remote: &mut RemoteFrameTarget,
        snapshot: &FrameTreeSnapshot,
        iframe_targets: &HashMap<FrameId, Vec<IframeTargetEndpoint>>,
        pending: &mut Vec<PendingTarget>,
        registry: &mut CaptureRegistry,
    ) {
        for parent_frame_id in &snapshot.ordered_frames {
            let Some(targets) = iframe_targets.get(parent_frame_id) else {
                continue;
            };
            for target in targets {
                if registry
                    .seen_targets
                    .contains(target.child_frame_id.as_ref())
                {
                    continue;
                }
                let owner_temp_id = match remote
                    .owner_temp_id(parent_frame_id, &target.child_frame_id)
                    .await
                {
                    Ok(Some(owner_temp_id)) => owner_temp_id,
                    _ => continue,
                };
                registry.child_frames_by_owner.insert(
                    FrameOwnerKey {
                        parent_frame_id: parent_frame_id.clone(),
                        owner_temp_id,
                    },
                    target.child_frame_id.clone(),
                );
                pending.push(PendingTarget::Remote {
                    target_id: target.child_frame_id.as_ref().to_string(),
                    websocket_url: target.websocket_url.clone(),
                    required: false,
                });
            }
        }
    }
}
