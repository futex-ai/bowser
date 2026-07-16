//! Element-level YAML rendering.

use serde_yaml::{Mapping, Sequence, Value};

use super::structure_render::{render_container, render_iframe, render_list, render_table};
use super::value_render::{list_key, render_image_leaf, render_input_map, render_text_leaf};
use crate::model::Element;
use crate::model::image_label;

pub(super) fn render_children(children: &[Element]) -> Sequence {
    children.iter().map(render_element).collect()
}

pub(super) fn render_element(element: &Element) -> Value {
    let mut mapping = Mapping::new();
    match element {
        Element::Heading { level, text } => {
            mapping.insert(Value::from(format!("h{level}")), Value::from(text.clone()));
        }
        Element::Text { text } => {
            mapping.insert(Value::from("text"), Value::from(text.clone()));
        }
        Element::Link {
            id, text, focused, ..
        } => {
            mapping.insert(
                Value::from(format!("link#{id}")),
                render_text_leaf(text, *focused),
            );
        }
        Element::Button { id, text, focused } => {
            mapping.insert(
                Value::from(format!("button#{id}")),
                render_text_leaf(text, *focused),
            );
        }
        Element::Input {
            id,
            name,
            input_type,
            placeholder,
            value,
            label,
            options,
            focused,
        } => {
            let key = id.map_or_else(|| "input".to_string(), |id| format!("input#{id}"));
            mapping.insert(
                Value::from(key),
                Value::Mapping(render_input_map(
                    input_type,
                    name.as_deref(),
                    placeholder.as_deref(),
                    value,
                    label.as_deref(),
                    options,
                    id.is_some().then_some(*focused),
                )),
            );
        }
        Element::Image {
            id,
            alt,
            src,
            description,
            describable,
            focused,
        } => {
            mapping.insert(
                Value::from(format!("image#{id}")),
                render_image_leaf(
                    &image_label(alt, src, description.as_deref()),
                    *focused,
                    *describable,
                ),
            );
        }
        Element::Table {
            id,
            headers,
            rows,
            truncation,
            focused,
        } => {
            mapping.insert(
                Value::from(format!("table#{id}")),
                Value::Mapping(render_table(headers, rows, truncation.as_ref(), *focused)),
            );
        }
        Element::List {
            id,
            list_type,
            items,
            truncation,
            focused,
        } => {
            mapping.insert(
                Value::from(format!("{}#{id}", list_key(list_type))),
                render_list(items, truncation.as_ref(), *focused),
            );
        }
        Element::Nav {
            id,
            children,
            truncation,
            focused,
        } => {
            mapping.insert(
                Value::from(format!("nav#{id}")),
                render_container(children, truncation.as_ref(), None, *focused),
            );
        }
        Element::Form {
            id,
            action,
            children,
            truncation,
            focused,
        } => {
            mapping.insert(
                Value::from(format!("form#{id}")),
                render_container(children, truncation.as_ref(), action.as_deref(), *focused),
            );
        }
        Element::Section {
            id,
            tag,
            children,
            truncation,
            focused,
        } => {
            mapping.insert(
                Value::from(format!("{tag}#{id}")),
                render_container(children, truncation.as_ref(), None, *focused),
            );
        }
        Element::Iframe {
            id,
            src,
            children,
            truncation,
            focused,
        } => {
            mapping.insert(
                Value::from(format!("iframe#{id}")),
                Value::Mapping(render_iframe(src, children, truncation.as_ref(), *focused)),
            );
        }
    }
    Value::Mapping(mapping)
}
