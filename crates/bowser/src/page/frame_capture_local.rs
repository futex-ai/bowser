//! Chromiumoxide-backed frame capture within the current page target.

use std::collections::{HashMap, HashSet};

use chromiumoxide::cdp::browser_protocol::dom::{GetFrameOwnerParams, ResolveNodeParams};
use chromiumoxide::cdp::browser_protocol::page::FrameId;
use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use serde::Deserialize;

use crate::capture::{FinalizedCapture, FrameOwnerKey, build_capture_script};
use crate::error::{Error, Result};

use super::frame_capture_support::{CaptureRegistry, IframeTargetEndpoint, PendingTarget};
use super::isolated_world::{create_bowser_world, evaluate_bowser_world_value_for_page};
use super::types::LivePage;

impl LivePage {
    pub(super) async fn capture_local_frames(
        &self,
        page: &chromiumoxide::Page,
        ordered_frames: &[FrameId],
        registry: &mut CaptureRegistry,
        required: bool,
    ) -> Result<()> {
        for (index, frame_id) in ordered_frames.iter().enumerate() {
            match self
                .evaluate_frame_value_for_page(
                    page,
                    frame_id,
                    build_capture_script(self.config.output.include_hidden),
                )
                .await
            {
                Ok(capture) => {
                    registry.captures.insert(frame_id.clone(), capture);
                    registry.frame_pages.insert(frame_id.clone(), page.clone());
                }
                Err(error) if required && index == 0 => return Err(error),
                Err(_) => {}
            }
        }
        Ok(())
    }

    pub(super) async fn register_same_target_children(
        &self,
        page: &chromiumoxide::Page,
        ordered_frames: &[FrameId],
        registry: &mut CaptureRegistry,
    ) {
        for frame_id in ordered_frames.iter().skip(1) {
            let parent_frame_id = match self.frame_parent_for_page(page, frame_id).await {
                Ok(Some(parent_frame_id)) => parent_frame_id,
                _ => continue,
            };
            if let Ok(Some(owner_temp_id)) =
                self.owner_temp_id(page, &parent_frame_id, frame_id).await
            {
                registry.child_frames_by_owner.insert(
                    FrameOwnerKey {
                        parent_frame_id,
                        owner_temp_id,
                    },
                    frame_id.clone(),
                );
            }
        }
    }

    pub(super) async fn enqueue_local_iframe_targets(
        &self,
        page: &chromiumoxide::Page,
        ordered_frames: &[FrameId],
        iframe_targets: &HashMap<FrameId, Vec<IframeTargetEndpoint>>,
        pending: &mut Vec<PendingTarget>,
        registry: &mut CaptureRegistry,
    ) {
        for parent_frame_id in ordered_frames {
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
                let owner_temp_id = match self
                    .owner_temp_id(page, parent_frame_id, &target.child_frame_id)
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

    pub(super) async fn ordered_frames_for_page(
        &self,
        page: &chromiumoxide::Page,
    ) -> Result<Vec<FrameId>> {
        let main_frame_id = page
            .mainframe()
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?
            .ok_or(Error::JsEvaluation {
                reason: "main frame unavailable".to_string(),
            })?;
        let mut seen = HashSet::from([main_frame_id.clone()]);
        let mut frames = vec![(0_usize, main_frame_id)];
        for frame_id in page.frames().await.map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })? {
            if seen.insert(frame_id.clone()) {
                frames.push((self.frame_depth_for_page(page, &frame_id).await?, frame_id));
            }
        }
        frames.sort_by_key(|(depth, _)| *depth);
        Ok(frames.into_iter().map(|(_, frame_id)| frame_id).collect())
    }

    async fn frame_depth_for_page(
        &self,
        page: &chromiumoxide::Page,
        frame_id: &FrameId,
    ) -> Result<usize> {
        let mut depth = 0_usize;
        let mut current = frame_id.clone();
        while let Some(parent) = self.frame_parent_for_page(page, &current).await? {
            depth += 1;
            current = parent;
        }
        Ok(depth)
    }

    async fn frame_parent_for_page(
        &self,
        page: &chromiumoxide::Page,
        frame_id: &FrameId,
    ) -> Result<Option<FrameId>> {
        page.frame_parent(frame_id.clone())
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })
    }

    async fn owner_temp_id(
        &self,
        page: &chromiumoxide::Page,
        owner_frame_id: &FrameId,
        child_frame_id: &FrameId,
    ) -> Result<Option<u32>> {
        let owner = page
            .execute(GetFrameOwnerParams::new(child_frame_id.clone()))
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?;
        let context_id = create_bowser_world(page, owner_frame_id).await?;
        let resolved = page
            .execute(
                ResolveNodeParams::builder()
                    .backend_node_id(owner.backend_node_id)
                    .execution_context_id(context_id)
                    .build(),
            )
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?;
        let Some(object_id) = resolved.result.object.object_id else {
            return Ok(None);
        };
        let result = page
            .execute(
                CallFunctionOnParams::builder()
                    .function_declaration("function() { return this.__bowserCaptureId ?? null; }")
                    .object_id(object_id)
                    .return_by_value(true)
                    .build()
                    .map_err(|reason| Error::JsEvaluation { reason })?,
            )
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?;
        Ok(result
            .result
            .result
            .value
            .and_then(|value| value.as_u64())
            .and_then(|value| u32::try_from(value).ok()))
    }

    pub(super) async fn evaluate_frame_value_for_page<T: for<'de> Deserialize<'de>>(
        &self,
        page: &chromiumoxide::Page,
        frame_id: &FrameId,
        expression: String,
    ) -> Result<T> {
        let main_frame = page.mainframe().await.map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })?;
        if main_frame.as_ref() == Some(frame_id) {
            return evaluate_bowser_world_value_for_page(page, frame_id, expression).await;
        }
        evaluate_bowser_world_value_for_page(page, frame_id, expression).await
    }

    pub(super) async fn rebuild_live_id_maps(
        &self,
        frame_pages: &HashMap<FrameId, chromiumoxide::Page>,
        finalized: &FinalizedCapture,
    ) -> Result<()> {
        for (frame_id, mappings) in &finalized.id_mappings {
            let Some(page) = frame_pages.get(frame_id) else {
                continue;
            };
            let mappings_json =
                serde_json::to_string(mappings).map_err(|err| Error::JsEvaluation {
                    reason: err.to_string(),
                })?;
            let script = format!(
                "(() => {{ const current = window.__bowserElements instanceof Map ? window.__bowserElements : new Map(); const rewritten = new Map(); for (const [tempId, finalId] of {mappings_json}) {{ const el = current.get(tempId); if (el) {{ rewritten.set(finalId, el); }} }} window.__bowserElements = rewritten; return true; }})()"
            );
            let _ = evaluate_bowser_world_value_for_page::<bool>(page, frame_id, script).await?;
        }
        Ok(())
    }
}
