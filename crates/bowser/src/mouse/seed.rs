//! Seeded randomness helpers for pointer movement.

use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{CursorPosition, ViewportArea};

pub(super) fn initial_seed(element_id: u32, target: CursorPosition, viewport: ViewportArea) -> u64 {
    let time_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    mix_seed(
        time_seed
            ^ (u64::from(element_id) << 32)
            ^ target.x.to_bits()
            ^ target.y.to_bits()
            ^ viewport.width.to_bits()
            ^ viewport.height.to_bits(),
    )
}

pub(super) fn next_index(seed: &mut u64, upper: usize) -> usize {
    if upper <= 1 {
        return 0;
    }
    *seed = mix_seed(*seed);
    (*seed % upper as u64) as usize
}

pub(super) fn next_range(seed: &mut u64, min: f64, max: f64) -> f64 {
    *seed = mix_seed(*seed);
    let unit = (*seed as f64) / (u64::MAX as f64);
    min + (max - min) * unit
}

fn mix_seed(seed: u64) -> u64 {
    let mut value = if seed == 0 {
        0xA24B_AED4_0FBF_5A1D
    } else {
        seed
    };
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}
