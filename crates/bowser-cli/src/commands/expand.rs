//! `bowser expand`.

use std::sync::Arc;

use bowser::{FileSessionStore, SessionStore, default_session_dir};

use crate::ExpandArgs;
use crate::commands::{render_element, write_output};
use crate::error::Result;

/// Runs element expansion against stored session state.
pub async fn run(config: &bowser::BrowserConfig, args: ExpandArgs) -> Result<()> {
    let store = store(config).await;
    let session_id = config
        .session
        .id
        .clone()
        .ok_or_else(|| bowser::Error::SessionNotFound {
            session_id: "<missing --session>".to_string(),
        })?;
    let metadata = store.load(&session_id).await?;
    let capture = metadata
        .selected_page()
        .and_then(|page| page.full_capture.clone().or(page.preview_capture.clone()))
        .ok_or(bowser::Error::ExpandNotFound {
            element_id: args.element_id,
        })?;
    let element =
        bowser::expand_element(&capture, args.element_id).ok_or(bowser::Error::ExpandNotFound {
            element_id: args.element_id,
        })?;
    let output = render_element(&element, args.format)?;
    write_output(args.output.as_deref(), &output).await
}

async fn store(config: &bowser::BrowserConfig) -> Arc<dyn SessionStore> {
    Arc::new(FileSessionStore::new(
        config
            .session
            .dir
            .clone()
            .unwrap_or_else(default_session_dir),
    ))
}
