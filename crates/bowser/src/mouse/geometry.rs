//! Pointer path geometry helpers.

use super::{
    seed::next_range,
    types::{CursorPosition, ViewportArea},
};

pub(super) fn random_viewport_position(viewport: ViewportArea, seed: &mut u64) -> CursorPosition {
    let (min_x, max_x) = viewport_axis_limits(viewport.width);
    let (min_y, max_y) = viewport_axis_limits(viewport.height);
    CursorPosition {
        x: next_range(seed, min_x, max_x),
        y: next_range(seed, min_y, max_y),
    }
}

pub(super) fn mirror_across_viewport(
    target: CursorPosition,
    viewport: ViewportArea,
    seed: &mut u64,
) -> CursorPosition {
    let (min_x, max_x) = viewport_axis_limits(viewport.width);
    let (min_y, max_y) = viewport_axis_limits(viewport.height);
    let x = if target.x < viewport.width * 0.5 {
        next_range(seed, viewport.width * 0.62, max_x)
    } else {
        next_range(seed, min_x, viewport.width * 0.38)
    };
    let y = if target.y < viewport.height * 0.5 {
        next_range(seed, viewport.height * 0.58, max_y)
    } else {
        next_range(seed, min_y, viewport.height * 0.42)
    };
    clamp_to_viewport(CursorPosition { x, y }, viewport, 0.0)
}

pub(super) fn approach_start(
    start: CursorPosition,
    target: CursorPosition,
    viewport: ViewportArea,
    seed: &mut u64,
) -> CursorPosition {
    let route_distance = distance(start, target);
    let radius = (18.0 + route_distance * 0.04 + next_range(seed, 0.0, 18.0)).clamp(12.0, 74.0);
    let (unit_x, unit_y) = unit_from_to(target, start);
    let perpendicular = next_range(seed, -10.0, 10.0);
    clamp_to_viewport(
        CursorPosition {
            x: target.x + unit_x * radius - unit_y * perpendicular,
            y: target.y + unit_y * radius + unit_x * perpendicular,
        },
        viewport,
        1.0,
    )
}

pub(super) fn settle_start(
    approach_start: CursorPosition,
    target: CursorPosition,
    viewport: ViewportArea,
    seed: &mut u64,
) -> CursorPosition {
    let radius = 5.0 + next_range(seed, 0.0, 8.0);
    let (unit_x, unit_y) = unit_from_to(target, approach_start);
    let perpendicular = next_range(seed, -2.5, 2.5);
    clamp_to_viewport(
        CursorPosition {
            x: target.x + unit_x * radius - unit_y * perpendicular,
            y: target.y + unit_y * radius + unit_x * perpendicular,
        },
        viewport,
        0.0,
    )
}

pub(super) fn curve_control_point(
    start: CursorPosition,
    target: CursorPosition,
    route_distance: f64,
    viewport: ViewportArea,
    seed: &mut u64,
) -> CursorPosition {
    let midpoint = CursorPosition {
        x: lerp(start.x, target.x, 0.52),
        y: lerp(start.y, target.y, 0.52),
    };
    let (unit_x, unit_y) = unit_from_to(start, target);
    let bend = next_range(seed, -0.16, 0.16) * route_distance.min(520.0);
    clamp_to_viewport(
        CursorPosition {
            x: midpoint.x - unit_y * bend,
            y: midpoint.y + unit_x * bend,
        },
        viewport,
        1.0,
    )
}

pub(super) fn quadratic_point(
    start: CursorPosition,
    control: CursorPosition,
    target: CursorPosition,
    progress: f64,
) -> CursorPosition {
    let inverse = 1.0 - progress;
    CursorPosition {
        x: inverse * inverse * start.x
            + 2.0 * inverse * progress * control.x
            + progress * progress * target.x,
        y: inverse * inverse * start.y
            + 2.0 * inverse * progress * control.y
            + progress * progress * target.y,
    }
}

pub(super) fn jittered(
    position: CursorPosition,
    viewport: ViewportArea,
    radius: f64,
    seed: &mut u64,
) -> CursorPosition {
    clamp_to_viewport(
        CursorPosition {
            x: position.x + next_range(seed, -radius, radius),
            y: position.y + next_range(seed, -radius, radius),
        },
        viewport,
        0.0,
    )
}

pub(super) fn clamp_to_viewport(
    position: CursorPosition,
    viewport: ViewportArea,
    margin: f64,
) -> CursorPosition {
    CursorPosition {
        x: clamp(position.x, margin, (viewport.width - margin).max(margin)),
        y: clamp(position.y, margin, (viewport.height - margin).max(margin)),
    }
}

pub(super) fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + (end - start) * progress
}

pub(super) fn distance(left: CursorPosition, right: CursorPosition) -> f64 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    (dx * dx + dy * dy).sqrt()
}

pub(super) fn viewport_diagonal(viewport: ViewportArea) -> f64 {
    (viewport.width * viewport.width + viewport.height * viewport.height).sqrt()
}

fn viewport_axis_limits(limit: f64) -> (f64, f64) {
    let max = (limit - 1.0).max(0.0);
    let margin = (limit * 0.08).clamp(0.0, 48.0).min(max * 0.5);
    (margin, max - margin)
}

fn unit_from_to(start: CursorPosition, target: CursorPosition) -> (f64, f64) {
    let dx = target.x - start.x;
    let dy = target.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f64::EPSILON {
        (1.0, 0.0)
    } else {
        (dx / length, dy / length)
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}
