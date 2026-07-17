use super::{DisplayPlan, display_plan};

#[test]
fn headless_never_requires_display_setup() {
    assert_eq!(display_plan(true, None, None, true), DisplayPlan::Headless);
}

#[test]
fn headed_prefers_existing_display() {
    assert_eq!(
        display_plan(false, Some(":7"), None, true),
        DisplayPlan::ExistingDisplay
    );
}

#[test]
fn headed_wayland_does_not_spawn_xvfb() {
    assert_eq!(
        display_plan(false, None, Some("wayland-0"), true),
        DisplayPlan::ExistingWayland
    );
}

#[test]
fn headed_linux_without_display_spawns_xvfb() {
    assert_eq!(
        display_plan(false, None, None, true),
        DisplayPlan::SpawnXvfb
    );
}

#[test]
fn headed_non_linux_without_display_is_unsupported() {
    assert_eq!(
        display_plan(false, None, None, false),
        DisplayPlan::UnsupportedHeaded
    );
}
