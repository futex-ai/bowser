use crate::browser_identity::{
    AUTOMATION_CONTROLLED_ARG, HIDE_CRASH_RESTORE_BUBBLE_ARG, desktop_profile, diagnostic_script,
    init_script, profile_init_script, runtime_cleanup_script, stealth_launch_args,
};

#[test]
fn stealth_launch_args_are_composed_by_identity_module() {
    let args = stealth_launch_args(true);
    assert!(args.contains(&HIDE_CRASH_RESTORE_BUBBLE_ARG.to_string()));
    assert!(args.contains(&AUTOMATION_CONTROLLED_ARG.to_string()));
    assert!(!args.contains(&"--lang=en-GB".to_string()));
    assert!(!args.contains(&"--force-device-scale-factor=1".to_string()));
    assert!(stealth_launch_args(false).is_empty());
}

#[test]
fn identity_scripts_do_not_contain_fixed_spoofed_values() {
    for script in [init_script(), runtime_cleanup_script(), diagnostic_script()] {
        assert!(!script.contains("Intel Inc."));
        assert!(!script.contains("Intel Iris OpenGL Engine"));
        assert!(!script.contains("Chrome PDF Viewer"));
        assert!(!script.contains("Chromium PDF Viewer"));
        assert!(!script.contains("WebKit built-in PDF"));
        assert!(!script.contains("chrome.runtime = {}"));
    }
}

#[test]
fn init_script_keeps_narrow_permissions_patch() {
    let script = init_script();

    assert!(script.contains("window.navigator.permissions.query"));
    assert!(script.contains("parameters.name === 'notifications'"));
    assert!(script.contains("state: 'prompt'"));
}

#[test]
fn desktop_profile_keeps_google_smoke_identity_coherent() {
    let profile = desktop_profile("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36".to_string());

    assert!(!profile.user_agent.contains("HeadlessChrome"));
}

#[test]
fn profile_script_avoids_broad_fingerprint_synthesis() {
    let profile = desktop_profile("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36".to_string());
    let script = profile_init_script(&profile).expect("profile script");

    assert_eq!(script, init_script());
    assert!(!script.contains("Linux x86_64"));
    assert!(!script.contains("1280"));
    assert!(!script.contains("800"));
    assert!(!script.contains("Chrome PDF Viewer"));
    assert!(!script.contains("Microsoft Edge PDF Viewer"));
    assert!(!script.contains("WebKit built-in PDF"));
}
