//! Metadata geometry updates.

use crate::model::ElementBounds;

use super::MetadataRecord;

impl MetadataRecord {
    /// Returns a copy with current geometry attached.
    pub fn with_bounds(self, bounds: ElementBounds) -> Self {
        match self {
            Self::Link {
                element_id,
                text,
                href,
                focused,
                visibility,
                ..
            } => Self::Link {
                element_id,
                text,
                href,
                focused,
                visibility,
                bounds: Some(bounds),
            },
            Self::Image {
                element_id,
                alt,
                src,
                description,
                describable,
                focused,
                visibility,
                ..
            } => Self::Image {
                element_id,
                alt,
                src,
                description,
                describable,
                focused,
                visibility,
                bounds: Some(bounds),
            },
            Self::Button {
                element_id,
                text,
                focused,
                visibility,
                ..
            } => Self::Button {
                element_id,
                text,
                focused,
                visibility,
                bounds: Some(bounds),
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
                visibility,
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
                bounds: Some(bounds),
            },
            Self::Table {
                element_id,
                headers,
                rows,
                focused,
                visibility,
                ..
            } => Self::Table {
                element_id,
                headers,
                rows,
                focused,
                visibility,
                bounds: Some(bounds),
            },
            Self::List {
                element_id,
                list_type,
                items,
                focused,
                visibility,
                ..
            } => Self::List {
                element_id,
                list_type,
                items,
                focused,
                visibility,
                bounds: Some(bounds),
            },
            Self::Nav {
                element_id,
                children,
                focused,
                visibility,
                ..
            } => Self::Nav {
                element_id,
                children,
                focused,
                visibility,
                bounds: Some(bounds),
            },
            Self::Form {
                element_id,
                action,
                children,
                focused,
                visibility,
                ..
            } => Self::Form {
                element_id,
                action,
                children,
                focused,
                visibility,
                bounds: Some(bounds),
            },
            Self::Section {
                element_id,
                tag,
                children,
                focused,
                visibility,
                ..
            } => Self::Section {
                element_id,
                tag,
                children,
                focused,
                visibility,
                bounds: Some(bounds),
            },
            Self::Iframe {
                element_id,
                src,
                children,
                focused,
                visibility,
                ..
            } => Self::Iframe {
                element_id,
                src,
                children,
                focused,
                visibility,
                bounds: Some(bounds),
            },
        }
    }
}
