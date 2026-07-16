use std::ffi::OsStr;

use super::support;

#[test]
fn bowser_command_forces_headless_browser_defaults() {
    let command = support::bowser_command();
    let envs: Vec<(&OsStr, Option<&OsStr>)> = command.get_envs().collect();

    assert!(envs.contains(&(OsStr::new("BOWSER_HEADLESS"), Some(OsStr::new("true")))));
    assert!(envs.contains(&(
        OsStr::new("BOWSER_INTERNAL_STEALTH_FEATURES"),
        Some(OsStr::new("-launch-headed,-launch-native-window"))
    )));
}
