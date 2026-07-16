//! Browser identity and conservative stealth helpers.

mod profile;
mod scripts;
mod snapshot;

pub(crate) use profile::{
    apply_profile_to_page, browser_product_user_agent, desktop_profile, profile_init_script,
    stealth_launch_args,
};
pub use scripts::diagnostic_script;
pub(crate) use scripts::{init_script, runtime_cleanup_script};
pub use snapshot::{
    BrowserIdentitySnapshot, ChromeIdentity, MimeTypesIdentity, PluginIdentity,
    UserAgentBrandIdentity, UserAgentDataIdentity, ViewportIdentity, WebGlIdentity,
};

/// Chromium launch flag that disables Blink's automation-controlled feature.
pub const AUTOMATION_CONTROLLED_ARG: &str = "--disable-blink-features=AutomationControlled";

/// Chromium launch flag used by the passing gstack-style control setup.
pub const HIDE_CRASH_RESTORE_BUBBLE_ARG: &str = "--hide-crash-restore-bubble";

#[cfg(test)]
#[path = "_tests_/browser_identity_tests.rs"]
mod browser_identity_tests;
