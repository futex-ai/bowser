//! Scroll helpers for target-backed iframe documents.

use chromiumoxide::cdp::browser_protocol::page::FrameId;

use crate::error::Result;

use super::evaluate::finish_action_response;
use super::form_scripts::scroll_to_element_expression;
use super::iframe_target::RemoteFrameTarget;
use super::types::{ActionResponse, LiveElementContext, LivePage};

impl LivePage {
    pub(super) async fn scroll_frame_element(
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
                if let Some(owner_element_id) = owner_element_id {
                    let _ = self.element_bounds(owner_element_id).await?;
                }
                let mut remote = RemoteFrameTarget::connect(&websocket_url).await?;
                remote_scroll_to_element(&mut remote, &frame_id, element_id, requested_element_id)
                    .await
            }
        }
    }
}

async fn remote_scroll_to_element(
    remote: &mut RemoteFrameTarget,
    frame_id: &FrameId,
    mapped_element_id: u32,
    requested_element_id: u32,
) -> Result<()> {
    let response = remote
        .evaluate_frame_value::<ActionResponse>(
            frame_id,
            scroll_to_element_expression(mapped_element_id),
        )
        .await?;
    finish_action_response(response, requested_element_id)
}
