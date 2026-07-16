//! Browser configuration helpers for integration tests.

use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bowser::{BrowserConfig, Viewport};

pub(crate) const BROWSER_TEST_TIMEOUT: Duration = Duration::from_secs(60);

pub fn chrome_path() -> PathBuf {
    if let Ok(path) = env::var("BOWSER_TEST_CHROME_PATH") {
        return PathBuf::from(path);
    }
    let absolute_candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
    ];
    for candidate in absolute_candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return path;
        }
    }
    if let Some(path) = env::var_os("PATH") {
        for root in env::split_paths(&path) {
            for candidate in ["google-chrome", "chromium", "chromium-browser"] {
                let resolved = root.join(candidate);
                if resolved.exists() {
                    return resolved;
                }
            }
        }
    }
    panic!("Chrome or Chromium not found; set BOWSER_TEST_CHROME_PATH");
}

#[allow(dead_code)]
pub fn test_config(session_dir: &Path) -> BrowserConfig {
    let mut config = BrowserConfig {
        chrome_path: Some(chrome_path()),
        chrome_args: vec!["--disable-gpu".to_string()],
        timeout: BROWSER_TEST_TIMEOUT,
        viewport: Viewport {
            width: 1280,
            height: 800,
        },
        stealth: false,
        ..BrowserConfig::default()
    };
    config.session.dir = Some(session_dir.to_path_buf());
    config.ai.enabled = false;
    config
}
