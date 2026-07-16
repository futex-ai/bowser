//! Deferred metadata lookup helpers.

use crate::model::MetadataRecord;

/// Returns metadata for an element ID.
pub fn metadata_for_element(
    metadata: &[MetadataRecord],
    element_id: u32,
) -> Option<MetadataRecord> {
    metadata
        .iter()
        .find(|record| record.element_id() == element_id)
        .cloned()
}
