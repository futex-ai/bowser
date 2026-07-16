//! Browser engine traits, state, and lifecycle entrypoints.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    config::BrowserConfig,
    error::Result,
    model::{SessionInfo, SessionPageSummary},
    page::PageEngine,
    session::{
        FileSessionStore, SessionStore, cleanup_expired_sessions, default_session_dir,
        terminate_session_processes,
    },
};

use super::{
    lifecycle::{
        allocate_port, connect_browser, launch_chrome, resolve_chrome_path, resolve_user_data_dir,
        validate_session_age,
    },
    state::BrowserState,
};

/// Browser trait boundary.
#[async_trait]
pub trait BrowserEngine: Send + Sync {
    /// Returns metadata for the current live or resumed browser session.
    async fn session_info(&self) -> Result<SessionInfo>;
    /// Reattaches the currently selected page as a page engine.
    async fn current_page(&self) -> Result<Box<dyn PageEngine>>;
    /// Lists the known live and stored page summaries for the session.
    async fn list_pages(&self) -> Result<Vec<SessionPageSummary>>;
    /// Selects one known page by Bowser page identifier.
    async fn select_page(&self, page_id: &str) -> Result<Box<dyn PageEngine>>;
    /// Opens a new page and returns it as the selected page engine.
    async fn new_page(&self, url: Option<&str>) -> Result<Box<dyn PageEngine>>;
    /// Closes the selected page, or the provided page id, and returns the next selected page.
    async fn close_page(&self, page_id: Option<&str>) -> Result<Box<dyn PageEngine>>;
    /// Detaches from the live browser process while keeping the session resumable.
    async fn detach(&self) -> Result<()>;
    /// Closes the browser process and removes persisted session metadata.
    async fn close(&self) -> Result<()>;
}

/// Live browser session.
pub struct Browser {
    pub(super) session_info: SessionInfo,
    pub(super) config: BrowserConfig,
    pub(super) store: Arc<dyn SessionStore>,
    pub(super) inner: Mutex<BrowserState>,
}

impl Browser {
    /// Launches or resumes a Bowser browser session.
    pub async fn launch(config: BrowserConfig) -> Result<Self> {
        let session_root = config
            .session
            .dir
            .clone()
            .unwrap_or_else(default_session_dir);
        let store: Arc<dyn SessionStore> = Arc::new(FileSessionStore::new(session_root.clone()));
        Self::launch_with_store(config, store).await
    }

    /// Launches or resumes a Bowser browser session using a caller-provided session store.
    pub async fn launch_with_store(
        config: BrowserConfig,
        store: Arc<dyn SessionStore>,
    ) -> Result<Self> {
        let session_root = config
            .session
            .dir
            .clone()
            .unwrap_or_else(default_session_dir);
        let _ = cleanup_expired_sessions(store.clone(), config.session.idle_ttl).await;

        let (session_info, browser, handler_task, metadata) =
            if let Some(session_id) = config.session.id.clone() {
                let metadata = store.load(&session_id).await?;
                validate_session_age(&metadata, config.session.idle_ttl)?;
                let (browser, handler_task) = connect_browser(&metadata.http_url).await?;
                (
                    SessionInfo {
                        id: session_id,
                        resumed: true,
                        selected_page_id: metadata.selected_page_id.clone(),
                    },
                    browser,
                    handler_task,
                    metadata,
                )
            } else {
                let chrome_path = resolve_chrome_path(config.chrome_path.as_deref())?;
                let user_data_dir = resolve_user_data_dir(&config, &session_root);
                std::fs::create_dir_all(&user_data_dir)
                    .map_err(|err| crate::error::Error::io("create user data directory", err))?;
                let port = allocate_port()?;
                let (pid, http_url, websocket_url, xvfb) =
                    launch_chrome(&config, &chrome_path, &user_data_dir, port).await?;
                let mut metadata = crate::session::SessionMetadata::new(
                    http_url.clone(),
                    websocket_url,
                    pid,
                    user_data_dir,
                );
                if let Some(xvfb) = xvfb {
                    metadata.xvfb_pid = Some(xvfb.pid);
                    metadata.xvfb_display = Some(xvfb.display);
                }
                let session_id = metadata.id.clone();
                store.save(&metadata).await?;
                let (browser, handler_task) = connect_browser(&http_url).await?;
                (
                    SessionInfo {
                        id: session_id,
                        resumed: false,
                        selected_page_id: metadata.selected_page_id.clone(),
                    },
                    browser,
                    handler_task,
                    metadata,
                )
            };

        Ok(Self {
            session_info,
            config,
            store,
            inner: Mutex::new(BrowserState {
                browser,
                handler_task,
                metadata,
            }),
        })
    }
}

#[async_trait]
impl BrowserEngine for Browser {
    async fn session_info(&self) -> Result<SessionInfo> {
        let state = self.inner.lock().await;
        let mut info = self.session_info.clone();
        info.selected_page_id = state.metadata.selected_page_id.clone();
        Ok(info)
    }

    async fn current_page(&self) -> Result<Box<dyn PageEngine>> {
        self.current_page_impl().await
    }

    async fn list_pages(&self) -> Result<Vec<SessionPageSummary>> {
        self.list_pages_impl().await
    }

    async fn select_page(&self, page_id: &str) -> Result<Box<dyn PageEngine>> {
        self.select_page_impl(page_id).await
    }

    async fn new_page(&self, url: Option<&str>) -> Result<Box<dyn PageEngine>> {
        self.new_page_impl(url).await
    }

    async fn close_page(&self, page_id: Option<&str>) -> Result<Box<dyn PageEngine>> {
        self.close_page_impl(page_id).await
    }

    async fn detach(&self) -> Result<()> {
        let mut state = self.inner.lock().await;
        let mut metadata = self
            .store
            .load(&state.metadata.id)
            .await
            .unwrap_or_else(|_| state.metadata.clone());
        metadata.updated_at = chrono::Utc::now();
        metadata.mark_all_pages_stale();
        self.store.save(&metadata).await?;
        state.metadata = metadata;
        state.handler_task.abort();
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        let mut state = self.inner.lock().await;
        let _ = tokio::time::timeout(Duration::from_secs(5), state.browser.close()).await;
        state.handler_task.abort();
        terminate_session_processes(&state.metadata);
        self.store.remove(&state.metadata.id).await?;
        Ok(())
    }
}
