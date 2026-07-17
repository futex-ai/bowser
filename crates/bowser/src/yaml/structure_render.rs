//! Shared collection and container YAML rendering.

use serde_yaml::{Mapping, Value};

use super::element_render::{render_children, render_element};
use super::value_render::insert_focused;
use crate::model::{Element, ListItem, TableCell, TableRow, TruncationInfo};

pub(super) fn render_table(
    headers: &[TableCell],
    rows: &[TableRow],
    truncation: Option<&TruncationInfo>,
    focused: bool,
) -> Mapping {
    let mut inner = Mapping::new();
    insert_focused(&mut inner, focused);
    inner.insert(
        Value::from("headers"),
        Value::Sequence(headers.iter().map(render_cell_payload).collect()),
    );
    inner.insert(
        Value::from("rows"),
        Value::Sequence(
            rows.iter()
                .map(|row| {
                    let mut row_map = Mapping::new();
                    row_map.insert(
                        Value::from("cells"),
                        Value::Sequence(row.cells.iter().map(render_cell_payload).collect()),
                    );
                    Value::Mapping(row_map)
                })
                .collect(),
        ),
    );
    if let Some(truncation) = truncation {
        inner.insert(Value::from("truncated"), render_truncation(truncation));
    }
    inner
}

pub(super) fn render_list(
    items: &[ListItem],
    truncation: Option<&TruncationInfo>,
    focused: bool,
) -> Value {
    if truncation.is_none() && !focused {
        return Value::Sequence(items.iter().map(render_list_item).collect());
    }
    let mut mapping = Mapping::new();
    insert_focused(&mut mapping, focused);
    mapping.insert(
        Value::from("items"),
        Value::Sequence(items.iter().map(render_list_item).collect()),
    );
    if let Some(truncation) = truncation {
        mapping.insert(Value::from("truncated"), render_truncation(truncation));
    }
    Value::Mapping(mapping)
}

pub(super) fn render_container(
    children: &[Element],
    truncation: Option<&TruncationInfo>,
    action: Option<&str>,
    focused: bool,
) -> Value {
    if truncation.is_none() && action.is_none() && !focused {
        return Value::Sequence(render_children(children));
    }
    let mut mapping = Mapping::new();
    insert_focused(&mut mapping, focused);
    if let Some(action) = action {
        mapping.insert(Value::from("action"), Value::from(action.to_string()));
    }
    mapping.insert(
        Value::from("content"),
        Value::Sequence(render_children(children)),
    );
    if let Some(truncation) = truncation {
        mapping.insert(Value::from("truncated"), render_truncation(truncation));
    }
    Value::Mapping(mapping)
}

pub(super) fn render_iframe(
    src: &str,
    children: &[Element],
    truncation: Option<&TruncationInfo>,
    focused: bool,
) -> Mapping {
    let mut inner = Mapping::new();
    inner.insert(Value::from("src"), Value::from(src.to_string()));
    insert_focused(&mut inner, focused);
    inner.insert(
        Value::from("content"),
        Value::Sequence(render_children(children)),
    );
    if let Some(truncation) = truncation {
        inner.insert(Value::from("truncated"), render_truncation(truncation));
    }
    inner
}

fn render_list_item(item: &ListItem) -> Value {
    render_children_or_content(&item.children)
}

fn render_cell_payload(cell: &TableCell) -> Value {
    render_children_or_content(&cell.children)
}

fn render_children_or_content(children: &[Element]) -> Value {
    if children.len() == 1 {
        render_element(&children[0])
    } else {
        let mut mapping = Mapping::new();
        mapping.insert(
            Value::from("content"),
            Value::Sequence(render_children(children)),
        );
        Value::Mapping(mapping)
    }
}

fn render_truncation(truncation: &TruncationInfo) -> Value {
    let mut inner = Mapping::new();
    inner.insert(Value::from("shown"), Value::from(truncation.shown as i64));
    inner.insert(Value::from("total"), Value::from(truncation.total as i64));
    Value::Mapping(inner)
}
