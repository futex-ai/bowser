//! Core Bowser data model.

mod download;
mod element;
mod favicon;
mod geometry;
mod image;
mod input;
mod metadata;
mod page;
mod session;

pub use self::download::DownloadResult;
pub use self::element::{Element, ListItem, TableCell, TableRow};
pub use self::favicon::{BrowserFavicon, LiveSessionPage};
pub use self::geometry::{ElementBounds, ElementVisibility, ScrollTarget, Viewport};
pub(crate) use self::image::image_label;
pub use self::image::{ImageDescription, image_filename};
pub use self::input::{InputType, ListType};
pub use self::metadata::MetadataRecord;
pub(crate) use self::metadata::collect_metadata_records;
pub use self::page::{PageCapture, PageContent, TruncationInfo};
pub use self::session::{SessionInfo, SessionPageSummary, SessionPageType, SessionSummary};
