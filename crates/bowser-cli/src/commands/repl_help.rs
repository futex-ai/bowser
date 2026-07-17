//! Interactive REPL help text.

use std::fmt::Write;

struct HelpEntry {
    usage: &'static str,
    description: &'static str,
}

/// Canonical REPL command names for completion.
pub const COMMANDS: &[&str] = &[
    "goto",
    "back",
    "forward",
    "reload",
    "pages",
    "page",
    "new",
    "click",
    "clear",
    "close",
    "yaml",
    "keypress",
    "type",
    "select",
    "submit",
    "scroll",
    "screenshot",
    "wait",
    "refresh",
    "expand",
    "meta",
    "describe",
    "session",
    "url",
    "title",
    "html",
    "js",
    "help",
    "quit",
    "exit",
];

const HELP_ENTRIES: &[HelpEntry] = &[
    HelpEntry {
        usage: "goto <URL>",
        description: "Navigate to a new URL",
    },
    HelpEntry {
        usage: "back",
        description: "Go back in browser history",
    },
    HelpEntry {
        usage: "forward",
        description: "Go forward in browser history",
    },
    HelpEntry {
        usage: "reload",
        description: "Reload the current page",
    },
    HelpEntry {
        usage: "pages",
        description: "List known pages in the session and mark the selected one",
    },
    HelpEntry {
        usage: "page <PAGE_ID>",
        description: "Switch to a different live page in the current session",
    },
    HelpEntry {
        usage: "new page [URL]",
        description: "Open a new page, optionally navigating it immediately",
    },
    HelpEntry {
        usage: "close page [PAGE_ID]",
        description: "Close the current page, or close the given live page by ID",
    },
    HelpEntry {
        usage: "yaml [ID]",
        description: "Print the current page, or a captured subtree for the given ID",
    },
    HelpEntry {
        usage: "click <ID>",
        description: "Activate a clickable element",
    },
    HelpEntry {
        usage: "keypress <KEY...>",
        description: "Press one or more space-separated keys together on the focused element",
    },
    HelpEntry {
        usage: "type <ID> <TEXT>",
        description: "Replace an input value by typing text character-by-character",
    },
    HelpEntry {
        usage: "clear <ID>",
        description: "Clear a text-capable input field",
    },
    HelpEntry {
        usage: "select <ID> <VALUE>",
        description: "Select an option in a <select> by visible text",
    },
    HelpEntry {
        usage: "submit <ID>",
        description: "Submit the form containing the given element",
    },
    HelpEntry {
        usage: "scroll <down|up|to ID>",
        description: "Scroll by one viewport, or scroll the given element into view",
    },
    HelpEntry {
        usage: "screenshot [PATH]",
        description: "Save a full-page screenshot",
    },
    HelpEntry {
        usage: "screenshot <ID> [PATH]",
        description: "Save a screenshot of a specific element",
    },
    HelpEntry {
        usage: "wait <SELECTOR>",
        description: "Wait for a CSS selector to appear",
    },
    HelpEntry {
        usage: "refresh",
        description: "Re-capture and print the current page",
    },
    HelpEntry {
        usage: "expand <ID>",
        description: "Print the full subtree for a truncated element",
    },
    HelpEntry {
        usage: "meta <ID>",
        description: "Print metadata and live visibility details for an element",
    },
    HelpEntry {
        usage: "describe <ID>",
        description: "Generate or print a cached AI description for an image",
    },
    HelpEntry {
        usage: "session",
        description: "Print the current Bowser session ID",
    },
    HelpEntry {
        usage: "url",
        description: "Print the current page URL",
    },
    HelpEntry {
        usage: "title",
        description: "Print the current page title",
    },
    HelpEntry {
        usage: "html",
        description: "Print the current page as full rendered HTML",
    },
    HelpEntry {
        usage: "js <EXPRESSION>",
        description: "Execute JavaScript and print the result",
    },
    HelpEntry {
        usage: "help",
        description: "Print command usage with one-line descriptions",
    },
    HelpEntry {
        usage: "quit | exit",
        description: "Exit interactive mode and print the resumable session ID",
    },
];

/// Renders the interactive help text.
pub fn help_text() -> String {
    let width = HELP_ENTRIES
        .iter()
        .map(|entry| entry.usage.len())
        .max()
        .unwrap_or(0);
    let mut output = String::from("Available commands:\n");
    for entry in HELP_ENTRIES {
        let _ = writeln!(
            output,
            "  {usage:width$}  {description}",
            usage = entry.usage,
            width = width,
            description = entry.description
        );
    }
    let _ = write!(
        output,
        "\nID arguments accept either bare numbers like `12` or rendered keys like `input#12`."
    );
    output
}

#[cfg(test)]
#[path = "_tests_/repl_help_tests.rs"]
mod repl_help_tests;
