//! Raw DOM capture value types.

use chromiumoxide::cdp::browser_protocol::page::FrameId;
use serde::Deserialize;

use crate::model::{InputType, ListType, TruncationInfo};

/// Raw capture envelope returned from a single frame context.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RawCaptureEnvelope {
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub body_id: Option<u32>,
    #[serde(default)]
    pub obscured_body_id: Option<u32>,
    #[serde(default)]
    pub content: RawPageContent,
}

/// Raw root content split by visibility before final ID assignment.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct RawPageContent {
    pub visible: Vec<RawElement>,
    pub obscured: Vec<RawElement>,
}

/// Raw captured element shape before cross-frame stitching and final renumbering.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RawElement {
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
        #[serde(default)]
        describable: bool,
        #[serde(default)]
        focused: bool,
    },
    Table {
        id: u32,
        headers: Vec<RawTableCell>,
        rows: Vec<RawTableRow>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    List {
        id: u32,
        list_type: ListType,
        items: Vec<RawListItem>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Nav {
        id: u32,
        children: Vec<RawElement>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Form {
        id: u32,
        action: Option<String>,
        children: Vec<RawElement>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Section {
        id: u32,
        tag: String,
        children: Vec<RawElement>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
    Iframe {
        id: u32,
        src: String,
        children: Vec<RawElement>,
        truncation: Option<TruncationInfo>,
        #[serde(default)]
        focused: bool,
    },
}

/// Captured child-frame ownership keyed by the parent frame and owning iframe ID.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FrameOwnerKey {
    pub parent_frame_id: FrameId,
    pub owner_temp_id: u32,
}

/// Raw list item payload.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RawListItem {
    pub children: Vec<RawElement>,
}

/// Raw table row payload.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RawTableRow {
    pub cells: Vec<RawTableCell>,
}

/// Raw table cell payload.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RawTableCell {
    pub children: Vec<RawElement>,
}
