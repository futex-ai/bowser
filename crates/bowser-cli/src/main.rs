//! Bowser CLI.

mod args;
mod cli;
mod commands;
mod dispatch;
mod error;
mod format;
mod url;

pub use self::args::{
    CaptureArgs, DescribeArgs, DownloadArgs, ElementActionArgs, ExpandArgs, GetArgs,
    InteractiveArgs, KeyArgs, MetaArgs, PageSubcommand, PointerLogArgs, ScrollArgs,
    SessionSubcommand, TypeArgs,
};
pub use self::format::{PageFormat, StructuredFormat};

#[tokio::main]
async fn main() {
    if let Err(err) = dispatch::run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
#[path = "_tests_/main_tests.rs"]
mod main_tests;
