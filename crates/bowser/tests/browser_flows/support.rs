#[path = "../support/mod.rs"]
mod shared_support;

use std::time::Duration;

use bowser::{Element, ElementBounds, InputType};
use serde::Deserialize;

pub(crate) use shared_support::{
    BROWSER_TEST_TIMEOUT, TestServer, browser_test_guard, spawn_server, test_config,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TypingStats {
    pub(crate) value: String,
    pub(crate) keydown_count: u64,
    pub(crate) trusted_keydown_count: u64,
    pub(crate) input_count: u64,
    pub(crate) trusted_input_count: u64,
    pub(crate) keyup_count: u64,
    pub(crate) trusted_keyup_count: u64,
    pub(crate) change_count: u64,
    pub(crate) trusted_change_count: u64,
    pub(crate) elapsed_ms: f64,
}

pub(crate) fn contains_hidden_input(elements: &[Element]) -> bool {
    walk(elements, &|element| {
        matches!(
            element,
            Element::Input {
                id: None,
                input_type: InputType::Hidden,
                name: Some(name),
                value,
                ..
            } if name == "csrf_token" && value == "secret"
        )
    })
}

pub(crate) fn table_contains_coupon_input(elements: &[Element]) -> bool {
    walk(elements, &|element| match element {
        Element::Table { rows, .. } => rows.iter().any(|row| {
            row.cells.iter().any(|cell| {
                cell.children.iter().any(|child| {
                    matches!(
                        child,
                        Element::Input {
                            input_type: InputType::Text,
                            name: Some(name),
                            ..
                        } if name == "coupon"
                    )
                })
            })
        }),
        _ => false,
    })
}

pub(crate) fn contains_text(elements: &[Element], target: &str) -> bool {
    walk(elements, &|element| match element {
        Element::Heading { text, .. } | Element::Text { text } => text == target,
        _ => false,
    })
}

pub(crate) fn find_image_id(elements: &[Element]) -> Option<u32> {
    find_image_id_by_alt(elements, "Checkerboard")
}

pub(crate) fn find_image_id_by_alt(elements: &[Element], target_alt: &str) -> Option<u32> {
    find(elements, &|element| match element {
        Element::Image { id, alt, .. } if alt == target_alt => Some(*id),
        _ => None,
    })
}

pub(crate) fn find_input_id_by_name(elements: &[Element], target_name: &str) -> Option<u32> {
    find(elements, &|element| match element {
        Element::Input {
            id: Some(id),
            name: Some(name),
            ..
        } if name == target_name => Some(*id),
        _ => None,
    })
}

pub(crate) fn input_has_focus(elements: &[Element], target_name: &str, expected: bool) -> bool {
    walk(elements, &|element| match element {
        Element::Input {
            name: Some(name),
            focused,
            ..
        } if name == target_name => *focused == expected,
        _ => false,
    })
}

pub(crate) fn assert_has_bounds(bounds: Option<ElementBounds>) {
    let bounds = bounds.expect("element bounds");
    assert!(bounds.top_left.0 <= bounds.bottom_right.0);
    assert!(bounds.top_left.1 <= bounds.bottom_right.1);
}

pub(crate) fn find<T>(
    elements: &[Element],
    predicate: &dyn Fn(&Element) -> Option<T>,
) -> Option<T> {
    for element in elements {
        if let Some(found) = predicate(element) {
            return Some(found);
        }
        match element {
            Element::Table { headers, rows, .. } => {
                for header in headers {
                    if let Some(found) = find(&header.children, predicate) {
                        return Some(found);
                    }
                }
                for row in rows {
                    for cell in &row.cells {
                        if let Some(found) = find(&cell.children, predicate) {
                            return Some(found);
                        }
                    }
                }
            }
            Element::List { items, .. } => {
                for item in items {
                    if let Some(found) = find(&item.children, predicate) {
                        return Some(found);
                    }
                }
            }
            Element::Nav { children, .. }
            | Element::Form { children, .. }
            | Element::Section { children, .. }
            | Element::Iframe { children, .. } => {
                if let Some(found) = find(children, predicate) {
                    return Some(found);
                }
            }
            Element::Heading { .. }
            | Element::Text { .. }
            | Element::Link { .. }
            | Element::Button { .. }
            | Element::Input { .. }
            | Element::Image { .. } => {}
        }
    }
    None
}

fn walk(elements: &[Element], predicate: &dyn Fn(&Element) -> bool) -> bool {
    find(elements, &|element| predicate(element).then_some(())).is_some()
}

#[test]
fn shared_browser_test_timeout_allows_slow_ci_startup() {
    assert!(BROWSER_TEST_TIMEOUT >= Duration::from_secs(60));
}
