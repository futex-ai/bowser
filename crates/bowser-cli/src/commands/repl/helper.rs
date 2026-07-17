//! REPL editor state and prompt helpers.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use bowser::{PageCapture, SessionPageSummary};
use rustyline::hint::HistoryHinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Config, Editor, Helper};

use crate::commands::repl_help::help_text;
use crate::error::{CliError, Result};

pub(crate) struct ReplHelper {
    pub(super) state: Arc<Mutex<HelperState>>,
    pub(super) history: HistoryHinter,
}

impl Helper for ReplHelper {}
impl Validator for ReplHelper {}

#[derive(Clone, Debug)]
pub struct HelperState {
    pub ids: Vec<u32>,
    pub page_ids: Vec<String>,
    pub current_url: Option<String>,
    pub current_page_id: Option<String>,
}

pub(crate) fn build_editor(
    helper_state: Arc<Mutex<HelperState>>,
) -> Result<Editor<ReplHelper, DefaultHistory>> {
    let config = Config::builder().auto_add_history(true).build();
    let mut editor = Editor::<ReplHelper, DefaultHistory>::with_config(config).map_err(|err| {
        CliError::Readline {
            reason: err.to_string(),
        }
    })?;
    editor.set_helper(Some(ReplHelper {
        state: helper_state,
        history: HistoryHinter::new(),
    }));
    Ok(editor)
}

pub(crate) fn lock_helper_state(
    helper_state: &Arc<Mutex<HelperState>>,
) -> MutexGuard<'_, HelperState> {
    match helper_state.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub(crate) fn update_capture_state(helper_state: &Arc<Mutex<HelperState>>, capture: &PageCapture) {
    let mut ids = Vec::new();
    for children in capture.content.buckets() {
        for child in children {
            child.collect_ids(&mut ids);
        }
    }
    let mut state = lock_helper_state(helper_state);
    state.ids = ids;
    state.current_url = Some(capture.url.clone());
}

pub(crate) fn update_page_state(
    helper_state: &Arc<Mutex<HelperState>>,
    pages: &[SessionPageSummary],
) {
    let mut state = lock_helper_state(helper_state);
    state.page_ids = pages.iter().map(|page| page.id.clone()).collect();
    state.current_page_id = pages
        .iter()
        .find(|page| page.selected)
        .map(|page| page.id.clone());
}

pub(crate) fn prompt(helper_state: &Arc<Mutex<HelperState>>) -> (String, String) {
    let url = lock_helper_state(helper_state).current_url.clone();
    let label = prompt_label(url.as_deref());
    let raw = format!("[{label}] bowser> ");
    let styled =
        format!("\x1b[90m[\x1b[0m\x1b[36m{label}\x1b[0m\x1b[90m]\x1b[0m \x1b[32mbowser>\x1b[0m ");
    (raw, styled)
}

pub(crate) fn default_screenshot_path() -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    PathBuf::from(format!("screenshot-{timestamp}.png"))
}

pub(crate) fn print_help() {
    println!("{}", help_text());
}

fn prompt_label(url: Option<&str>) -> String {
    let url = url.unwrap_or("about:blank");
    let stripped = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .trim_end_matches('/');
    if stripped.len() <= 48 {
        if stripped.is_empty() {
            "about:blank".to_string()
        } else {
            stripped.to_string()
        }
    } else {
        format!("{}...", &stripped[..45])
    }
}
