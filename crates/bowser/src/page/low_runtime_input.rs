//! Low-runtime input helpers.

use std::time::Duration;

use super::types::LivePage;
use crate::model::{Element, InputType, MetadataRecord, PageCapture};

impl LivePage {
    pub(super) async fn search_submit_input_id(&self, element_id: u32) -> Option<u32> {
        let state = self.state.lock().await;
        let search_submit = state.metadata_records.iter().any(|record| {
            matches!(
                record,
                MetadataRecord::Button { element_id: id, text, .. }
                    if *id == element_id && is_search_submit_label(text)
            )
        }) || state
            .preview_capture
            .as_ref()
            .and_then(|capture| button_text_in_capture(capture, element_id))
            .is_some_and(is_search_submit_label);
        if !search_submit {
            return None;
        }
        focused_text_input_id(&state.metadata_records)
            .or_else(|| non_empty_text_input_id(&state.metadata_records))
            .or_else(|| first_text_input_id(&state.metadata_records))
    }

    pub(super) async fn wait_after_possible_navigation(&self, timeout: Duration) {
        let _ = tokio::time::timeout(timeout, self.page.wait_for_navigation()).await;
    }

    pub(super) async fn enter_activatable_element(&self, element_id: u32) -> bool {
        let state = self.state.lock().await;
        state.metadata_records.iter().any(|record| {
            matches!(
                record,
                MetadataRecord::Button { element_id: id, .. }
                    | MetadataRecord::Link { element_id: id, .. }
                    if *id == element_id
            )
        }) || state
            .preview_capture
            .as_ref()
            .is_some_and(|capture| enter_activatable_in_capture(capture, element_id))
    }
}

fn is_search_submit_label(text: &str) -> bool {
    matches!(
        text.trim().to_ascii_lowercase().as_str(),
        "search" | "google search"
    )
}

fn button_text_in_capture(capture: &PageCapture, element_id: u32) -> Option<&str> {
    for elements in capture.content.buckets() {
        if let Some(found) = button_text_in_elements(elements, element_id) {
            return Some(found);
        }
    }
    None
}

fn button_text_in_elements(elements: &[Element], element_id: u32) -> Option<&str> {
    for element in elements {
        if let Element::Button { id, text, .. } = element
            && *id == element_id
        {
            return Some(text);
        }
        if let Some(found) = element
            .children()
            .and_then(|children| button_text_in_elements(children, element_id))
        {
            return Some(found);
        }
    }
    None
}

fn enter_activatable_in_capture(capture: &PageCapture, element_id: u32) -> bool {
    capture
        .content
        .buckets()
        .into_iter()
        .any(|elements| enter_activatable_in_elements(elements, element_id))
}

fn enter_activatable_in_elements(elements: &[Element], element_id: u32) -> bool {
    for element in elements {
        match element {
            Element::Button { id, .. } | Element::Link { id, .. } if *id == element_id => {
                return true;
            }
            _ => {}
        }
        if element
            .children()
            .is_some_and(|children| enter_activatable_in_elements(children, element_id))
        {
            return true;
        }
    }
    false
}

fn focused_text_input_id(records: &[MetadataRecord]) -> Option<u32> {
    records.iter().find_map(|record| match record {
        MetadataRecord::Input {
            element_id,
            input_type: InputType::Text | InputType::Search,
            focused: true,
            ..
        } => Some(*element_id),
        _ => None,
    })
}

fn non_empty_text_input_id(records: &[MetadataRecord]) -> Option<u32> {
    records.iter().find_map(|record| match record {
        MetadataRecord::Input {
            element_id,
            input_type: InputType::Text | InputType::Search,
            value,
            ..
        } if !value.is_empty() => Some(*element_id),
        _ => None,
    })
}

fn first_text_input_id(records: &[MetadataRecord]) -> Option<u32> {
    records.iter().find_map(|record| match record {
        MetadataRecord::Input {
            element_id,
            input_type: InputType::Text | InputType::Search,
            ..
        } => Some(*element_id),
        _ => None,
    })
}
