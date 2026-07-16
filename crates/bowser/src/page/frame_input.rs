//! Form and keyboard helpers for target-backed iframe documents.

use chromiumoxide::cdp::browser_protocol::page::FrameId;

use crate::error::{Error, Result};
use crate::keyboard::{resolve_pressed_keys, resolve_text_character};

use super::evaluate::typing_seed_for_text;
use super::evaluate::{finish_action_response, prepare_text_entry_expression, typing_delay_ms};
use super::form_scripts::{
    select_option_expression, submit_dom_expression, submit_plan_expression,
};
use super::iframe_target::RemoteFrameTarget;
use super::types::{ActionResponse, LiveElementContext, LivePage, SubmitPlan};

impl LivePage {
    pub(super) async fn type_frame_text(
        &self,
        context: LiveElementContext,
        requested_element_id: u32,
        text: &str,
    ) -> Result<()> {
        let mut remote = remote_for_context(&context).await?;
        let (frame_id, mapped_element_id) = frame_target(context);
        remote_focus_text_entry(
            &mut remote,
            &frame_id,
            mapped_element_id,
            requested_element_id,
            true,
        )
        .await?;
        let characters = text
            .chars()
            .map(|ch| resolve_text_character(&ch.to_string()))
            .collect::<Result<Vec<_>>>()?;
        let mut typing_seed = typing_seed_for_text(requested_element_id, text);
        for (index, key) in characters.iter().enumerate() {
            remote
                .press_resolved_keys(std::slice::from_ref(key))
                .await?;
            if index + 1 < characters.len() {
                tokio::time::sleep(std::time::Duration::from_millis(typing_delay_ms(
                    &mut typing_seed,
                )))
                .await;
            }
        }
        Ok(())
    }

    pub(super) async fn clear_frame_text(
        &self,
        context: LiveElementContext,
        requested_element_id: u32,
    ) -> Result<()> {
        let mut remote = remote_for_context(&context).await?;
        let (frame_id, mapped_element_id) = frame_target(context);
        remote_focus_text_entry(
            &mut remote,
            &frame_id,
            mapped_element_id,
            requested_element_id,
            true,
        )
        .await?;
        let backspace = resolve_pressed_keys(&[String::from("backspace")])?;
        remote.press_resolved_keys(&backspace).await
    }

    pub(super) async fn select_frame_option(
        &self,
        context: LiveElementContext,
        requested_element_id: u32,
        value: &str,
    ) -> Result<()> {
        let value = serde_json::to_string(value).map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })?;
        let mut remote = remote_for_context(&context).await?;
        let (frame_id, mapped_element_id) = frame_target(context);
        remote_run_action(
            &mut remote,
            &frame_id,
            select_option_expression(mapped_element_id, &value),
            requested_element_id,
        )
        .await
    }

    pub(super) async fn submit_frame_element(
        &self,
        context: LiveElementContext,
        requested_element_id: u32,
    ) -> Result<()> {
        let mut remote = remote_for_context(&context).await?;
        let (frame_id, mapped_element_id) = frame_target(context.clone());
        let plan = remote
            .evaluate_frame_value::<SubmitPlan>(
                &frame_id,
                submit_plan_expression(mapped_element_id),
            )
            .await?;
        if !plan.ok {
            return match plan.kind.as_deref() {
                Some("not_interactable") => Err(Error::ElementNotInteractable {
                    element_id: requested_element_id,
                }),
                _ => Err(Error::ElementNotFound {
                    element_id: requested_element_id,
                }),
            };
        }
        match plan.strategy.as_deref() {
            Some("click_self") => {
                self.click_frame_element(context, requested_element_id)
                    .await
            }
            Some("enter") => {
                let enter = resolve_pressed_keys(&[String::from("enter")])?;
                remote.press_resolved_keys(&enter).await
            }
            _ => {
                remote_run_action(
                    &mut remote,
                    &frame_id,
                    submit_dom_expression(mapped_element_id),
                    requested_element_id,
                )
                .await
            }
        }
    }
}

async fn remote_for_context(context: &LiveElementContext) -> Result<RemoteFrameTarget> {
    match context {
        LiveElementContext::Remote { websocket_url, .. } => {
            RemoteFrameTarget::connect(websocket_url).await
        }
    }
}

fn frame_target(context: LiveElementContext) -> (FrameId, u32) {
    match context {
        LiveElementContext::Remote {
            frame_id,
            element_id,
            ..
        } => (frame_id, element_id),
    }
}

async fn remote_focus_text_entry(
    remote: &mut RemoteFrameTarget,
    frame_id: &FrameId,
    mapped_element_id: u32,
    requested_element_id: u32,
    select_existing: bool,
) -> Result<()> {
    remote_run_action(
        remote,
        frame_id,
        prepare_text_entry_expression(mapped_element_id, select_existing),
        requested_element_id,
    )
    .await
}

async fn remote_run_action(
    remote: &mut RemoteFrameTarget,
    frame_id: &FrameId,
    expression: String,
    requested_element_id: u32,
) -> Result<()> {
    let response = remote
        .evaluate_frame_value::<ActionResponse>(frame_id, expression)
        .await?;
    finish_action_response(response, requested_element_id)
}
