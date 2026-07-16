//! Preview truncation helpers for captured DOM trees.

use crate::{
    config::OutputConfig,
    model::{Element, PageCapture, PageContent, TruncationInfo},
};

/// Applies preview truncation rules.
pub(crate) fn truncate_capture(full: &PageCapture, config: &OutputConfig) -> PageCapture {
    PageCapture {
        url: full.url.clone(),
        title: full.title.clone(),
        body_id: full.body_id,
        obscured_body_id: full.obscured_body_id,
        content: PageContent::new(
            truncate_root_bucket(&full.content.visible, full.body_id, config),
            truncate_root_bucket(&full.content.obscured, full.obscured_body_id, config),
        ),
    }
}

fn truncate_root_bucket(
    children: &[Element],
    body_id: Option<u32>,
    config: &OutputConfig,
) -> Vec<Element> {
    if config.truncate
        && children.len() > config.max_children
        && let Some(body_id) = body_id
    {
        return vec![Element::Section {
            id: body_id,
            tag: "body".to_string(),
            children: truncate_root_children(children, config),
            truncation: Some(TruncationInfo {
                shown: config.max_children,
                total: children.len(),
            }),
            focused: false,
        }];
    }
    truncate_children(children, config)
}

fn truncate_children(children: &[Element], config: &OutputConfig) -> Vec<Element> {
    let mut truncated = recursively_truncated(children, config);
    if config.truncate && truncated.len() > config.max_children {
        let total = truncated.len();
        truncated.truncate(config.max_children);
        if let Some(last) = truncated.last_mut() {
            attach_truncation(last, config.max_children, total);
        }
    }
    truncated
}

fn truncate_root_children(children: &[Element], config: &OutputConfig) -> Vec<Element> {
    let mut truncated = recursively_truncated(children, config);
    truncated.truncate(config.max_children);
    truncated
}

fn recursively_truncated(children: &[Element], config: &OutputConfig) -> Vec<Element> {
    let mut truncated = children.to_vec();
    for child in &mut truncated {
        truncate_element(child, config);
    }
    truncated
}

pub(super) fn truncate_element(element: &mut Element, config: &OutputConfig) {
    match element {
        Element::Table {
            rows, truncation, ..
        } => {
            for row in rows.iter_mut() {
                for cell in &mut row.cells {
                    cell.children = truncate_children(&cell.children, config);
                }
            }
            if config.truncate && rows.len() > config.max_table_rows {
                *truncation = Some(TruncationInfo {
                    shown: config.max_table_rows,
                    total: rows.len(),
                });
                rows.truncate(config.max_table_rows);
            }
        }
        Element::List {
            items, truncation, ..
        } => {
            for item in items.iter_mut() {
                item.children = truncate_children(&item.children, config);
            }
            if config.truncate && items.len() > config.max_list_items {
                *truncation = Some(TruncationInfo {
                    shown: config.max_list_items,
                    total: items.len(),
                });
                items.truncate(config.max_list_items);
            }
        }
        Element::Nav {
            children,
            truncation,
            ..
        }
        | Element::Form {
            children,
            truncation,
            ..
        }
        | Element::Section {
            children,
            truncation,
            ..
        }
        | Element::Iframe {
            children,
            truncation,
            ..
        } => {
            let total = children.len();
            *children = recursively_truncated(children, config);
            if config.truncate && total > config.max_children {
                *truncation = Some(TruncationInfo {
                    shown: config.max_children,
                    total,
                });
                children.truncate(config.max_children);
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

fn attach_truncation(element: &mut Element, shown: usize, total: usize) {
    let info = Some(TruncationInfo { shown, total });
    match element {
        Element::Table { truncation, .. }
        | Element::List { truncation, .. }
        | Element::Nav { truncation, .. }
        | Element::Form { truncation, .. }
        | Element::Section { truncation, .. }
        | Element::Iframe { truncation, .. } => *truncation = info,
        _ => {}
    }
}
