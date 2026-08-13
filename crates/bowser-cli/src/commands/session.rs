//! `bowser session`.

use std::sync::Arc;

use bowser::{
    Browser, BrowserEngine, FileSessionStore, SessionStore, cleanup_expired_sessions,
    default_session_dir, remove_owned_session_profile, terminate_session_processes_and_wait,
};

use crate::SessionSubcommand;
use crate::commands::page::render_page_summaries;
use crate::error::Result;
use crate::output::{CommandContext, CommandOutput};

/// Runs session lifecycle commands.
pub async fn run(
    config: &bowser::BrowserConfig,
    command: SessionSubcommand,
    context: &mut CommandContext,
) -> Result<CommandOutput> {
    let session_root = config
        .session
        .dir
        .clone()
        .unwrap_or_else(default_session_dir);
    let store: Arc<dyn SessionStore> = Arc::new(FileSessionStore::new(session_root.clone()));
    match command {
        SessionSubcommand::List => {
            let summaries =
                cleanup_expired_sessions(store, &session_root, config.session.idle_ttl).await?;
            let human = summaries
                .iter()
                .map(|summary| {
                    format!(
                        "{}\t{}\t{}\t{}",
                        summary.id,
                        summary.url.clone().unwrap_or_default(),
                        summary.updated_at,
                        summary.pid
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let human = if human.is_empty() {
                human
            } else {
                format!("{human}\n")
            };
            Ok(
                CommandOutput::result(serde_json::json!({ "sessions": summaries }))?
                    .human_stdout(human),
            )
        }
        SessionSubcommand::Info { session_id } => {
            let metadata = store.load(&session_id).await?;
            context.session = Some(metadata.id.clone());
            context.page = metadata.selected_page_id.clone();
            let mut human = format!(
                "id: {}\nhttp_url: {}\nwebsocket_url: {}\npid: {}\nuser_data_dir: {}\nupdated_at: {}\n",
                metadata.id,
                metadata.http_url,
                metadata.websocket_url,
                metadata.pid,
                metadata.user_data_dir.display(),
                metadata.updated_at
            );
            if let Some(page_id) = metadata.selected_page_id.as_deref() {
                human.push_str(&format!("selected_page_id: {page_id}\n"));
            }
            if let Some(page) = metadata.summary_page() {
                if let Some(url) = page.url() {
                    human.push_str(&format!("url: {url}\n"));
                }
                if let Some(title) = page.title() {
                    human.push_str(&format!("title: {title}\n"));
                }
            }
            human.push_str(&render_page_summaries(
                &metadata.page_summaries(std::iter::empty::<&str>()),
            ));
            CommandOutput::result(serde_json::json!({ "session": metadata }))
                .map(|output| output.human_stdout(human))
        }
        SessionSubcommand::Close { session_id } => {
            let metadata = store.load(&session_id).await?;
            context.session = Some(metadata.id.clone());
            context.page = metadata.selected_page_id.clone();
            terminate_session_processes_and_wait(&metadata).await?;
            remove_owned_session_profile(&session_root, &metadata).await?;
            store.remove(&session_id).await?;
            CommandOutput::result(serde_json::json!({ "closed": true }))
        }
        SessionSubcommand::Export { to } => {
            if config.session.id.is_none() {
                return Err(bowser::Error::SessionNotFound {
                    session_id: "<missing --session>".to_string(),
                }
                .into());
            }
            let browser = Browser::launch(config.clone()).await?;
            context.update_session(&browser.session_info().await?);
            let export = browser.export_checkpoint().await;
            let detach = browser.detach().await;
            let checkpoint = export?;
            detach?;
            bowser::write_checkpoint(&to, &checkpoint)?;
            let summary = bowser::CheckpointSummary::from(&checkpoint);
            CommandOutput::result(serde_json::json!({
                "path": to,
                "checkpoint": summary,
            }))
            .map(|output| {
                output.human_stdout(format!(
                    "Exported checkpoint {} ({} cookies, {} origins, {} pages)\n",
                    to.display(),
                    summary.cookies,
                    summary.origins,
                    summary.pages
                ))
            })
        }
        SessionSubcommand::Restore { from } => {
            let checkpoint = bowser::read_checkpoint(&from)?;
            let summary = bowser::CheckpointSummary::from(&checkpoint);
            let browser = Browser::restore(config.clone(), checkpoint).await?;
            context.update_session(&browser.session_info().await?);
            browser.detach().await?;
            CommandOutput::result(serde_json::json!({
                "checkpoint": summary,
                "restored": true,
            }))
            .map(|output| {
                output.human_stdout(format!(
                    "Restored session {} (selected page {})\n",
                    context.session.as_deref().unwrap_or_default(),
                    context.page.as_deref().unwrap_or_default()
                ))
            })
        }
    }
}
