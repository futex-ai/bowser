//! `bowser session`.

use std::sync::Arc;

use bowser::{
    FileSessionStore, SessionStore, cleanup_expired_sessions, default_session_dir,
    remove_owned_session_profile, terminate_session_processes_and_wait,
};

use crate::SessionSubcommand;
use crate::commands::page::render_page_summaries;
use crate::error::Result;

/// Runs session lifecycle commands.
pub async fn run(config: &bowser::BrowserConfig, command: SessionSubcommand) -> Result<()> {
    let session_root = config
        .session
        .dir
        .clone()
        .unwrap_or_else(default_session_dir);
    let store: Arc<dyn SessionStore> = Arc::new(FileSessionStore::new(session_root.clone()));
    match command {
        SessionSubcommand::List => {
            for summary in
                cleanup_expired_sessions(store, &session_root, config.session.idle_ttl).await?
            {
                println!(
                    "{}\t{}\t{}\t{}",
                    summary.id,
                    summary.url.unwrap_or_default(),
                    summary.updated_at,
                    summary.pid
                );
            }
        }
        SessionSubcommand::Info { session_id } => {
            let metadata = store.load(&session_id).await?;
            println!("id: {}", metadata.id);
            println!("http_url: {}", metadata.http_url);
            println!("websocket_url: {}", metadata.websocket_url);
            println!("pid: {}", metadata.pid);
            println!("user_data_dir: {}", metadata.user_data_dir.display());
            println!("updated_at: {}", metadata.updated_at);
            if let Some(page_id) = metadata.selected_page_id.as_deref() {
                println!("selected_page_id: {page_id}");
            }
            if let Some(page) = metadata.summary_page() {
                if let Some(url) = page.url() {
                    println!("url: {url}");
                }
                if let Some(title) = page.title() {
                    println!("title: {title}");
                }
            }
            print!(
                "{}",
                render_page_summaries(&metadata.page_summaries(std::iter::empty::<&str>()))
            );
        }
        SessionSubcommand::Close { session_id } => {
            let metadata = store.load(&session_id).await?;
            terminate_session_processes_and_wait(&metadata).await?;
            remove_owned_session_profile(&session_root, &metadata).await?;
            store.remove(&session_id).await?;
        }
    }
    Ok(())
}
