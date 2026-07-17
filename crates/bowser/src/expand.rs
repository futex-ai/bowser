//! Element lookup and expansion helpers.

use crate::model::{Element, ListItem, PageCapture, TableCell, TableRow};

/// Finds an element by ID.
pub fn find_element(elements: &[Element], element_id: u32) -> Option<&Element> {
    for element in elements {
        if element.id() == Some(element_id) {
            return Some(element);
        }
        if let Some(found) = find_in_element(element, element_id) {
            return Some(found);
        }
    }
    None
}

/// Finds an element by ID mutably.
pub fn find_element_mut(elements: &mut [Element], element_id: u32) -> Option<&mut Element> {
    for element in elements {
        if element.id() == Some(element_id) {
            return Some(element);
        }
        if let Some(found) = find_in_element_mut(element, element_id) {
            return Some(found);
        }
    }
    None
}

/// Expands an element from a full capture.
pub fn expand_element(capture: &PageCapture, element_id: u32) -> Option<Element> {
    find_element_in_capture(capture, element_id)
        .cloned()
        .or_else(|| synthetic_body_root(capture, element_id, None))
}

/// Finds an element by ID within a full page capture, including synthetic root containers.
pub fn find_capture_element(capture: &PageCapture, element_id: u32) -> Option<Element> {
    find_element_in_capture(capture, element_id)
        .cloned()
        .or_else(|| synthetic_body_root(capture, element_id, None))
}

/// Builds the synthetic root body container when its ID is requested.
pub fn synthetic_body_root(
    capture: &PageCapture,
    element_id: u32,
    truncation: Option<crate::model::TruncationInfo>,
) -> Option<Element> {
    let children = if capture.body_id == Some(element_id) {
        &capture.content.visible
    } else if capture.obscured_body_id == Some(element_id) {
        &capture.content.obscured
    } else {
        return None;
    };
    Some(Element::Section {
        id: element_id,
        tag: "body".to_string(),
        children: children.clone(),
        truncation,
        focused: false,
    })
}

fn find_element_in_capture(capture: &PageCapture, element_id: u32) -> Option<&Element> {
    capture
        .content
        .buckets()
        .into_iter()
        .find_map(|children| find_element(children, element_id))
}

fn find_in_element(element: &Element, element_id: u32) -> Option<&Element> {
    match element {
        Element::Table { headers, rows, .. } => {
            find_in_cells(headers, element_id).or_else(|| find_in_rows(rows, element_id))
        }
        Element::List { items, .. } => find_in_items(items, element_id),
        Element::Nav { children, .. }
        | Element::Form { children, .. }
        | Element::Section { children, .. }
        | Element::Iframe { children, .. } => find_element(children, element_id),
        Element::Heading { .. }
        | Element::Text { .. }
        | Element::Link { .. }
        | Element::Button { .. }
        | Element::Input { .. }
        | Element::Image { .. } => None,
    }
}

fn find_in_element_mut(element: &mut Element, element_id: u32) -> Option<&mut Element> {
    match element {
        Element::Table { headers, rows, .. } => {
            find_in_cells_mut(headers, element_id).or_else(|| find_in_rows_mut(rows, element_id))
        }
        Element::List { items, .. } => find_in_items_mut(items, element_id),
        Element::Nav { children, .. }
        | Element::Form { children, .. }
        | Element::Section { children, .. }
        | Element::Iframe { children, .. } => find_element_mut(children, element_id),
        Element::Heading { .. }
        | Element::Text { .. }
        | Element::Link { .. }
        | Element::Button { .. }
        | Element::Input { .. }
        | Element::Image { .. } => None,
    }
}

fn find_in_items(items: &[ListItem], element_id: u32) -> Option<&Element> {
    for item in items {
        if let Some(found) = find_element(&item.children, element_id) {
            return Some(found);
        }
    }
    None
}

fn find_in_items_mut(items: &mut [ListItem], element_id: u32) -> Option<&mut Element> {
    for item in items {
        if let Some(found) = find_element_mut(&mut item.children, element_id) {
            return Some(found);
        }
    }
    None
}

fn find_in_rows(rows: &[TableRow], element_id: u32) -> Option<&Element> {
    for row in rows {
        if let Some(found) = find_in_cells(&row.cells, element_id) {
            return Some(found);
        }
    }
    None
}

fn find_in_rows_mut(rows: &mut [TableRow], element_id: u32) -> Option<&mut Element> {
    for row in rows {
        if let Some(found) = find_in_cells_mut(&mut row.cells, element_id) {
            return Some(found);
        }
    }
    None
}

fn find_in_cells(cells: &[TableCell], element_id: u32) -> Option<&Element> {
    for cell in cells {
        if let Some(found) = find_element(&cell.children, element_id) {
            return Some(found);
        }
    }
    None
}

fn find_in_cells_mut(cells: &mut [TableCell], element_id: u32) -> Option<&mut Element> {
    for cell in cells {
        if let Some(found) = find_element_mut(&mut cell.children, element_id) {
            return Some(found);
        }
    }
    None
}
