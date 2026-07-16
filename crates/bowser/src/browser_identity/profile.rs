//! Coherent browser identity profile application.

use chromiumoxide::Page as ChromiumPage;
use chromiumoxide::cdp::browser_protocol::browser::GetVersionParams;
use chromiumoxide::cdp::browser_protocol::emulation::{
    SetUserAgentOverrideParams, UserAgentBrandVersion, UserAgentMetadata,
};
use serde::Serialize;

use crate::browser_identity::{
    AUTOMATION_CONTROLLED_ARG, HIDE_CRASH_RESTORE_BUBBLE_ARG, init_script,
};
use crate::cdp_trace;
use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct BrowserIdentityProfile {
    pub(crate) user_agent: String,
}

pub(crate) fn stealth_launch_args(enabled: bool) -> Vec<String> {
    if enabled {
        vec![
            HIDE_CRASH_RESTORE_BUBBLE_ARG.to_string(),
            AUTOMATION_CONTROLLED_ARG.to_string(),
        ]
    } else {
        Vec::new()
    }
}

pub(crate) fn desktop_profile(user_agent: String) -> BrowserIdentityProfile {
    BrowserIdentityProfile { user_agent }
}

pub(crate) async fn apply_profile_to_page(
    page: &ChromiumPage,
    profile: &BrowserIdentityProfile,
) -> Result<()> {
    cdp_trace::record_method("Emulation.setUserAgentOverride");
    let user_agent = SetUserAgentOverrideParams::builder()
        .user_agent(profile.user_agent.clone())
        .user_agent_metadata(user_agent_metadata(profile)?)
        .build()
        .map_err(|reason| Error::StealthInjection { reason })?;
    page.execute(user_agent)
        .await
        .map_err(|err| Error::StealthInjection {
            reason: format!("failed to apply user agent profile: {err}"),
        })?;

    Ok(())
}

pub(crate) fn profile_init_script(_profile: &BrowserIdentityProfile) -> Result<String> {
    Ok(init_script().to_string())
}

pub(crate) async fn browser_product_user_agent(page: &ChromiumPage, original: &str) -> String {
    cdp_trace::record_method("Browser.getVersion");
    let product = match page.execute(GetVersionParams {}).await {
        Ok(version) => Some(version.product.clone()),
        Err(_) => None,
    };
    cleaned_user_agent(original, product.as_deref())
}

fn cleaned_user_agent(original: &str, product: Option<&str>) -> String {
    let without_headless = original.replace("HeadlessChrome/", "Chrome/");
    let Some(product_version) = product.and_then(|value| value.strip_prefix("Chrome/")) else {
        return without_headless;
    };
    replace_chrome_version(&without_headless, product_version)
}

fn replace_chrome_version(user_agent: &str, version: &str) -> String {
    let Some((prefix, suffix)) = user_agent.split_once("Chrome/") else {
        return user_agent.to_string();
    };
    let suffix = suffix
        .split_once(' ')
        .map(|(_, rest)| rest)
        .unwrap_or_default();
    format!("{prefix}Chrome/{version} {suffix}")
}

fn user_agent_metadata(profile: &BrowserIdentityProfile) -> Result<UserAgentMetadata> {
    let (major, full) = chrome_versions(&profile.user_agent);
    UserAgentMetadata::builder()
        .brands([
            UserAgentBrandVersion::new("Chromium", major.clone()),
            UserAgentBrandVersion::new("Google Chrome", major),
            UserAgentBrandVersion::new("Not/A)Brand", "99"),
        ])
        .full_version_lists([
            UserAgentBrandVersion::new("Chromium", full.clone()),
            UserAgentBrandVersion::new("Google Chrome", full),
            UserAgentBrandVersion::new("Not/A)Brand", "99.0.0.0"),
        ])
        .platform(ua_metadata_platform(&profile.user_agent))
        .platform_version(ua_metadata_platform_version(&profile.user_agent))
        .architecture("x86")
        .model("")
        .mobile(false)
        .bitness("64")
        .wow64(false)
        .form_factor("Desktop")
        .build()
        .map_err(|reason| Error::StealthInjection { reason })
}

fn chrome_versions(user_agent: &str) -> (String, String) {
    let full = user_agent
        .split("Chrome/")
        .nth(1)
        .and_then(|value| value.split_whitespace().next())
        .unwrap_or("148.0.0.0")
        .to_string();
    let major = full.split('.').next().unwrap_or("148").to_string();
    (major, full)
}

fn ua_metadata_platform(user_agent: &str) -> &'static str {
    if user_agent.contains("Mac OS X") {
        "macOS"
    } else if user_agent.contains("Windows") {
        "Windows"
    } else if user_agent.contains("Linux") {
        "Linux"
    } else {
        ""
    }
}

fn ua_metadata_platform_version(user_agent: &str) -> &'static str {
    if user_agent.contains("Mac OS X 10_15_7") {
        "10.15.7"
    } else if user_agent.contains("Windows NT 10.0") {
        "10.0.0"
    } else {
        ""
    }
}
