//! Bounded transient favicon discovery and normalization.

pub(crate) mod acquisition;
mod candidate;
pub(crate) mod chromium;
mod image;
mod resource;
pub(crate) mod scheduler;
mod svg_guard;
pub(crate) mod transport;
