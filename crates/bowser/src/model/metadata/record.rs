//! Metadata record variants.

use crate::model::{ElementBounds, ElementVisibility, InputType, ListType};

/// Deferred metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetadataRecord {
    Link {
        element_id: u32,
        text: String,
        href: String,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Image {
        element_id: u32,
        alt: String,
        src: String,
        description: Option<String>,
        #[serde(default, skip_serializing_if = "is_false")]
        describable: bool,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Button {
        element_id: u32,
        text: String,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Input {
        element_id: u32,
        name: Option<String>,
        input_type: InputType,
        placeholder: Option<String>,
        value: String,
        label: Option<String>,
        options: Vec<String>,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Table {
        element_id: u32,
        headers: usize,
        rows: usize,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    List {
        element_id: u32,
        list_type: ListType,
        items: usize,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Nav {
        element_id: u32,
        children: usize,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Form {
        element_id: u32,
        action: Option<String>,
        children: usize,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Section {
        element_id: u32,
        tag: String,
        children: usize,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
    Iframe {
        element_id: u32,
        src: String,
        children: usize,
        #[serde(default)]
        focused: bool,
        #[serde(default)]
        visibility: Option<ElementVisibility>,
        #[serde(default)]
        bounds: Option<ElementBounds>,
    },
}

impl MetadataRecord {
    /// Returns the referenced element ID.
    pub fn element_id(&self) -> u32 {
        match self {
            Self::Link { element_id, .. }
            | Self::Image { element_id, .. }
            | Self::Button { element_id, .. }
            | Self::Input { element_id, .. }
            | Self::Table { element_id, .. }
            | Self::List { element_id, .. }
            | Self::Nav { element_id, .. }
            | Self::Form { element_id, .. }
            | Self::Section { element_id, .. }
            | Self::Iframe { element_id, .. } => *element_id,
        }
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}
