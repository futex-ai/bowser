//! REPL completion helpers.

use rustyline::Context;
use rustyline::completion::{Completer, Pair};

use super::helper::{ReplHelper, lock_helper_state};
use crate::commands::repl_help::COMMANDS;

const COMMON_KEYPRESS_KEYS: &[&str] = &[
    "cmd",
    "ctrl",
    "alt",
    "shift",
    "enter",
    "space",
    "tab",
    "escape",
    "up",
    "down",
    "left",
    "right",
    "backspace",
    "delete",
];

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let prefix = &line[..pos];
        let state = lock_helper_state(&self.state);
        let suggestions = suggestions(prefix, &state.ids, &state.page_ids);
        let start = prefix
            .rfind(|ch: char| ch.is_whitespace())
            .map(|index| index + 1)
            .unwrap_or(0);
        Ok((
            start,
            suggestions
                .into_iter()
                .map(|value| Pair {
                    display: value.clone(),
                    replacement: value,
                })
                .collect(),
        ))
    }
}

pub(crate) fn suggestions(line: &str, ids: &[u32], page_ids: &[String]) -> Vec<String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() || (tokens.len() == 1 && !line.ends_with(' ')) {
        let prefix = tokens.first().copied().unwrap_or_default();
        return COMMANDS
            .iter()
            .copied()
            .filter(|command| command.starts_with(prefix))
            .map(ToString::to_string)
            .collect();
    }
    let current = if line.ends_with(' ') {
        ""
    } else {
        tokens.last().copied().unwrap_or_default()
    };
    let command = tokens[0];
    if command == "page" {
        return page_ids
            .iter()
            .filter(|value| value.starts_with(current))
            .cloned()
            .collect();
    }
    if matches!(command, "new" | "close") && tokens.len() <= 2 {
        return ["page"]
            .iter()
            .copied()
            .filter(|value| value.starts_with(current))
            .map(ToString::to_string)
            .collect();
    }
    if command == "close" && tokens.get(1) == Some(&"page") {
        return page_ids
            .iter()
            .filter(|value| value.starts_with(current))
            .cloned()
            .collect();
    }
    let needs_id = matches!(
        command,
        "yaml" | "click" | "type" | "clear" | "select" | "submit" | "expand" | "meta" | "describe"
    ) || (command == "scroll" && tokens.get(1) == Some(&"to"));
    if needs_id {
        return ids
            .iter()
            .map(u32::to_string)
            .filter(|value| value.starts_with(current))
            .collect();
    }
    if command == "keypress" {
        return COMMON_KEYPRESS_KEYS
            .iter()
            .copied()
            .filter(|value| value.starts_with(current))
            .map(ToString::to_string)
            .collect();
    }
    Vec::new()
}
