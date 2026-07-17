//! Metadata construction from captured elements.

use super::MetadataRecord;
use crate::model::{Element, PageCapture};

impl MetadataRecord {
    /// Builds metadata for a captured element.
    pub fn from_element(element: &Element) -> Option<Self> {
        match element {
            Element::Link {
                id,
                text,
                href,
                focused,
            } => Some(Self::Link {
                element_id: *id,
                text: text.clone(),
                href: href.clone(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Image {
                id,
                alt,
                src,
                description,
                describable,
                focused,
            } => Some(Self::Image {
                element_id: *id,
                alt: alt.clone(),
                src: src.clone(),
                description: description.clone(),
                describable: *describable,
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Button { id, text, focused } => Some(Self::Button {
                element_id: *id,
                text: text.clone(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Input {
                id: Some(id),
                name,
                input_type,
                placeholder,
                value,
                label,
                options,
                focused,
            } => Some(Self::Input {
                element_id: *id,
                name: name.clone(),
                input_type: input_type.clone(),
                placeholder: placeholder.clone(),
                value: value.clone(),
                label: label.clone(),
                options: options.clone(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Table {
                id,
                headers,
                rows,
                focused,
                ..
            } => Some(Self::Table {
                element_id: *id,
                headers: headers.len(),
                rows: rows.len(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::List {
                id,
                list_type,
                items,
                focused,
                ..
            } => Some(Self::List {
                element_id: *id,
                list_type: list_type.clone(),
                items: items.len(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Nav {
                id,
                children,
                focused,
                ..
            } => Some(Self::Nav {
                element_id: *id,
                children: children.len(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Form {
                id,
                action,
                children,
                focused,
                ..
            } => Some(Self::Form {
                element_id: *id,
                action: action.clone(),
                children: children.len(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Section {
                id,
                tag,
                children,
                focused,
                ..
            } => Some(Self::Section {
                element_id: *id,
                tag: tag.clone(),
                children: children.len(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Iframe {
                id,
                src,
                children,
                focused,
                ..
            } => Some(Self::Iframe {
                element_id: *id,
                src: src.clone(),
                children: children.len(),
                focused: *focused,
                visibility: None,
                bounds: None,
            }),
            Element::Input { id: None, .. } | Element::Heading { .. } | Element::Text { .. } => {
                None
            }
        }
    }
}

/// Collects metadata records for every ID-bearing element in a capture tree.
pub(crate) fn collect_metadata_records(capture: &PageCapture) -> Vec<MetadataRecord> {
    let mut records = Vec::new();
    for children in capture.content.buckets() {
        collect_element_records(children, &mut records);
    }
    records
}

fn collect_element_records(elements: &[Element], records: &mut Vec<MetadataRecord>) {
    for element in elements {
        if let Some(record) = MetadataRecord::from_element(element) {
            records.push(record);
        }
        match element {
            Element::Table { headers, rows, .. } => {
                for header in headers {
                    collect_element_records(&header.children, records);
                }
                for row in rows {
                    for cell in &row.cells {
                        collect_element_records(&cell.children, records);
                    }
                }
            }
            Element::List { items, .. } => {
                for item in items {
                    collect_element_records(&item.children, records);
                }
            }
            Element::Nav { children, .. }
            | Element::Form { children, .. }
            | Element::Section { children, .. }
            | Element::Iframe { children, .. } => collect_element_records(children, records),
            Element::Heading { .. }
            | Element::Text { .. }
            | Element::Link { .. }
            | Element::Button { .. }
            | Element::Input { .. }
            | Element::Image { .. } => {}
        }
    }
}
