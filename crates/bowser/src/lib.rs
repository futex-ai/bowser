//! Bowser library entry points.

mod ai;
mod browser;
mod browser_identity;
mod capture;
mod cdp_trace;
mod config;
mod error;
mod expand;
mod keyboard;
mod metadata;
mod model;
mod mouse;
mod page;
mod session;
mod stability;
mod stealth_features;
mod yaml;

pub use ai::{
    AnthropicImageSummarizer, ImageFormat, ImageSummarizer, NoopImageSummarizer,
    OllamaImageSummarizer, OpenAiImageSummarizer, build_image_summarizer,
};
pub use browser::{Browser, BrowserEngine};
pub use browser_identity::{
    BrowserIdentitySnapshot, ChromeIdentity, MimeTypesIdentity, PluginIdentity,
    UserAgentBrandIdentity, UserAgentDataIdentity, ViewportIdentity, WebGlIdentity,
    diagnostic_script as browser_identity_diagnostic_script,
};
pub use config::{
    AiConfig, AiProvider, BrowserConfig, ConfigOverrides, OutputConfig, SessionConfig,
    default_config_path, load_config,
};
pub use error::{Error, Result};
pub use expand::{expand_element, find_element, find_element_mut};
pub use metadata::metadata_for_element;
pub use model::{
    DownloadResult, Element, ElementBounds, ElementVisibility, ImageDescription, InputType,
    ListItem, ListType, MetadataRecord, PageCapture, PageContent, ScrollTarget, SessionInfo,
    SessionPageSummary, SessionPageType, SessionSummary, TableCell, TableRow, TruncationInfo,
    Viewport, image_filename,
};
pub use page::PageEngine;
pub use session::{
    FileSessionStore, SessionMetadata, SessionPageMetadata, SessionStore, cleanup_expired_sessions,
    default_session_dir, terminate_process, terminate_session_processes,
};
pub use yaml::{
    from_json, image_description_to_yaml, metadata_to_yaml, to_json, to_yaml, to_yaml_element,
};
