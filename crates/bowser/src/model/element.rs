//! Captured element model types.

use super::{InputType, ListType, TruncationInfo};

/// A single captured element.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Element {
    Heading {
        level: u8,
        text: String,
    },
    Text {
        text: String,
    },
    Link {
        id: u32,
        text: String,
        href: String,
        #[serde(default)]
        focused: bool,
    },
    Button {
        id: u32,
        text: String,
        #[serde(default)]
        focused: bool,
    },
    Input {
        id: Option<u32>,
        name: Option<String>,
        input_type: InputType,
        placeholder: Option<String>,
        value: String,
        label: Option<String>,
        options: Vec<String>,
        #[serde(default)]
        focused: bool,
    },
    Image {
        id: u32,
        alt: String,
        src: String,
        description: Option<String>,
        #[serde(default, skip_serializing_if = "is_false")]
        describable: bool,
        #[serde(default)]
        focused: bool,
    },
    Table {
        id: u32,
        headers: Vec<TableCell>,
        rows: Vec<TableRow>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    List {
        id: u32,
        list_type: ListType,
        items: Vec<ListItem>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Nav {
        id: u32,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Form {
        id: u32,
        action: Option<String>,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Section {
        id: u32,
        tag: String,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Iframe {
        id: u32,
        src: String,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
}

impl Element {
    /// Returns the element ID when present.
    pub fn id(&self) -> Option<u32> {
        match self {
            Self::Link { id, .. }
            | Self::Button { id, .. }
            | Self::Image { id, .. }
            | Self::Table { id, .. }
            | Self::List { id, .. }
            | Self::Nav { id, .. }
            | Self::Form { id, .. }
            | Self::Section { id, .. }
            | Self::Iframe { id, .. } => Some(*id),
            Self::Input { id, .. } => *id,
            Self::Heading { .. } | Self::Text { .. } => None,
        }
    }

    /// Returns the captured focus state when available.
    pub fn focused(&self) -> Option<bool> {
        match self {
            Self::Link { focused, .. }
            | Self::Button { focused, .. }
            | Self::Image { focused, .. }
            | Self::Table { focused, .. }
            | Self::List { focused, .. }
            | Self::Nav { focused, .. }
            | Self::Form { focused, .. }
            | Self::Section { focused, .. }
            | Self::Iframe { focused, .. } => Some(*focused),
            Self::Input { id, focused, .. } => (*id).map(|_| *focused),
            Self::Heading { .. } | Self::Text { .. } => None,
        }
    }

    /// Returns true when the element can be expanded.
    pub fn is_expandable(&self) -> bool {
        match self {
            Self::Table { truncation, .. }
            | Self::List { truncation, .. }
            | Self::Nav { truncation, .. }
            | Self::Form { truncation, .. }
            | Self::Section { truncation, .. }
            | Self::Iframe { truncation, .. } => truncation.is_some(),
            _ => false,
        }
    }

    /// Returns all direct children.
    pub fn children(&self) -> Option<&[Element]> {
        match self {
            Self::Nav { children, .. }
            | Self::Form { children, .. }
            | Self::Section { children, .. }
            | Self::Iframe { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Returns all mutable direct children.
    pub fn children_mut(&mut self) -> Option<&mut Vec<Element>> {
        match self {
            Self::Nav { children, .. }
            | Self::Form { children, .. }
            | Self::Section { children, .. }
            | Self::Iframe { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Returns this element and nested IDs.
    pub fn collect_ids(&self, ids: &mut Vec<u32>) {
        if let Some(id) = self.id() {
            ids.push(id);
        }
        match self {
            Self::Table { headers, rows, .. } => {
                for header in headers {
                    header.collect_ids(ids);
                }
                for row in rows {
                    row.collect_ids(ids);
                }
            }
            Self::List { items, .. } => {
                for item in items {
                    item.collect_ids(ids);
                }
            }
            Self::Nav { children, .. }
            | Self::Form { children, .. }
            | Self::Section { children, .. }
            | Self::Iframe { children, .. } => {
                for child in children {
                    child.collect_ids(ids);
                }
            }
            Self::Heading { .. }
            | Self::Text { .. }
            | Self::Link { .. }
            | Self::Button { .. }
            | Self::Input { .. }
            | Self::Image { .. } => {}
        }
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// A list item.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ListItem {
    pub children: Vec<Element>,
}

impl ListItem {
    /// Collects nested IDs.
    pub fn collect_ids(&self, ids: &mut Vec<u32>) {
        for child in &self.children {
            child.collect_ids(ids);
        }
    }
}

/// A table row.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

impl TableRow {
    /// Collects nested IDs.
    pub fn collect_ids(&self, ids: &mut Vec<u32>) {
        for cell in &self.cells {
            cell.collect_ids(ids);
        }
    }
}

/// A table cell.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TableCell {
    pub children: Vec<Element>,
}

impl TableCell {
    /// Collects nested IDs.
    pub fn collect_ids(&self, ids: &mut Vec<u32>) {
        for child in &self.children {
            child.collect_ids(ids);
        }
    }
}
