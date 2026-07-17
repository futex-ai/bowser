//! REPL parsing and editor helpers.

mod arguments;
mod command;
mod completion;
mod helper;
mod highlighting;
mod hints;
mod parsing;

pub(crate) use self::command::ReplCommand;
pub(crate) use self::helper::{
    HelperState, build_editor, default_screenshot_path, lock_helper_state, print_help, prompt,
    update_capture_state, update_page_state,
};
pub(crate) use self::parsing::parse_command;

#[cfg(test)]
#[path = "_tests_/repl_tests.rs"]
mod repl_tests;
