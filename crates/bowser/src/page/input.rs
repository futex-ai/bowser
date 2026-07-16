//! Keyboard, form, and submit helpers.

use super::evaluate::{prepare_text_entry_expression, typing_delay_ms, typing_seed_for_text};
use super::form_scripts::{
    select_option_expression, submit_dom_expression, submit_plan_expression,
};
use super::interaction_scripts::focus_after_pointer_click_expression;
use super::types::{InteractionTarget, LiveElementContext, LivePage, SubmitPlan};
use crate::error::{Error, Result};
use crate::keyboard::{resolve_pressed_keys, resolve_text_character};
use crate::stealth_features::StealthFeatures;

impl LivePage {
    pub(super) async fn click_impl(&self, element_id: u32) -> Result<()> {
        self.ensure_live_ids(element_id).await?;
        if let Some(context @ LiveElementContext::Remote { .. }) =
            self.live_element_context(element_id).await
        {
            return self.click_frame_element(context, element_id).await;
        }
        let features = StealthFeatures::from_config(&self.config);
        if features.enter_submit()
            && let Some(input_id) = self.search_submit_input_id(element_id).await
        {
            if !self.backend_focus(input_id).await? {
                self.focus_text_entry(input_id, false).await?;
            }
            let enter = resolve_pressed_keys(&[String::from("enter")])?;
            self.press_resolved_keys(&enter).await?;
            self.wait_after_possible_navigation(std::time::Duration::from_secs(5))
                .await;
            return Ok(());
        }
        if features.enter_submit()
            && self.enter_activatable_element(element_id).await
            && self.backend_focus(element_id).await?
        {
            let enter = resolve_pressed_keys(&[String::from("enter")])?;
            self.press_resolved_keys(&enter).await?;
            self.wait_after_possible_navigation(std::time::Duration::from_millis(800))
                .await;
            return Ok(());
        }
        if features.backend_pointer_target()
            && let Some(target) = self.backend_interaction_target(element_id).await?
        {
            self.pointer_click(element_id, target).await?;
            self.wait_after_possible_navigation(std::time::Duration::from_millis(800))
                .await;
            return Ok(());
        }
        let target = self.interaction_target(element_id).await?;
        if target.visible && target.pointer_enabled {
            match self.pointer_click(element_id, target).await {
                Ok(()) => {
                    self.focus_after_pointer_click(element_id, target).await;
                    Ok(())
                }
                Err(_) => self.synthetic_click(element_id).await,
            }
        } else {
            self.synthetic_click(element_id).await
        }
    }

    pub(super) async fn press_keys_impl(&self, keys: &[String]) -> Result<()> {
        self.prepare().await?;
        let resolved = resolve_pressed_keys(keys)?;
        if resolved.is_empty() {
            return Err(Error::KeyPress {
                reason: "no keys provided".to_string(),
            });
        }
        self.press_resolved_keys(&resolved).await
    }

    pub(super) async fn type_text_impl(&self, element_id: u32, text: &str) -> Result<()> {
        self.ensure_live_ids(element_id).await?;
        if let Some(context @ LiveElementContext::Remote { .. }) =
            self.live_element_context(element_id).await
        {
            return self.type_frame_text(context, element_id, text).await;
        }
        let characters = text
            .chars()
            .map(|ch| resolve_text_character(&ch.to_string()))
            .collect::<Result<Vec<_>>>()?;
        let mut typing_seed = typing_seed_for_text(element_id, text);
        if StealthFeatures::from_config(&self.config).backend_focus_input()
            && self.backend_focus(element_id).await?
        {
            self.select_all_text().await?;
            for (index, key) in characters.iter().enumerate() {
                self.press_resolved_keys(std::slice::from_ref(key)).await?;
                if index + 1 < characters.len() {
                    tokio::time::sleep(std::time::Duration::from_millis(typing_delay_ms(
                        &mut typing_seed,
                    )))
                    .await;
                }
            }
            return Ok(());
        }
        self.focus_text_entry(element_id, true).await?;
        for (index, key) in characters.iter().enumerate() {
            self.press_resolved_keys(std::slice::from_ref(key)).await?;
            if index + 1 < characters.len() {
                tokio::time::sleep(std::time::Duration::from_millis(typing_delay_ms(
                    &mut typing_seed,
                )))
                .await;
            }
        }
        Ok(())
    }

    pub(super) async fn clear_impl(&self, element_id: u32) -> Result<()> {
        self.ensure_live_ids(element_id).await?;
        if let Some(context @ LiveElementContext::Remote { .. }) =
            self.live_element_context(element_id).await
        {
            return self.clear_frame_text(context, element_id).await;
        }
        self.focus_text_entry(element_id, true).await?;
        let backspace = resolve_pressed_keys(&[String::from("backspace")])?;
        self.press_resolved_keys(&backspace).await
    }

    pub(super) async fn select_option_impl(&self, element_id: u32, value: &str) -> Result<()> {
        self.ensure_live_ids(element_id).await?;
        if let Some(context @ LiveElementContext::Remote { .. }) =
            self.live_element_context(element_id).await
        {
            return self.select_frame_option(context, element_id, value).await;
        }
        let value = serde_json::to_string(value).map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })?;
        self.run_action(select_option_expression(element_id, &value), element_id)
            .await
    }

    pub(super) async fn submit_impl(&self, element_id: u32) -> Result<()> {
        self.ensure_live_ids(element_id).await?;
        if let Some(context @ LiveElementContext::Remote { .. }) =
            self.live_element_context(element_id).await
        {
            return self.submit_frame_element(context, element_id).await;
        }
        let plan = self.submit_plan(element_id).await?;
        if !plan.ok {
            return match plan.kind.as_deref() {
                Some("not_interactable") => Err(Error::ElementNotInteractable { element_id }),
                _ => Err(Error::ElementNotFound { element_id }),
            };
        }
        match plan.strategy.as_deref() {
            Some("click_self") => self.click_impl(element_id).await,
            Some("enter") => {
                let enter = resolve_pressed_keys(&[String::from("enter")])?;
                self.press_resolved_keys(&enter).await
            }
            _ => {
                self.run_action(submit_dom_expression(element_id), element_id)
                    .await
            }
        }
    }

    async fn focus_text_entry(&self, element_id: u32, select_existing: bool) -> Result<()> {
        if let Ok(target) = self.interaction_target(element_id).await
            && target.visible
            && target.pointer_enabled
        {
            let _ = self.pointer_click(element_id, target).await;
        }
        self.run_action(
            prepare_text_entry_expression(element_id, select_existing),
            element_id,
        )
        .await
    }

    async fn submit_plan(&self, element_id: u32) -> Result<SubmitPlan> {
        self.evaluate_bowser_value(submit_plan_expression(element_id))
            .await
    }

    async fn focus_after_pointer_click(&self, element_id: u32, target: InteractionTarget) {
        if target.focus_after_click {
            let _ = self
                .run_action(focus_after_pointer_click_expression(element_id), element_id)
                .await;
        }
    }
}
