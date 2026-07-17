//! Mouse path generation helpers.

use super::{
    easing::{ease_in_out, ease_out},
    geometry::{
        approach_start, clamp_to_viewport, curve_control_point, distance, jittered, lerp,
        mirror_across_viewport, quadratic_point, random_viewport_position, settle_start,
        viewport_diagonal,
    },
    seed::{initial_seed, next_index},
    timing::{
        final_settle_pause_ms, frame_pause_ms, hesitation_pause_ms, reaction_pause_ms,
        secondary_hesitation_pause_ms,
    },
    types::{CursorPosition, MouseMoveStep, ViewportArea},
};

/// Builds a seeded in-viewport idle position for the first visible click.
pub(crate) fn initial_cursor_position(
    viewport: ViewportArea,
    element_id: u32,
    target: CursorPosition,
) -> CursorPosition {
    let mut seed = initial_seed(element_id, target, viewport);
    let minimum_distance = viewport_diagonal(viewport) * 0.28;
    let mut candidate = random_viewport_position(viewport, &mut seed);
    for _ in 0..6 {
        if distance(candidate, target) >= minimum_distance {
            return candidate;
        }
        candidate = random_viewport_position(viewport, &mut seed);
    }
    mirror_across_viewport(target, viewport, &mut seed)
}

/// Builds a fixture-shaped mouse movement route between two points.
pub(crate) fn movement_steps(
    start: CursorPosition,
    target: CursorPosition,
    viewport: ViewportArea,
    element_id: u32,
    entering_viewport: bool,
) -> Vec<MouseMoveStep> {
    let mut seed =
        initial_seed(element_id, target, viewport) ^ ((start.x.to_bits() << 1) ^ start.y.to_bits());

    let mut steps = Vec::new();
    let current = clamp_to_viewport(start, viewport, 0.0);
    let target = clamp_to_viewport(target, viewport, 0.0);
    let current = add_idle_drift(&mut steps, current, viewport, entering_viewport, &mut seed);
    if distance(current, target) <= 0.5 {
        steps.push(MouseMoveStep {
            position: target,
            pause_ms: final_settle_pause_ms(&mut seed),
        });
        return steps;
    }

    let approach_start = approach_start(current, target, viewport, &mut seed);
    append_coarse_travel(&mut steps, current, approach_start, viewport, &mut seed);
    let settle_start = settle_start(approach_start, target, viewport, &mut seed);
    append_approach(
        &mut steps,
        approach_start,
        settle_start,
        viewport,
        &mut seed,
    );
    append_micro_corrections(&mut steps, settle_start, target, viewport, &mut seed);
    steps
}

fn add_idle_drift(
    steps: &mut Vec<MouseMoveStep>,
    start: CursorPosition,
    viewport: ViewportArea,
    entering_viewport: bool,
    seed: &mut u64,
) -> CursorPosition {
    let drift = if entering_viewport { 5.0 } else { 3.0 };
    let position = jittered(start, viewport, drift, seed);
    steps.push(MouseMoveStep {
        position,
        pause_ms: reaction_pause_ms(entering_viewport, seed),
    });
    position
}

fn append_coarse_travel(
    steps: &mut Vec<MouseMoveStep>,
    start: CursorPosition,
    target: CursorPosition,
    viewport: ViewportArea,
    seed: &mut u64,
) {
    let route_distance = distance(start, target);
    let step_count = ((route_distance / 44.0).round() as usize).clamp(5, 18) + next_index(seed, 5);
    let control = curve_control_point(start, target, route_distance, viewport, seed);
    let first_hesitation = 1 + next_index(seed, (step_count / 3).max(1));
    let second_hesitation = if route_distance > 260.0 && next_index(seed, 100) < 65 {
        Some((step_count * 2 / 3).max(first_hesitation + 1))
    } else {
        None
    };

    for step in 1..=step_count {
        let progress = ease_in_out(step as f64 / step_count as f64);
        let mut point = quadratic_point(start, control, target, progress);
        if step != step_count {
            let jitter = (2.0 + route_distance * 0.008).min(8.5) * (1.0 - progress * 0.55);
            point = jittered(point, viewport, jitter, seed);
        }
        let pause_ms = if step == first_hesitation {
            hesitation_pause_ms(seed)
        } else if Some(step) == second_hesitation {
            secondary_hesitation_pause_ms(seed)
        } else {
            frame_pause_ms(seed)
        };
        steps.push(MouseMoveStep {
            position: point,
            pause_ms,
        });
    }
}

fn append_approach(
    steps: &mut Vec<MouseMoveStep>,
    start: CursorPosition,
    target: CursorPosition,
    viewport: ViewportArea,
    seed: &mut u64,
) {
    let step_count = 7 + next_index(seed, 7);
    for step in 1..=step_count {
        let progress = ease_out(step as f64 / step_count as f64);
        let mut point = CursorPosition {
            x: lerp(start.x, target.x, progress),
            y: lerp(start.y, target.y, progress),
        };
        if step != step_count {
            point = jittered(point, viewport, 2.4 * (1.0 - progress), seed);
        }
        steps.push(MouseMoveStep {
            position: point,
            pause_ms: frame_pause_ms(seed),
        });
    }
}

fn append_micro_corrections(
    steps: &mut Vec<MouseMoveStep>,
    start: CursorPosition,
    target: CursorPosition,
    viewport: ViewportArea,
    seed: &mut u64,
) {
    let step_count = 5 + next_index(seed, 7);
    for step in 1..=step_count {
        let progress = ease_out(step as f64 / step_count as f64);
        let mut point = CursorPosition {
            x: lerp(start.x, target.x, progress),
            y: lerp(start.y, target.y, progress),
        };
        if step != step_count {
            point = jittered(point, viewport, 0.8 * (1.0 - progress), seed);
        } else {
            point = target;
        }
        let pause_ms = if step == step_count {
            final_settle_pause_ms(seed)
        } else {
            frame_pause_ms(seed)
        };
        steps.push(MouseMoveStep {
            position: point,
            pause_ms,
        });
    }
}
