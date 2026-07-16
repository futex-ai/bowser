//! Input and list-kind model types.

/// Input types.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Tel,
    Url,
    Search,
    Textarea,
    Select,
    Checkbox,
    Radio,
    Date,
    File,
    Hidden,
    Other(String),
}

/// List type.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListType {
    Ordered,
    Unordered,
    Description,
}
