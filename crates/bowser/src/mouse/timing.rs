//! Pointer movement timing helpers.

use super::{
    seed::{initial_seed, next_index},
    types::{CursorPosition, ViewportArea},
};

/// Builds a seeded mouse-button hold duration.
pub(crate) fn click_hold_ms(
    element_id: u32,
    target: CursorPosition,
    viewport: ViewportArea,
) -> u64 {
    let mut seed = initial_seed(element_id, target, viewport) ^ 0x6E63_1E6D_94A4_A0B5;
    56 + next_index(&mut seed, 33) as u64
}

pub(super) fn reaction_pause_ms(entering_viewport: bool, seed: &mut u64) -> u64 {
    if entering_viewport {
        92 + next_index(seed, 190) as u64
    } else {
        32 + next_index(seed, 150) as u64
    }
}

pub(super) fn hesitation_pause_ms(seed: &mut u64) -> u64 {
    112 + next_index(seed, 180) as u64
}

pub(super) fn secondary_hesitation_pause_ms(seed: &mut u64) -> u64 {
    74 + next_index(seed, 120) as u64
}

pub(super) fn frame_pause_ms(seed: &mut u64) -> u64 {
    match next_index(seed, 100) {
        0..=54 => 8 + next_index(seed, 2) as u64,
        55..=83 => 15 + next_index(seed, 3) as u64,
        84..=93 => 20 + next_index(seed, 13) as u64,
        _ => 4 + next_index(seed, 4) as u64,
    }
}

pub(super) fn final_settle_pause_ms(seed: &mut u64) -> u64 {
    18 + next_index(seed, 50) as u64
}
