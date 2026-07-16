//! Interactive command loop and command execution.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use bowser::{Browser, BrowserEngine};
use rustyline::error::{ReadlineError, Signal};

use crate::InteractiveArgs;
use crate::commands::repl::{
    HelperState, ReplCommand, build_editor, parse_command, prompt, update_capture_state,
};
use crate::error::{CliError, Result};
use crate::url::normalize_navigation_target;

use super::capture::print_full_capture;
use super::command_dispatch::dispatch_command;
use super::interrupts::{
    CommandExecution, clear_pending_sigint, install_sigint_handler, is_interrupt,
    run_interruptible, should_exit_on_interrupt,
};
use super::preview::stored_selected_preview;
use super::state::{history_path, refresh_page_state, sync_prompt_state};

const MINIMUM_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const CAPTURE_COMMAND_GRACE: Duration = Duration::from_secs(7);
const PAGE_MANAGEMENT_COMMAND_GRACE: Duration = Duration::from_secs(12);
const WAIT_COMMAND_TIMEOUT: Duration = Duration::from_secs(12);

pub async fn run(config: bowser::BrowserConfig, args: InteractiveArgs) -> Result<()> {
    let browser = Browser::launch(config.clone()).await?;
    let session = browser.session_info().await?;
    let mut page = browser.current_page().await?;
    let helper_state = Arc::new(Mutex::new(HelperState {
        ids: Vec::new(),
        page_ids: Vec::new(),
        current_url: None,
        current_page_id: None,
    }));
    let _sigint_handler = install_sigint_handler()?;
    let mut editor = build_editor(helper_state.clone())?;
    if let Some(path) = history_path(&config) {
        let _ = editor.load_history(&path);
    }

    let should_use_stored_preview = session.resumed && args.url.is_none() && args.wait.is_none();
    if let Some(url) = args.url.as_deref() {
        let url = normalize_navigation_target(url);
        page.navigate(&url).await?;
    }
    if let Some(selector) = args.wait.as_deref() {
        page.wait_for_selector(selector, Duration::from_secs(args.wait_timeout))
            .await?;
    }
    let capture = if should_use_stored_preview {
        stored_selected_preview(&config, &session.id).await?
    } else {
        None
    };
    let capture = match capture {
        Some(capture) => capture,
        None => page.capture().await?,
    };
    update_capture_state(&helper_state, &capture);
    refresh_page_state(&config, &session.id, &helper_state).await?;
    print_full_capture(&capture)?;

    let mut interrupted_once = false;
    loop {
        let current_prompt = prompt(&helper_state);
        match editor.readline(&current_prompt) {
            Ok(line) => {
                interrupted_once = false;
                clear_pending_sigint();
                if line.trim().is_empty() {
                    continue;
                }
                let command = match parse_command(&line) {
                    Ok(command) => command,
                    Err(err @ CliError::InvalidCommand { .. }) => {
                        eprintln!("{err}");
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                if matches!(command, ReplCommand::Quit) {
                    break;
                }
                clear_pending_sigint();
                let command_timeout = command_timeout(config.timeout, &command);
                match run_interruptible(
                    dispatch_command(
                        &browser,
                        &mut page,
                        &config,
                        &session.id,
                        command,
                        &helper_state,
                    ),
                    command_timeout,
                )
                .await
                {
                    CommandExecution::Completed(Ok(())) => {}
                    CommandExecution::Completed(Err(err)) => {
                        sync_prompt_state(&*page, &helper_state).await;
                        eprintln!("{err}");
                    }
                    CommandExecution::Interrupted => {
                        sync_prompt_state(&*page, &helper_state).await;
                        eprintln!("[bowser-cli/repl] command interrupted");
                    }
                    CommandExecution::TimedOut => {
                        sync_prompt_state(&*page, &helper_state).await;
                        eprintln!(
                            "[bowser-cli/repl] command timed out after {}s",
                            command_timeout.as_secs()
                        );
                    }
                }
            }
            Err(err) if is_interrupt(&err) => {
                if should_exit_on_interrupt(&mut interrupted_once) {
                    break;
                }
                println!();
            }
            Err(ReadlineError::Signal(Signal::Resize)) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                return Err(CliError::Readline {
                    reason: err.to_string(),
                });
            }
        }
    }

    if let Some(path) = history_path(&config) {
        let _ = editor.save_history(&path);
    }
    browser.detach().await?;
    eprintln!("Session: {}", session.id);
    Ok(())
}

pub(super) fn command_timeout(config_timeout: Duration, command: &ReplCommand) -> Duration {
    let base_timeout = config_timeout.max(MINIMUM_COMMAND_TIMEOUT);
    match command {
        ReplCommand::Goto(_)
        | ReplCommand::Back
        | ReplCommand::Forward
        | ReplCommand::Reload
        | ReplCommand::Yaml(_)
        | ReplCommand::Click(_)
        | ReplCommand::KeyPress(_)
        | ReplCommand::Type(_, _)
        | ReplCommand::Clear(_)
        | ReplCommand::Select(_, _)
        | ReplCommand::Submit(_)
        | ReplCommand::Refresh => base_timeout + CAPTURE_COMMAND_GRACE,
        ReplCommand::Page(_) | ReplCommand::NewPage(_) | ReplCommand::ClosePage(_) => {
            base_timeout + PAGE_MANAGEMENT_COMMAND_GRACE
        }
        ReplCommand::Wait(_) => base_timeout.max(WAIT_COMMAND_TIMEOUT),
        _ => base_timeout,
    }
}
