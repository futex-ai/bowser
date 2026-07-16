//! REPL inline hint helpers.

use rustyline::Context;
use rustyline::hint::Hinter;

use super::completion::suggestions;
use super::helper::{ReplHelper, lock_helper_state};

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        let state = lock_helper_state(&self.state);
        if let Some(hint) = inline_hint(line, pos, &state.ids, &state.page_ids) {
            return Some(hint);
        }
        self.history.hint(line, pos, ctx)
    }
}

pub(crate) fn inline_hint(
    line: &str,
    pos: usize,
    ids: &[u32],
    page_ids: &[String],
) -> Option<String> {
    if line.is_empty() || pos != line.len() || current_token(line).is_none_or(str::is_empty) {
        return None;
    }
    let token = current_token(line).unwrap_or_default();
    suggestions(line, ids, page_ids)
        .into_iter()
        .find_map(|candidate| {
            candidate
                .strip_prefix(token)
                .map(ToString::to_string)
                .filter(|hint| !hint.is_empty())
        })
}

fn current_token(line: &str) -> Option<&str> {
    if line.ends_with(' ') {
        return None;
    }
    Some(
        line.rsplit_once(char::is_whitespace)
            .map_or(line, |(_, token)| token),
    )
}
