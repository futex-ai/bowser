//! CDP and evaluation helpers.

use std::future::Future;
use std::time::{SystemTime, UNIX_EPOCH};

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use serde::Deserialize;

use crate::browser_identity;
use crate::cdp_trace;
use crate::error::{Error, Result};
use crate::keyboard::ResolvedKey;

use super::types::{ActionResponse, LivePage};

pub(crate) const INPUT_CDP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

impl LivePage {
    pub(super) async fn evaluate_value<T: for<'de> Deserialize<'de>>(
        &self,
        expression: String,
    ) -> Result<T> {
        if self.config.stealth {
            let _ = self
                .evaluate_raw_value::<serde_json::Value>(
                    browser_identity::runtime_cleanup_script().to_string(),
                )
                .await;
        }
        self.evaluate_raw_value(expression).await
    }

    pub(super) async fn evaluate_raw_value<T: for<'de> Deserialize<'de>>(
        &self,
        expression: String,
    ) -> Result<T> {
        cdp_trace::record_method("Runtime.evaluate");
        let mut params = EvaluateParams::new(expression);
        params.await_promise = Some(true);
        params.return_by_value = Some(true);
        let response = self
            .page
            .execute(params)
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?
            .result;
        if let Some(exception) = response.exception_details {
            return Err(Error::JsEvaluation {
                reason: format!("{exception:?}"),
            });
        }
        let value = response.result.value.unwrap_or(serde_json::Value::Null);
        serde_json::from_value(value).map_err(|err| Error::JsEvaluation {
            reason: err.to_string(),
        })
    }

    pub(super) async fn evaluate_side_effect(&self, expression: String) -> Result<()> {
        let _: serde_json::Value = self.evaluate_raw_value(expression).await?;
        Ok(())
    }

    pub(super) async fn run_action(&self, expression: String, element_id: u32) -> Result<()> {
        let response: ActionResponse = self.evaluate_bowser_value(expression).await?;
        finish_action_response(response, element_id)
    }

    pub(super) async fn input_cdp<F, T, E>(&self, operation: &str, future: F) -> Result<T>
    where
        F: Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        match tokio::time::timeout(INPUT_CDP_TIMEOUT, future).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(err)) => Err(Error::cdp(format!("failed to {operation}: {err}"))),
            Err(_) => Err(Error::cdp(format!(
                "timed out while trying to {operation} after {}ms",
                INPUT_CDP_TIMEOUT.as_millis()
            ))),
        }
    }

    pub(super) async fn dispatch_key_event(
        &self,
        key: &ResolvedKey,
        event_type: DispatchKeyEventType,
        modifiers: i64,
    ) -> Result<()> {
        let mut params = DispatchKeyEventParams::new(event_type.clone());
        params.modifiers = Some(modifiers);
        params.key = Some(key.key.clone());
        params.code = Some(key.code.clone());
        params.windows_virtual_key_code = Some(key.key_code);
        params.native_virtual_key_code = Some(key.key_code);
        if event_type == DispatchKeyEventType::KeyDown
            && let Some(text) = key.event_text()
        {
            params.text = Some(text.clone());
            params.unmodified_text = Some(text);
        }
        cdp_trace::record_method("Input.dispatchKeyEvent");
        self.page
            .execute(params)
            .await
            .map_err(|err| Error::cdp(format!("failed to dispatch key event: {err}")))?;
        Ok(())
    }

    pub(super) async fn press_resolved_keys(&self, keys: &[ResolvedKey]) -> Result<()> {
        let mut modifiers = 0_i64;
        for key in keys {
            let event_type = if !key.is_modifier() && key.event_text().is_some() {
                DispatchKeyEventType::KeyDown
            } else {
                DispatchKeyEventType::RawKeyDown
            };
            let next_modifiers = modifiers | key.modifier_mask;
            self.dispatch_key_event(key, event_type, next_modifiers)
                .await?;
            modifiers = next_modifiers;
        }
        for key in keys.iter().rev() {
            self.dispatch_key_event(key, DispatchKeyEventType::KeyUp, modifiers)
                .await?;
            modifiers &= !key.modifier_mask;
        }
        Ok(())
    }
}

pub(super) fn finish_action_response(response: ActionResponse, element_id: u32) -> Result<()> {
    if response.ok {
        return Ok(());
    }
    match response.kind.as_deref() {
        Some("not_interactable") => Err(Error::ElementNotInteractable { element_id }),
        _ => Err(Error::ElementNotFound { element_id }),
    }
}

pub(super) fn prepare_text_entry_expression(element_id: u32, select_existing: bool) -> String {
    let select_existing = if select_existing { "true" } else { "false" };
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  const tag = el.tagName ? el.tagName.toLowerCase() : '';
  const type = tag === 'input' ? (el.getAttribute('type') || 'text').toLowerCase() : tag;
  const allowed = new Set(['text', 'password', 'email', 'number', 'tel', 'url', 'search', 'date', 'textarea']);
  if (!allowed.has(type)) return {{ ok: false, kind: 'not_interactable' }};
  el.scrollIntoView({{ block: 'center', inline: 'center' }});
  if (typeof el.focus === 'function') {{
    try {{
      el.focus({{ preventScroll: true }});
    }} catch (_error) {{
      el.focus();
    }}
  }}
  if ({select_existing}) {{
    try {{
      if (typeof el.select === 'function') {{
        el.select();
      }} else if (typeof el.setSelectionRange === 'function') {{
        const value = typeof el.value === 'string' ? el.value : '';
        el.setSelectionRange(0, value.length);
      }}
    }} catch (_error) {{}}
  }}
  return {{ ok: true }};
}})()
"#
    )
}

fn initial_typing_seed(element_id: u32, text: &str) -> u64 {
    let time_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let text_seed = text.bytes().fold(0_u64, |acc, byte| {
        acc.wrapping_mul(131).wrapping_add(u64::from(byte))
    });
    mix_typing_seed(time_seed ^ (u64::from(element_id) << 32) ^ text_seed)
}

pub(crate) fn typing_delay_ms(seed: &mut u64) -> u64 {
    *seed = mix_typing_seed(*seed);
    14 + (*seed % 17)
}

pub(crate) fn mix_typing_seed(seed: u64) -> u64 {
    let mut value = if seed == 0 {
        0x9E37_79B9_7F4A_7C15
    } else {
        seed
    };
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}

pub(super) fn typing_seed_for_text(element_id: u32, text: &str) -> u64 {
    initial_typing_seed(element_id, text)
}
