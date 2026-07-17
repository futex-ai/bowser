//! Input helpers for raw target-backed iframe connections.

use chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventType;
use serde_json::{Value, json};

use crate::error::Result;
use crate::keyboard::ResolvedKey;

use super::iframe_target::RemoteFrameTarget;

impl RemoteFrameTarget {
    pub(super) async fn dispatch_key_event(
        &mut self,
        key: &ResolvedKey,
        event_type: DispatchKeyEventType,
        modifiers: i64,
    ) -> Result<()> {
        let mut params = json!({
            "type": event_type.as_ref(),
            "modifiers": modifiers,
            "key": key.key.clone(),
            "code": key.code.clone(),
            "windowsVirtualKeyCode": key.key_code,
            "nativeVirtualKeyCode": key.key_code,
        });
        if event_type == DispatchKeyEventType::KeyDown
            && let Some(text) = key.event_text()
        {
            params["text"] = Value::from(text.clone());
            params["unmodifiedText"] = Value::from(text);
        }
        let _: Value = self.call("Input.dispatchKeyEvent", params).await?;
        Ok(())
    }

    pub(super) async fn press_resolved_keys(&mut self, keys: &[ResolvedKey]) -> Result<()> {
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
