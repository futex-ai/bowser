//! Documented handwritten test-double exceptions for the Rust trait audit.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ApprovedTestDouble {
    pub(crate) path: &'static str,
    pub(crate) trait_name: &'static str,
    pub(crate) impl_name: &'static str,
    pub(crate) reason: &'static str,
}

pub(crate) const APPROVED_TEST_DOUBLES: &[ApprovedTestDouble] = &[];
