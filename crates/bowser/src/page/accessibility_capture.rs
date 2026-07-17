//! Accessibility-tree capture and backend-node interactions.

use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::accessibility::{AxNode, AxValue, GetFullAxTreeParams};
use chromiumoxide::cdp::browser_protocol::dom::{
    BackendNodeId, FocusParams, GetBoxModelParams, Quad,
};

use crate::cdp_trace;
use crate::error::{Error, Result};
use crate::model::{Element, InputType, PageCapture, PageContent};

use super::types::{Bounds, InteractionTarget, LivePage};

impl LivePage {
    pub(super) async fn capture_accessibility_snapshot(&self) -> Result<PageCapture> {
        cdp_trace::record_method("Accessibility.getFullAXTree");
        let tree = match self
            .page
            .execute(GetFullAxTreeParams::builder().build())
            .await
        {
            Ok(tree) => tree,
            Err(err) => {
                return Err(Error::CaptureScript {
                    reason: err.to_string(),
                });
            }
        };

        let mut next_id = 1_u32;
        let mut backend_node_ids = HashMap::new();
        let mut title = String::new();
        let mut children = Vec::new();
        for node in &tree.nodes {
            let role = normalized_role(node);
            if title.is_empty() && role == "rootwebarea" {
                title = ax_text(node.name.as_ref());
            }
            if let Some((element, backend_node_id)) = ax_element(node, &role, &mut next_id) {
                if let Some(id) = element.id() {
                    backend_node_ids.insert(id, backend_node_id);
                }
                children.push(element);
            }
        }

        self.state.lock().await.backend_node_ids = backend_node_ids;
        let url = match self.page.url().await {
            Ok(url) => url.unwrap_or_default(),
            Err(err) => {
                return Err(Error::JsEvaluation {
                    reason: err.to_string(),
                });
            }
        };
        Ok(PageCapture {
            url,
            title,
            body_id: None,
            obscured_body_id: None,
            content: PageContent::visible_only(children),
        })
    }

    pub(super) async fn backend_interaction_target(
        &self,
        element_id: u32,
    ) -> Result<Option<InteractionTarget>> {
        let Some(backend_node_id) = self.backend_node_id(element_id).await else {
            return Ok(None);
        };
        cdp_trace::record_method("DOM.getBoxModel");
        let model = match self
            .page
            .execute(
                GetBoxModelParams::builder()
                    .backend_node_id(backend_node_id)
                    .build(),
            )
            .await
        {
            Ok(model) => model.model.clone(),
            Err(err) => {
                return Err(Error::cdp(format!(
                    "failed to get element box model: {err}"
                )));
            }
        };
        let bounds = quad_bounds(&model.content)?;
        Ok(Some(InteractionTarget {
            x: bounds.x + (bounds.width * 0.5),
            y: bounds.y + (bounds.height * 0.5),
            viewport_width: f64::from(self.config.viewport.width),
            viewport_height: f64::from(self.config.viewport.height),
            visible: bounds.width > 0.0 && bounds.height > 0.0,
            pointer_enabled: bounds.width > 0.0 && bounds.height > 0.0,
            enabled: true,
            focus_after_click: false,
        }))
    }

    pub(super) async fn backend_focus(&self, element_id: u32) -> Result<bool> {
        let Some(backend_node_id) = self.backend_node_id(element_id).await else {
            return Ok(false);
        };
        cdp_trace::record_method("DOM.focus");
        if let Err(err) = self
            .page
            .execute(
                FocusParams::builder()
                    .backend_node_id(backend_node_id)
                    .build(),
            )
            .await
        {
            return Err(Error::cdp(format!("failed to focus backend node: {err}")));
        }
        Ok(true)
    }

    async fn backend_node_id(&self, element_id: u32) -> Option<BackendNodeId> {
        self.state
            .lock()
            .await
            .backend_node_ids
            .get(&element_id)
            .copied()
    }
}

fn ax_element(node: &AxNode, role: &str, next_id: &mut u32) -> Option<(Element, BackendNodeId)> {
    if node.ignored {
        return None;
    }
    let backend_node_id = node.backend_dom_node_id?;
    let id = take_id(next_id);
    let name = ax_text(node.name.as_ref());
    let value = ax_text(node.value.as_ref());
    let element = match role {
        "button" | "menubutton" => Element::Button {
            id,
            text: fallback_label(&name, "button"),
            focused: ax_bool_property(node, "focused"),
        },
        "link" => Element::Link {
            id,
            text: fallback_label(&name, "link"),
            href: String::new(),
            focused: ax_bool_property(node, "focused"),
        },
        "checkbox" => ax_input(id, InputType::Checkbox, name, value, node),
        "radio" => ax_input(id, InputType::Radio, name, value, node),
        "searchbox" => ax_input(id, InputType::Search, name, value, node),
        "textbox" | "combobox" => ax_input(id, InputType::Text, name, value, node),
        _ => return None,
    };
    Some((element, backend_node_id))
}

fn ax_input(id: u32, input_type: InputType, name: String, value: String, node: &AxNode) -> Element {
    Element::Input {
        id: Some(id),
        name: None,
        input_type,
        placeholder: None,
        value,
        label: (!name.is_empty()).then_some(name),
        options: Vec::new(),
        focused: ax_bool_property(node, "focused"),
    }
}

fn normalized_role(node: &AxNode) -> String {
    ax_text(node.role.as_ref())
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn ax_text(value: Option<&AxValue>) -> String {
    value
        .and_then(|value| value.value.as_ref())
        .and_then(|value| match value {
            serde_json::Value::String(text) => Some(text.clone()),
            serde_json::Value::Bool(value) => Some(value.to_string()),
            serde_json::Value::Number(value) => Some(value.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

fn ax_bool_property(node: &AxNode, name: &str) -> bool {
    node.properties
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|property| property.name.as_ref() == name)
        .map(|property| ax_text(Some(&property.value)) == "true")
        .unwrap_or(false)
}

fn fallback_label(text: &str, fallback: &str) -> String {
    if text.is_empty() {
        format!("[{fallback}]")
    } else {
        text.to_string()
    }
}

fn take_id(next_id: &mut u32) -> u32 {
    let id = *next_id;
    *next_id = next_id.saturating_add(1);
    id
}

fn quad_bounds(quad: &Quad) -> Result<Bounds> {
    let points = quad.inner();
    if points.len() < 8 {
        return Err(Error::CaptureScript {
            reason: "backend node box model did not include a full quad".to_string(),
        });
    }
    let xs = [points[0], points[2], points[4], points[6]];
    let ys = [points[1], points[3], points[5], points[7]];
    let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Ok(Bounds {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(0.0),
        height: (max_y - min_y).max(0.0),
    })
}
