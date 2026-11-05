//! Browser-backed transient favicon inventory regressions.

mod cleanup_tests;
mod discovery_tests;
mod images;
mod race_tests;
mod session_tests;
mod support;

#[path = "../support/mod.rs"]
#[expect(
    dead_code,
    reason = "the shared browser fixture module contains routes outside this focused test binary"
)]
mod browser_support;
