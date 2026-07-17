use super::{CursorPosition, ViewportArea, click_hold_ms, initial_cursor_position, movement_steps};

#[test]
fn first_click_route_starts_from_in_viewport_idle_position() {
    let viewport = ViewportArea {
        width: 1280.0,
        height: 800.0,
    };
    let target = CursorPosition { x: 640.0, y: 320.0 };
    let start = initial_cursor_position(viewport, 7, target);
    let steps = movement_steps(start, target, viewport, 7, true);
    assert!(start.x >= 0.0 && start.x < viewport.width);
    assert!(start.y >= 0.0 && start.y < viewport.height);
    assert!(steps.len() >= 18);
    assert!(
        steps
            .iter()
            .map(|step| step.pause_ms)
            .collect::<Vec<_>>()
            .windows(2)
            .any(|window| window[0] != window[1])
    );
    let first = steps.first().expect("first step");
    assert!(first.position.x >= 0.0 && first.position.x < viewport.width);
    assert!(first.position.y >= 0.0 && first.position.y < viewport.height);
    assert!(distance(first.position, start) <= 8.0);
    assert!(steps.iter().any(|step| step.pause_ms >= 80));
    assert_eq!(steps.last().expect("last step").position, target);
    assert!(
        steps
            .iter()
            .rev()
            .take(5)
            .any(|step| distance(step.position, target) <= 3.0)
    );
}

#[test]
fn subsequent_click_route_uses_previous_cursor_position() {
    let viewport = ViewportArea {
        width: 1280.0,
        height: 800.0,
    };
    let start = CursorPosition { x: 120.0, y: 160.0 };
    let target = CursorPosition { x: 980.0, y: 480.0 };
    let steps = movement_steps(start, target, viewport, 11, false);
    assert!(steps.len() >= 18);
    let first = steps.first().expect("first step");
    assert!(first.position.x >= 0.0 && first.position.x <= viewport.width);
    assert!(first.position.y >= 0.0 && first.position.y <= viewport.height);
    assert!(distance(first.position, start) <= 6.0);
    assert!(distance(first.position, start) < distance(target, start));
    assert!(steps.iter().any(|step| step.pause_ms >= 80));
    assert_eq!(steps.last().expect("last step").position, target);
}

#[test]
fn click_hold_duration_matches_fixture_range() {
    let viewport = ViewportArea {
        width: 1280.0,
        height: 800.0,
    };
    let target = CursorPosition { x: 640.0, y: 320.0 };
    let hold_ms = click_hold_ms(7, target, viewport);
    assert!((56..=88).contains(&hold_ms));
}

fn distance(left: CursorPosition, right: CursorPosition) -> f64 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    (dx * dx + dy * dy).sqrt()
}
