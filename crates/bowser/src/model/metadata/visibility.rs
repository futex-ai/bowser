//! Metadata visibility updates.

use crate::model::ElementVisibility;

use super::MetadataRecord;

impl MetadataRecord {
    /// Returns a copy with current visibility details attached.
    pub fn with_visibility(self, visibility: ElementVisibility) -> Self {
        match self {
            Self::Link {
                element_id,
                text,
                href,
                focused,
                bounds,
                ..
            } => Self::Link {
                element_id,
                text,
                href,
                focused,
                visibility: Some(visibility),
                bounds,
            },
            Self::Image {
                element_id,
                alt,
                src,
                description,
                describable,
                focused,
                bounds,
                ..
            } => Self::Image {
                element_id,
                alt,
                src,
                description,
                describable,
                focused,
                visibility: Some(visibility),
                bounds,
            },
            Self::Button {
                element_id,
                text,
                focused,
                bounds,
                ..
            } => Self::Button {
                element_id,
                text,
                focused,
                visibility: Some(visibility),
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
                focused,
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
                visibility: Some(visibility),
                bounds,
            },
            Self::Table {
                element_id,
                headers,
                rows,
                focused,
                bounds,
                ..
            } => Self::Table {
                element_id,
                headers,
                rows,
                focused,
                visibility: Some(visibility),
                bounds,
            },
            Self::List {
                element_id,
                list_type,
                items,
                focused,
                bounds,
                ..
            } => Self::List {
                element_id,
                list_type,
                items,
                focused,
                visibility: Some(visibility),
                bounds,
            },
            Self::Nav {
                element_id,
                children,
                focused,
                bounds,
                ..
            } => Self::Nav {
                element_id,
                children,
                focused,
                visibility: Some(visibility),
                bounds,
            },
            Self::Form {
                element_id,
                action,
                children,
                focused,
                bounds,
                ..
            } => Self::Form {
                element_id,
                action,
                children,
                focused,
                visibility: Some(visibility),
                bounds,
            },
            Self::Section {
                element_id,
                tag,
                children,
                focused,
                bounds,
                ..
            } => Self::Section {
                element_id,
                tag,
                children,
                focused,
                visibility: Some(visibility),
                bounds,
            },
            Self::Iframe {
                element_id,
                src,
                children,
                focused,
                bounds,
                ..
            } => Self::Iframe {
                element_id,
                src,
                children,
                focused,
                visibility: Some(visibility),
                bounds,
            },
        }
    }
}
