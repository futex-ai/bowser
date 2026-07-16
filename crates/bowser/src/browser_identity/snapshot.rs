//! Browser identity diagnostic data types.

/// Captured browser identity diagnostic record.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrowserIdentitySnapshot {
    /// Browser User-Agent string visible to page JavaScript.
    pub user_agent: String,
    /// User-Agent client hints exposed by `navigator.userAgentData`, when available.
    pub user_agent_data: Option<UserAgentDataIdentity>,
    /// Browser platform string visible to page JavaScript.
    pub platform: String,
    /// Preferred browser language, when available.
    pub language: Option<String>,
    /// Preferred browser languages.
    pub languages: Vec<String>,
    /// Resolved browser timezone, when available.
    pub timezone: Option<String>,
    /// Viewport and screen metrics visible to page JavaScript.
    pub viewport: ViewportIdentity,
    /// WebGL renderer identity, when WebGL is available.
    pub webgl: Option<WebGlIdentity>,
    /// Browser plugins visible to page JavaScript.
    pub plugins: Vec<PluginIdentity>,
    /// Browser MIME types visible to page JavaScript.
    pub mime_types: MimeTypesIdentity,
    /// Chrome-specific global shape visible to page JavaScript.
    pub chrome: ChromeIdentity,
    /// Notification permission state reported by the Permissions API.
    pub notification_permission: Option<String>,
    /// Whether `navigator.webdriver === true`.
    pub webdriver_is_true: bool,
    /// Automation-only globals visible on `window`.
    pub automation_globals: Vec<String>,
}

/// User-Agent client hints visible to page JavaScript.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAgentDataIdentity {
    /// Brand/version entries reported by the browser.
    pub brands: Vec<UserAgentBrandIdentity>,
    /// Whether the browser reports a mobile client hint.
    pub mobile: bool,
    /// Platform reported by the browser client hints.
    pub platform: String,
}

/// User-Agent brand/version entry.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAgentBrandIdentity {
    /// Browser brand.
    pub brand: String,
    /// Browser brand version.
    pub version: String,
}

/// Browser viewport and screen metrics.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewportIdentity {
    /// Window inner width.
    pub inner_width: f64,
    /// Window inner height.
    pub inner_height: f64,
    /// Window outer width.
    pub outer_width: f64,
    /// Window outer height.
    pub outer_height: f64,
    /// Screen width.
    pub screen_width: f64,
    /// Screen height.
    pub screen_height: f64,
    /// Available screen width.
    pub avail_width: f64,
    /// Available screen height.
    pub avail_height: f64,
    /// Device pixel ratio.
    pub device_pixel_ratio: f64,
}

/// WebGL renderer identity.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebGlIdentity {
    /// WebGL vendor string.
    pub vendor: String,
    /// WebGL renderer string.
    pub renderer: String,
}

/// Browser plugin identity.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginIdentity {
    /// Plugin name.
    pub name: String,
    /// Plugin filename.
    pub filename: String,
    /// Plugin description.
    pub description: String,
}

/// Browser MIME type identity.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MimeTypesIdentity {
    /// JavaScript object tag for `navigator.mimeTypes`.
    pub object_tag: String,
    /// Number of MIME types visible to page JavaScript.
    pub length: u32,
}

/// Chrome-specific global identity.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChromeIdentity {
    /// Whether `window.chrome` exists.
    pub present: bool,
    /// Whether `window.chrome.runtime` exists.
    pub runtime_present: bool,
    /// JavaScript type of `window.chrome.runtime`.
    pub runtime_type: String,
    /// Own enumerable key count of `window.chrome.runtime`.
    pub runtime_key_count: u32,
}
