//! Metadata helpers for target-backed iframe documents.

use crate::error::Result;
use crate::model::ElementBounds;

use super::iframe_target::RemoteFrameTarget;
use super::interaction_scripts::runtime_state_expression;
use super::types::{ElementRuntimeState, LiveElementContext, LivePage};

impl LivePage {
    pub(super) async fn frame_element_runtime_state(
        &self,
        context: LiveElementContext,
    ) -> Result<ElementRuntimeState> {
        match context {
            LiveElementContext::Remote {
                frame_id,
                websocket_url,
                element_id,
                owner_element_id,
            } => {
                let mut remote = RemoteFrameTarget::connect(&websocket_url).await?;
                let mut state = remote
                    .evaluate_frame_value::<ElementRuntimeState>(
                        &frame_id,
                        runtime_state_expression(element_id, false),
                    )
                    .await?;
                if let (Some(owner_element_id), Some(bounds)) = (owner_element_id, state.bounds) {
                    let owner = self.element_bounds(owner_element_id).await?;
                    state.bounds = Some(ElementBounds {
                        top_left: (owner.x + bounds.top_left.0, owner.y + bounds.top_left.1),
                        bottom_right: (
                            owner.x + bounds.bottom_right.0,
                            owner.y + bounds.bottom_right.1,
                        ),
                    });
                }
                Ok(state)
            }
        }
    }
}
