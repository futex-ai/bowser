//! Capture tree traversal helpers.

use crate::model::Element;

pub(super) fn collect_image_ids(children: &[Element]) -> Vec<u32> {
    let mut ids = Vec::new();
    for child in children {
        match child {
            Element::Image { id, .. } => ids.push(*id),
            Element::Table { headers, rows, .. } => {
                for header in headers {
                    ids.extend(collect_image_ids(&header.children));
                }
                for row in rows {
                    for cell in &row.cells {
                        ids.extend(collect_image_ids(&cell.children));
                    }
                }
            }
            Element::List { items, .. } => {
                for item in items {
                    ids.extend(collect_image_ids(&item.children));
                }
            }
            Element::Nav { children, .. }
            | Element::Form { children, .. }
            | Element::Section { children, .. }
            | Element::Iframe { children, .. } => {
                ids.extend(collect_image_ids(children));
            }
            Element::Heading { .. }
            | Element::Text { .. }
            | Element::Link { .. }
            | Element::Button { .. }
            | Element::Input { .. } => {}
        }
    }
    ids
}
