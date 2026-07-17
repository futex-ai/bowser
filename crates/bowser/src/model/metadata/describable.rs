//! Metadata image-description capability updates.

use super::MetadataRecord;

impl MetadataRecord {
    /// Returns a copy with current image-description capability attached.
    pub fn with_describable(self, describable: bool) -> Self {
        match self {
            Self::Image {
                element_id,
                alt,
                src,
                description,
                focused,
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
            other => other,
        }
    }
}
