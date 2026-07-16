//! Shared finalize state and ID assignment helpers.

use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::page::FrameId;

use super::types::{FrameOwnerKey, RawCaptureEnvelope};
use crate::model::PageCapture;

pub(crate) type IdMappings = HashMap<FrameId, Vec<(u32, u32)>>;

/// Final stitched capture plus temp-to-final ID mappings by frame.
pub(crate) struct FinalizedCapture {
    pub(crate) capture: PageCapture,
    pub(crate) id_mappings: IdMappings,
}

pub(super) fn assign_id(
    frame_id: &FrameId,
    temp_id: u32,
    next_id: &mut u32,
    id_mappings: &mut IdMappings,
) -> u32 {
    let final_id = *next_id;
    *next_id += 1;
    id_mappings
        .entry(frame_id.clone())
        .or_default()
        .push((temp_id, final_id));
    final_id
}

pub(super) fn lookup_child_capture<'a>(
    frame_id: &FrameId,
    raw_id: u32,
    child_frames_by_owner: &HashMap<FrameOwnerKey, FrameId>,
    captures: &'a HashMap<FrameId, RawCaptureEnvelope>,
) -> Option<(FrameId, &'a RawCaptureEnvelope)> {
    let child_frame_id = child_frames_by_owner.get(&FrameOwnerKey {
        parent_frame_id: frame_id.clone(),
        owner_temp_id: raw_id,
    })?;
    captures
        .get(child_frame_id)
        .map(|capture| (child_frame_id.clone(), capture))
}
