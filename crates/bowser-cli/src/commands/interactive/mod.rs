//! Interactive REPL runtime.

mod capture;
mod command_dispatch;
mod execution;
mod interrupts;
mod page_actions;
mod preview;
mod state;

pub use self::execution::run;

#[cfg(test)]
#[path = "_tests_/interactive_tests.rs"]
mod interactive_tests;
