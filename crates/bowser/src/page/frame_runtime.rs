//! Runtime interaction helpers for target-backed iframe documents.

use std::collections::HashMap;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::page::FrameId;

use crate::error::{Error, Result};
use crate::mouse::{
    CursorPosition, ViewportArea, click_hold_ms, initial_cursor_position, movement_steps,
};

use super::evaluate::finish_action_response;
use super::iframe_target::RemoteFrameTarget;
use super::interaction_scripts::{
    focus_after_pointer_click_expression, interaction_target_expression,
    synthetic_click_expression, viewport_bounds_expression,
};
use super::types::{
    ActionResponse, Bounds, InteractionTarget, LiveElementContext, LivePage, ViewportMetrics,
};

impl LivePage {
    pub(super) async fn store_live_element_contexts(
        &self,
        contexts: HashMap<u32, LiveElementContext>,
    ) {
        self.state.lock().await.live_element_contexts = contexts;
    }

    pub(super) async fn live_element_context(&self, element_id: u32) -> Option<LiveElementContext> {
        self.state
            .lock()
            .await
            .live_element_contexts
            .get(&element_id)
            .cloned()
    }

    pub(super) async fn click_frame_element(
        &self,
        context: LiveElementContext,
        requested_element_id: u32,
    ) -> Result<()> {
        match context {
            LiveElementContext::Remote {
                frame_id,
                websocket_url,
                element_id,
                owner_element_id,
            } => {
                let mut remote = RemoteFrameTarget::connect(&websocket_url).await?;
                let target = remote
                    .evaluate_frame_value::<Option<InteractionTarget>>(
                        &frame_id,
                        interaction_target_expression(element_id),
                    )
                    .await?
                    .ok_or(Error::ElementNotFound {
                        element_id: requested_element_id,
                    })?;
                if target.visible && target.pointer_enabled {
                    match self
                        .pointer_click_remote_frame_target(
                            &mut remote,
                            requested_element_id,
                            target,
                            owner_element_id,
                        )
                        .await
                    {
                        Ok(()) => {
                            remote_focus_after_pointer_click(
                                &mut remote,
                                &frame_id,
                                element_id,
                                target,
                            )
                            .await;
                            Ok(())
                        }
                        Err(_) => {
                            remote_synthetic_click(
                                &mut remote,
                                &frame_id,
                                element_id,
                                requested_element_id,
                            )
                            .await
                        }
                    }
                } else {
                    remote_synthetic_click(&mut remote, &frame_id, element_id, requested_element_id)
                        .await
                }
            }
        }
    }

    async fn remote_pointer_click(
        &self,
        remote: &mut RemoteFrameTarget,
        element_id: u32,
        target: InteractionTarget,
    ) -> Result<()> {
        let viewport = ViewportArea {
            width: target.viewport_width.max(1.0),
            height: target.viewport_height.max(1.0),
        };
        let target_position = CursorPosition {
            x: target.x,
            y: target.y,
        };
        let prior_position = self.state.lock().await.cursor_position;
        let start_position = prior_position
            .filter(|position| {
                position.x >= 0.0
                    && position.y >= 0.0
                    && position.x < viewport.width
                    && position.y < viewport.height
            })
            .unwrap_or_else(|| initial_cursor_position(viewport, element_id, target_position));
        let entering_viewport = prior_position.is_none();
        let route = movement_steps(
            start_position,
            target_position,
            viewport,
            element_id,
            entering_viewport,
        );
        for step in route {
            remote
                .dispatch_mouse_event("mouseMoved", step.position.x, step.position.y, false)
                .await?;
            tokio::time::sleep(Duration::from_millis(step.pause_ms)).await;
        }
        remote
            .dispatch_mouse_event("mousePressed", target.x, target.y, true)
            .await?;
        tokio::time::sleep(Duration::from_millis(click_hold_ms(
            element_id,
            target_position,
            viewport,
        )))
        .await;
        remote
            .dispatch_mouse_event("mouseReleased", target.x, target.y, true)
            .await?;
        self.state.lock().await.cursor_position = Some(target_position);
        Ok(())
    }

    async fn pointer_click_remote_frame_target(
        &self,
        remote: &mut RemoteFrameTarget,
        element_id: u32,
        target: InteractionTarget,
        owner_element_id: Option<u32>,
    ) -> Result<()> {
        if let Some(owner_element_id) = owner_element_id {
            let owner = self.element_viewport_bounds(owner_element_id).await?;
            let viewport = self.viewport_metrics().await?;
            let translated = InteractionTarget {
                x: owner.x + target.x,
                y: owner.y + target.y,
                viewport_width: viewport.width,
                viewport_height: viewport.height,
                visible: true,
                pointer_enabled: true,
                enabled: target.enabled,
                focus_after_click: target.focus_after_click,
            };
            return self.pointer_click(element_id, translated).await;
        }
        self.remote_pointer_click(remote, element_id, target).await
    }

    async fn element_viewport_bounds(&self, element_id: u32) -> Result<Bounds> {
        self.evaluate_bowser_value::<Option<Bounds>>(viewport_bounds_expression(element_id))
            .await?
            .ok_or(Error::ElementNotFound { element_id })
    }

    async fn viewport_metrics(&self) -> Result<ViewportMetrics> {
        self.evaluate_value(
            r#"(() => ({
  width: Math.max(window.innerWidth || 0, 1),
  height: Math.max(window.innerHeight || 0, 1)
}))()"#
                .to_string(),
        )
        .await
    }
}

async fn remote_synthetic_click(
    remote: &mut RemoteFrameTarget,
    frame_id: &FrameId,
    mapped_element_id: u32,
    requested_element_id: u32,
) -> Result<()> {
    let response = remote
        .evaluate_frame_value::<ActionResponse>(
            frame_id,
            synthetic_click_expression(mapped_element_id),
        )
        .await?;
    finish_action_response(response, requested_element_id)
}

async fn remote_focus_after_pointer_click(
    remote: &mut RemoteFrameTarget,
    frame_id: &FrameId,
    mapped_element_id: u32,
    target: InteractionTarget,
) {
    if target.focus_after_click {
        let _ = remote
            .evaluate_frame_value::<ActionResponse>(
                frame_id,
                focus_after_pointer_click_expression(mapped_element_id),
            )
            .await;
    }
}
