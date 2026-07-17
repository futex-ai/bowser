//! Shared integration-test support for Bowser.

mod config;
mod fixtures;
mod guard;
mod server;

#[allow(unused_imports)]
pub(crate) use self::config::{BROWSER_TEST_TIMEOUT, chrome_path, test_config};
#[allow(unused_imports)]
pub(crate) use self::guard::{BrowserTestGuard, browser_test_guard};
#[allow(unused_imports)]
pub(crate) use self::server::{TestServer, spawn_server};
