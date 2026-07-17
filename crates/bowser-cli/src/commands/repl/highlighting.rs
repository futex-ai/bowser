//! REPL highlighting helpers.

use std::borrow::Cow;

use rustyline::highlight::Highlighter;

use super::helper::ReplHelper;

impl Highlighter for ReplHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{hint}\x1b[0m"))
    }
}
