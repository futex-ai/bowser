//! DOM capture script assembly.

const CAPTURE_SCRIPT_TEMPLATE: &str = include_str!("script.js");

/// Builds the DOM capture script.
pub(crate) fn build_capture_script(include_hidden: bool) -> String {
    let include_hidden = if include_hidden { "true" } else { "false" };
    CAPTURE_SCRIPT_TEMPLATE.replace("__BOWSER_INCLUDE_HIDDEN__", include_hidden)
}
