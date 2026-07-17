//! Deferred metadata model types.

mod bounds;
mod build;
mod describable;
mod focus;
mod record;
mod visibility;

pub(crate) use self::build::collect_metadata_records;
pub use self::record::MetadataRecord;
