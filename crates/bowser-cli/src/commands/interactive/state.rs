//! Session-backed prompt state helpers.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bowser::{FileSessionStore, PageEngine, SessionPageSummary, SessionStore, default_session_dir};

use crate::commands::repl::{HelperState, lock_helper_state, update_page_state};
use crate::error::Result;

pub(super) async fn sync_prompt_state(
    page: &dyn PageEngine,
    helper_state: &Arc<Mutex<HelperState>>,
) {
    let url = tokio::time::timeout(Duration::from_secs(1), page.url())
        .await
        .ok()
        .and_then(|result| result.ok());
    let mut state = lock_helper_state(helper_state);
    state.ids.clear();
    if let Some(url) = url {
        state.current_url = Some(url);
    }
}

pub(super) async fn refresh_page_state(
    config: &bowser::BrowserConfig,
    session_id: &str,
    helper_state: &Arc<Mutex<HelperState>>,
) -> Result<()> {
    let pages = load_stored_page_summaries(config, session_id).await?;
    update_page_state(helper_state, &pages);
    Ok(())
}

pub(super) async fn load_stored_page_summaries(
    config: &bowser::BrowserConfig,
    session_id: &str,
) -> Result<Vec<SessionPageSummary>> {
    let store: std::sync::Arc<dyn SessionStore> = std::sync::Arc::new(FileSessionStore::new(
        config
            .session
            .dir
            .clone()
            .unwrap_or_else(default_session_dir),
    ));
    let metadata = store.load(session_id).await?;
    let live_ids: Vec<&str> = metadata.pages.iter().map(|page| page.id.as_str()).collect();
    Ok(metadata.page_summaries(&live_ids))
}

pub(super) fn history_path(config: &bowser::BrowserConfig) -> Option<PathBuf> {
    let root = config.session.dir.clone()?;
    Some(root.join("repl-history.txt"))
}
