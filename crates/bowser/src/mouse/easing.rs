//! Pointer easing functions.

pub(super) fn ease_out(progress: f64) -> f64 {
    let inverse = 1.0 - progress;
    1.0 - inverse * inverse * inverse
}

pub(super) fn ease_in_out(progress: f64) -> f64 {
    if progress < 0.5 {
        4.0 * progress * progress * progress
    } else {
        let offset = -2.0 * progress + 2.0;
        1.0 - offset * offset * offset / 2.0
    }
}
