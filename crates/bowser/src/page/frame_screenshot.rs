//! Screenshot helpers for target-backed iframe documents.

use base64::{Engine, engine::general_purpose::STANDARD};
use chromiumoxide::cdp::browser_protocol::page::FrameId;
use serde::Deserialize;

use crate::error::{Error, Result};

use super::iframe_target::RemoteFrameTarget;
use super::interaction_scripts::{image_png_expression, viewport_bounds_expression};
use super::types::{Bounds, LiveElementContext, LivePage};

impl LivePage {
    pub(super) async fn screenshot_frame_element(
        &self,
        context: LiveElementContext,
        requested_element_id: u32,
    ) -> Result<Vec<u8>> {
        match context {
            LiveElementContext::Remote {
                frame_id,
                websocket_url,
                element_id,
                owner_element_id,
            } => {
                let mut remote = RemoteFrameTarget::connect(&websocket_url).await?;
                match remote_image_png(&mut remote, &frame_id, element_id, requested_element_id)
                    .await
                {
                    Ok(bytes) => Ok(bytes),
                    Err(Error::ElementNotFound { .. }) => Err(Error::ElementNotFound {
                        element_id: requested_element_id,
                    }),
                    Err(error) => {
                        let Some(owner_element_id) = owner_element_id else {
                            return Err(error);
                        };
                        self.screenshot_frame_clip(
                            &mut remote,
                            &frame_id,
                            element_id,
                            owner_element_id,
                            requested_element_id,
                        )
                        .await
                    }
                }
            }
        }
    }

    async fn screenshot_frame_clip(
        &self,
        remote: &mut RemoteFrameTarget,
        frame_id: &FrameId,
        mapped_element_id: u32,
        owner_element_id: u32,
        requested_element_id: u32,
    ) -> Result<Vec<u8>> {
        let local = remote
            .evaluate_frame_value::<Option<Bounds>>(
                frame_id,
                viewport_bounds_expression(mapped_element_id),
            )
            .await?
            .ok_or(Error::ElementNotFound {
                element_id: requested_element_id,
            })?;
        let owner = self.element_bounds(owner_element_id).await?;
        self.screenshot_clip(Bounds {
            x: owner.x + local.x,
            y: owner.y + local.y,
            width: local.width,
            height: local.height,
        })
        .await
    }
}

async fn remote_image_png(
    remote: &mut RemoteFrameTarget,
    frame_id: &FrameId,
    mapped_element_id: u32,
    requested_element_id: u32,
) -> Result<Vec<u8>> {
    let response = remote
        .evaluate_frame_value::<CanvasImageResponse>(
            frame_id,
            image_png_expression(mapped_element_id),
        )
        .await?;
    if response.ok {
        return decode_png_data_url(response, requested_element_id);
    }
    match response.kind.as_deref() {
        Some("not_found") => Err(Error::ElementNotFound {
            element_id: requested_element_id,
        }),
        Some("not_image") => Err(Error::DescribeNotImage {
            element_id: requested_element_id,
        }),
        Some(kind) => Err(Error::Screenshot {
            reason: format!("iframe image capture failed: {kind}"),
        }),
        None => Err(Error::Screenshot {
            reason: "iframe image capture failed".to_string(),
        }),
    }
}

fn decode_png_data_url(
    response: CanvasImageResponse,
    requested_element_id: u32,
) -> Result<Vec<u8>> {
    let Some(data_url) = response.data_url else {
        return Err(Error::ElementNotFound {
            element_id: requested_element_id,
        });
    };
    let Some(data) = data_url.strip_prefix("data:image/png;base64,") else {
        return Err(Error::Screenshot {
            reason: "iframe image capture returned non-PNG data".to_string(),
        });
    };
    match STANDARD.decode(data) {
        Ok(bytes) => Ok(bytes),
        Err(err) => Err(Error::Screenshot {
            reason: err.to_string(),
        }),
    }
}

#[derive(Deserialize)]
struct CanvasImageResponse {
    ok: bool,
    kind: Option<String>,
    data_url: Option<String>,
}
