//! Metadata focus-state updates.

use super::MetadataRecord;

impl MetadataRecord {
    /// Returns a copy with current focus state attached.
    pub fn with_focus(self, focused: bool) -> Self {
        match self {
            Self::Link {
                element_id,
                text,
                href,
                visibility,
                bounds,
                ..
            } => Self::Link {
                element_id,
                text,
                href,
                focused,
                visibility,
                bounds,
            },
            Self::Image {
                element_id,
                alt,
                src,
                description,
                describable,
                visibility,
                bounds,
                ..
            } => Self::Image {
                element_id,
                alt,
                src,
                description,
                describable,
                focused,
                visibility,
                bounds,
            },
            Self::Button {
                element_id,
                text,
                visibility,
                bounds,
                ..
            } => Self::Button {
                element_id,
                text,
                focused,
                visibility,
                bounds,
            },
            Self::Input {
                element_id,
                name,
                input_type,
                placeholder,
                value,
                label,
                options,
                visibility,
                bounds,
                ..
            } => Self::Input {
                element_id,
                name,
                input_type,
                placeholder,
                value,
                label,
                options,
                focused,
                visibility,
                bounds,
            },
            Self::Table {
                element_id,
                headers,
                rows,
                visibility,
                bounds,
                ..
            } => Self::Table {
                element_id,
                headers,
                rows,
                focused,
                visibility,
                bounds,
            },
            Self::List {
                element_id,
                list_type,
                items,
                visibility,
                bounds,
                ..
            } => Self::List {
                element_id,
                list_type,
                items,
                focused,
                visibility,
                bounds,
            },
            Self::Nav {
                element_id,
                children,
                visibility,
                bounds,
                ..
            } => Self::Nav {
                element_id,
                children,
                focused,
                visibility,
                bounds,
            },
            Self::Form {
                element_id,
                action,
                children,
                visibility,
                bounds,
                ..
            } => Self::Form {
                element_id,
                action,
                children,
                focused,
                visibility,
                bounds,
            },
            Self::Section {
                element_id,
                tag,
                children,
                visibility,
                bounds,
                ..
            } => Self::Section {
                element_id,
                tag,
                children,
                focused,
                visibility,
                bounds,
            },
            Self::Iframe {
                element_id,
                src,
                children,
                visibility,
                bounds,
                ..
            } => Self::Iframe {
                element_id,
                src,
                children,
                focused,
                visibility,
                bounds,
            },
        }
    }
}
