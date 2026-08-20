//! Bowser CLI.

mod args;
mod cli;
mod commands;
mod dispatch;
mod error;
mod format;
mod output;
mod url;

pub use self::args::{
    CaptureArgs, DescribeArgs, DownloadArgs, ElementActionArgs, ExpandArgs, GetArgs, HistoryArgs,
    InteractiveArgs, KeyArgs, MetaArgs, PageSubcommand, PointerLogArgs, ScrollArgs,
    SessionSubcommand, TypeArgs,
};
pub use self::format::{PageFormat, StructuredFormat};

#[tokio::main]
async fn main() {
    let exit_code = dispatch::run().await;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

#[cfg(test)]
#[path = "_tests_/main_tests.rs"]
mod main_tests;
