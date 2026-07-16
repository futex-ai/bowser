//! Leaf-value YAML rendering helpers.

use serde_yaml::{Mapping, Value};

use crate::model::{InputType, ListType};

pub(super) fn insert_focused(mapping: &mut Mapping, focused: bool) {
    if focused {
        mapping.insert(Value::from("focused"), Value::from(true));
    }
}

pub(super) fn render_input_map(
    input_type: &InputType,
    name: Option<&str>,
    placeholder: Option<&str>,
    value: &str,
    label: Option<&str>,
    options: &[String],
    focused: Option<bool>,
) -> Mapping {
    let mut inner = Mapping::new();
    inner.insert(
        Value::from("type"),
        Value::from(input_type_name(input_type)),
    );
    if let Some(name) = name {
        inner.insert(Value::from("name"), Value::from(name.to_string()));
    }
    if let Some(placeholder) = placeholder {
        inner.insert(
            Value::from("placeholder"),
            Value::from(placeholder.to_string()),
        );
    }
    inner.insert(Value::from("value"), Value::from(value.to_string()));
    if let Some(label) = label {
        inner.insert(Value::from("label"), Value::from(label.to_string()));
    }
    if !options.is_empty() {
        inner.insert(
            Value::from("options"),
            Value::Sequence(options.iter().cloned().map(Value::from).collect()),
        );
    }
    if matches!(focused, Some(true)) {
        inner.insert(Value::from("focused"), Value::from(true));
    }
    inner
}

pub(super) fn render_text_leaf(text: &str, focused: bool) -> Value {
    if !focused {
        return Value::from(text.to_string());
    }
    let mut inner = Mapping::new();
    inner.insert(Value::from("text"), Value::from(text.to_string()));
    inner.insert(Value::from("focused"), Value::from(true));
    Value::Mapping(inner)
}

pub(super) fn render_image_leaf(label: &str, focused: bool, describable: bool) -> Value {
    if !focused && !describable {
        return Value::from(label.to_string());
    }
    let mut inner = Mapping::new();
    inner.insert(Value::from("label"), Value::from(label.to_string()));
    if describable {
        inner.insert(Value::from("describable"), Value::from(true));
    }
    if focused {
        inner.insert(Value::from("focused"), Value::from(true));
    }
    Value::Mapping(inner)
}

pub(super) fn list_key(list_type: &ListType) -> &'static str {
    match list_type {
        ListType::Ordered => "ol",
        ListType::Unordered => "ul",
        ListType::Description => "dl",
    }
}

pub(super) fn input_type_name(input_type: &InputType) -> String {
    match input_type {
        InputType::Text => "text".to_string(),
        InputType::Password => "password".to_string(),
        InputType::Email => "email".to_string(),
        InputType::Number => "number".to_string(),
        InputType::Tel => "tel".to_string(),
        InputType::Url => "url".to_string(),
        InputType::Search => "search".to_string(),
        InputType::Textarea => "textarea".to_string(),
        InputType::Select => "select".to_string(),
        InputType::Checkbox => "checkbox".to_string(),
        InputType::Radio => "radio".to_string(),
        InputType::Date => "date".to_string(),
        InputType::File => "file".to_string(),
        InputType::Hidden => "hidden".to_string(),
        InputType::Other(value) => value.clone(),
    }
}
