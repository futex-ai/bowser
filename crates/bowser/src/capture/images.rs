//! Image capability helpers for captures.

use std::collections::HashMap;

use crate::{
    expand::find_element_mut,
    model::{Element, MetadataRecord, PageCapture},
};

use super::tree::collect_image_ids;

/// Marks image elements as describable when an AI description backend is available.
pub(crate) fn set_images_describable(
    capture: &mut PageCapture,
    metadata: &mut [MetadataRecord],
    describable: bool,
) {
    let ids = collect_capture_image_ids(capture);
    for id in ids {
        if let Some(Element::Image {
            describable: slot, ..
        }) = find_capture_element_mut(capture, id)
        {
            *slot = describable;
        }
    }
    for record in metadata.iter_mut() {
        if let MetadataRecord::Image {
            describable: slot, ..
        } = record
        {
            *slot = describable;
        }
    }
}

/// Collects cached image descriptions from a prior capture.
pub(crate) fn known_image_descriptions(capture: &PageCapture) -> HashMap<ImageKey, String> {
    let mut descriptions = HashMap::new();
    for children in capture.content.buckets() {
        collect_descriptions(children, &mut descriptions);
    }
    descriptions
}

/// Reapplies cached descriptions to matching images in a fresh capture.
pub(crate) fn restore_image_descriptions(
    capture: &mut PageCapture,
    metadata: &mut [MetadataRecord],
    descriptions: &HashMap<ImageKey, String>,
) {
    let ids = collect_capture_image_ids(capture);
    for id in ids {
        if let Some(Element::Image {
            alt,
            src,
            description: slot,
            ..
        }) = find_capture_element_mut(capture, id)
        {
            let key = ImageKey {
                alt: alt.clone(),
                src: src.clone(),
            };
            if let Some(description) = descriptions.get(&key) {
                *slot = Some(description.clone());
            }
        }
    }
    for record in metadata.iter_mut() {
        if let MetadataRecord::Image {
            alt,
            src,
            description: slot,
            ..
        } = record
        {
            let key = ImageKey {
                alt: alt.clone(),
                src: src.clone(),
            };
            if let Some(description) = descriptions.get(&key) {
                *slot = Some(description.clone());
            }
        }
    }
}

fn collect_capture_image_ids(capture: &PageCapture) -> Vec<u32> {
    capture
        .content
        .buckets()
        .into_iter()
        .flat_map(collect_image_ids)
        .collect()
}

fn find_capture_element_mut(capture: &mut PageCapture, id: u32) -> Option<&mut Element> {
    capture
        .content
        .buckets_mut()
        .into_iter()
        .find_map(|children| find_element_mut(children, id))
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ImageKey {
    alt: String,
    src: String,
}

fn collect_descriptions(elements: &[Element], descriptions: &mut HashMap<ImageKey, String>) {
    for element in elements {
        match element {
            Element::Image {
                alt,
                src,
                description: Some(description),
                ..
            } => {
                descriptions.insert(
                    ImageKey {
                        alt: alt.clone(),
                        src: src.clone(),
                    },
                    description.clone(),
                );
            }
            Element::Table { headers, rows, .. } => {
                for header in headers {
                    collect_descriptions(&header.children, descriptions);
                }
                for row in rows {
                    for cell in &row.cells {
                        collect_descriptions(&cell.children, descriptions);
                    }
                }
            }
            Element::List { items, .. } => {
                for item in items {
                    collect_descriptions(&item.children, descriptions);
                }
            }
            Element::Nav { children, .. }
            | Element::Form { children, .. }
            | Element::Section { children, .. }
            | Element::Iframe { children, .. } => collect_descriptions(children, descriptions),
            Element::Heading { .. }
            | Element::Text { .. }
            | Element::Link { .. }
            | Element::Button { .. }
            | Element::Input { .. }
            | Element::Image { .. } => {}
        }
    }
}
