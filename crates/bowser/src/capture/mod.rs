//! DOM capture and truncation.

mod finalize;
mod finalize_collections;
mod finalize_support;
mod images;
mod script;
mod tree;
mod truncation;
mod types;

pub(crate) use finalize::finalize_capture;
pub(crate) use finalize_support::FinalizedCapture;
pub(crate) use images::{
    ImageKey, known_image_descriptions, restore_image_descriptions, set_images_describable,
};
pub(crate) use script::build_capture_script;
pub(crate) use truncation::truncate_capture;
pub(crate) use types::{FrameOwnerKey, RawCaptureEnvelope};

#[cfg(any(test, doctest))]
use truncation::truncate_element;

#[cfg(test)]
#[path = "_tests_/capture_tests.rs"]
mod capture_tests;
