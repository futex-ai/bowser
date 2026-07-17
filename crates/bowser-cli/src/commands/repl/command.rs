//! REPL command variants.

use std::path::PathBuf;

use bowser::ScrollTarget;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ReplCommand {
    Goto(String),
    Back,
    Forward,
    Reload,
    Pages,
    Page(String),
    NewPage(Option<String>),
    ClosePage(Option<String>),
    Yaml(Option<u32>),
    Click(u32),
    KeyPress(Vec<String>),
    Type(u32, String),
    Clear(u32),
    Select(u32, String),
    Submit(u32),
    Scroll(ScrollTarget),
    Screenshot(Option<u32>, Option<PathBuf>),
    Wait(String),
    Refresh,
    Expand(u32),
    Meta(u32),
    Describe(u32),
    Session,
    Url,
    Title,
    Html,
    Js(String),
    Help,
    Quit,
}
