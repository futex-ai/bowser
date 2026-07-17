//! Metadata extraction helpers.

use crate::error::{Error, Result};
use crate::expand::find_capture_element;
use crate::model::MetadataRecord;

use super::interaction_scripts::runtime_state_expression;
use super::types::{Bounds, ElementRuntimeState, LivePage};

impl LivePage {
    pub(super) async fn metadata_impl(&self, element_id: u32) -> Result<MetadataRecord> {
        self.ensure_live_ids(element_id).await?;
        let record = {
            let state = self.state.lock().await;
            state
                .full_capture
                .clone()
                .or_else(|| state.preview_capture.clone())
        }
        .and_then(|capture| find_capture_element(&capture, element_id))
        .and_then(|element| MetadataRecord::from_element(&element))
        .ok_or(Error::MetadataNotFound { element_id })?;
        let runtime_state = if let Some(context) = self.live_element_context(element_id).await {
            self.frame_element_runtime_state(context).await?
        } else {
            self.element_runtime_state(element_id).await?
        };
        let record = record
            .with_describable(self.summarizer.is_some())
            .with_focus(runtime_state.focused)
            .with_visibility(runtime_state.visibility());
        Ok(match runtime_state.bounds {
            Some(bounds) => record.with_bounds(bounds),
            None => record,
        })
    }

    pub(super) async fn element_bounds(&self, element_id: u32) -> Result<Bounds> {
        let expression = format!(
            r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return null;
  el.scrollIntoView({{ block: 'center', inline: 'center' }});
  const rect = el.getBoundingClientRect();
  return {{
    x: rect.x + window.scrollX,
    y: rect.y + window.scrollY,
    width: rect.width,
    height: rect.height
  }};
}})()
"#
        );
        self.evaluate_bowser_value::<Option<Bounds>>(expression)
            .await?
            .ok_or(Error::ElementNotFound { element_id })
    }

    async fn element_runtime_state(&self, element_id: u32) -> Result<ElementRuntimeState> {
        self.evaluate_bowser_value(runtime_state_expression(element_id, true))
            .await
    }
}
