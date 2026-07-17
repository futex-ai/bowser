//! Cross-frame capture stitching and final ID assignment.

use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::page::FrameId;

use crate::error::{Error, Result};
use crate::model::{Element, PageCapture, PageContent};

use super::finalize_collections::{finalize_list_items, finalize_table_cells, finalize_table_rows};
use super::finalize_support::{FinalizedCapture, IdMappings, assign_id, lookup_child_capture};
use super::types::{FrameOwnerKey, RawCaptureEnvelope, RawElement};

/// Finalizes a captured frame tree into the public `PageCapture` model.
pub(crate) fn finalize_capture(
    root_frame_id: &FrameId,
    captures: &HashMap<FrameId, RawCaptureEnvelope>,
    child_frames_by_owner: &HashMap<FrameOwnerKey, FrameId>,
) -> Result<FinalizedCapture> {
    let root = captures
        .get(root_frame_id)
        .ok_or(Error::CaptureParse)?
        .clone();
    let mut state = FinalizeState {
        captures,
        child_frames_by_owner,
        next_id: 1_u32,
        id_mappings: HashMap::new(),
    };
    let content = PageContent::new(
        state.children(&root.content.visible, root_frame_id),
        state.children(&root.content.obscured, root_frame_id),
    );
    let body_id = root
        .body_id
        .map(|temp_id| state.assign(root_frame_id, temp_id));
    let obscured_body_id = root
        .obscured_body_id
        .map(|temp_id| state.assign(root_frame_id, temp_id));
    Ok(FinalizedCapture {
        capture: PageCapture {
            url: root.url,
            title: root.title,
            body_id,
            obscured_body_id,
            content,
        },
        id_mappings: state.id_mappings,
    })
}

struct FinalizeState<'a> {
    captures: &'a HashMap<FrameId, RawCaptureEnvelope>,
    child_frames_by_owner: &'a HashMap<FrameOwnerKey, FrameId>,
    next_id: u32,
    id_mappings: IdMappings,
}

impl FinalizeState<'_> {
    fn assign(&mut self, frame_id: &FrameId, temp_id: u32) -> u32 {
        assign_id(frame_id, temp_id, &mut self.next_id, &mut self.id_mappings)
    }

    fn children(&mut self, children: &[RawElement], frame_id: &FrameId) -> Vec<Element> {
        children
            .iter()
            .map(|child| self.element(child, frame_id))
            .collect()
    }

    fn frame_content_children(
        &mut self,
        capture: &RawCaptureEnvelope,
        frame_id: &FrameId,
    ) -> Vec<Element> {
        let mut children = self.children(&capture.content.visible, frame_id);
        children.extend(self.children(&capture.content.obscured, frame_id));
        children
    }

    fn element(&mut self, element: &RawElement, frame_id: &FrameId) -> Element {
        match element {
            RawElement::Heading { level, text } => Element::Heading {
                level: *level,
                text: text.clone(),
            },
            RawElement::Text { text } => Element::Text { text: text.clone() },
            RawElement::Link {
                id,
                text,
                href,
                focused,
            } => Element::Link {
                id: self.assign(frame_id, *id),
                text: text.clone(),
                href: href.clone(),
                focused: *focused,
            },
            RawElement::Button { id, text, focused } => Element::Button {
                id: self.assign(frame_id, *id),
                text: text.clone(),
                focused: *focused,
            },
            RawElement::Input {
                id,
                name,
                input_type,
                placeholder,
                value,
                label,
                options,
                focused,
            } => Element::Input {
                id: id.map(|temp_id| self.assign(frame_id, temp_id)),
                name: name.clone(),
                input_type: input_type.clone(),
                placeholder: placeholder.clone(),
                value: value.clone(),
                label: label.clone(),
                options: options.clone(),
                focused: *focused,
            },
            RawElement::Image {
                id,
                alt,
                src,
                description,
                describable,
                focused,
            } => Element::Image {
                id: self.assign(frame_id, *id),
                alt: alt.clone(),
                src: src.clone(),
                description: description.clone(),
                describable: *describable,
                focused: *focused,
            },
            RawElement::Table {
                id,
                headers,
                rows,
                truncation,
                focused,
            } => {
                let id = self.assign(frame_id, *id);
                let mut finalize = |children: &[RawElement]| self.children(children, frame_id);
                Element::Table {
                    id,
                    headers: finalize_table_cells(headers, &mut finalize),
                    rows: finalize_table_rows(rows, &mut finalize),
                    truncation: truncation.clone(),
                    focused: *focused,
                }
            }
            RawElement::List {
                id,
                list_type,
                items,
                truncation,
                focused,
            } => {
                let id = self.assign(frame_id, *id);
                let mut finalize = |children: &[RawElement]| self.children(children, frame_id);
                Element::List {
                    id,
                    list_type: list_type.clone(),
                    items: finalize_list_items(items, &mut finalize),
                    truncation: truncation.clone(),
                    focused: *focused,
                }
            }
            RawElement::Nav {
                id,
                children,
                truncation,
                focused,
            } => {
                let id = self.assign(frame_id, *id);
                Element::Nav {
                    id,
                    children: self.children(children, frame_id),
                    truncation: truncation.clone(),
                    focused: *focused,
                }
            }
            RawElement::Form {
                id,
                action,
                children,
                truncation,
                focused,
            } => {
                let id = self.assign(frame_id, *id);
                Element::Form {
                    id,
                    action: action.clone(),
                    children: self.children(children, frame_id),
                    truncation: truncation.clone(),
                    focused: *focused,
                }
            }
            RawElement::Section {
                id,
                tag,
                children,
                truncation,
                focused,
            } => {
                let id = self.assign(frame_id, *id);
                Element::Section {
                    id,
                    tag: tag.clone(),
                    children: self.children(children, frame_id),
                    truncation: truncation.clone(),
                    focused: *focused,
                }
            }
            RawElement::Iframe {
                id: raw_id,
                src,
                children,
                truncation,
                focused,
            } => {
                let id = self.assign(frame_id, *raw_id);
                let children = if children.is_empty() {
                    lookup_child_capture(
                        frame_id,
                        *raw_id,
                        self.child_frames_by_owner,
                        self.captures,
                    )
                    .map(|(child_frame_id, capture)| {
                        self.frame_content_children(capture, &child_frame_id)
                    })
                    .unwrap_or_default()
                } else {
                    self.children(children, frame_id)
                };
                Element::Iframe {
                    id,
                    src: src.clone(),
                    children,
                    truncation: truncation.clone(),
                    focused: *focused,
                }
            }
        }
    }
}
