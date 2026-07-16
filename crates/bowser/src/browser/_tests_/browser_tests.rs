use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use unimock::{MockFn, Unimock, matching};

use crate::config::{BrowserConfig, SessionConfig};
use crate::error::Error;
use crate::session::SessionMetadata;
use crate::session::session_tests::SessionStoreMock;

use super::{Browser, build_chrome_args, owns_user_data_dir, resolve_user_data_dir};

#[test]
fn headed_launch_omits_headless_flags() {
    let config = BrowserConfig {
        headless: false,
        ..BrowserConfig::default()
    };
    let args = build_chrome_args(
        &config,
        PathBuf::from("/tmp/profile").as_path(),
        9222,
        false,
    );
    assert!(!args.iter().any(|arg| arg == "--headless=new"));
    assert!(!args.iter().any(|arg| arg == "--disable-gpu"));
}

#[test]
fn default_stealth_launch_uses_automation_safe_headed_shape() {
    let config = BrowserConfig::default();
    let args = build_chrome_args(
        &config,
        PathBuf::from("/tmp/profile").as_path(),
        9222,
        false,
    );
    assert!(!args.iter().any(|arg| arg == "--headless=new"));
    assert!(!args.iter().any(|arg| arg == "--disable-gpu"));
    assert!(!args.iter().any(|arg| arg.starts_with("--window-size=")));
    assert!(!args.iter().any(|arg| arg.starts_with("--window-position=")));
}

#[test]
fn xvfb_stealth_launch_uses_configured_window_size() {
    let config = BrowserConfig::default();
    let args = build_chrome_args(&config, PathBuf::from("/tmp/profile").as_path(), 9222, true);

    assert!(args.iter().any(|arg| arg == "--window-size=1920,1080"));
    assert!(!args.iter().any(|arg| arg.starts_with("--window-position=")));
}

#[test]
fn stealth_launch_adds_automation_control_flag() {
    let config = BrowserConfig::default();
    let args = build_chrome_args(
        &config,
        PathBuf::from("/tmp/profile").as_path(),
        9222,
        false,
    );
    assert!(
        args.iter()
            .any(|arg| arg == "--disable-blink-features=AutomationControlled")
    );
    assert!(args.iter().any(|arg| arg == "--hide-crash-restore-bubble"));
}

#[test]
fn disabled_stealth_omits_automation_control_flag() {
    let config = BrowserConfig {
        stealth: false,
        ..BrowserConfig::default()
    };
    let args = build_chrome_args(
        &config,
        PathBuf::from("/tmp/profile").as_path(),
        9222,
        false,
    );
    assert!(args.iter().any(|arg| arg == "--headless=new"));
    assert!(
        !args
            .iter()
            .any(|arg| arg == "--disable-blink-features=AutomationControlled")
    );
}

#[test]
fn persistent_profiles_use_a_stable_default_path() {
    let config = BrowserConfig {
        persistent_profile: true,
        ..BrowserConfig::default()
    };
    let root = PathBuf::from("/tmp/bowser-session-root");
    assert_eq!(
        resolve_user_data_dir(&config, &root),
        root.join("profiles").join("default")
    );
}

#[test]
fn explicit_user_data_dir_beats_persistent_profile() {
    let config = BrowserConfig {
        persistent_profile: true,
        user_data_dir: Some(PathBuf::from("/tmp/custom-profile")),
        ..BrowserConfig::default()
    };
    let root = PathBuf::from("/tmp/bowser-session-root");
    assert_eq!(
        resolve_user_data_dir(&config, &root),
        PathBuf::from("/tmp/custom-profile")
    );
}

#[test]
fn only_generated_ephemeral_profiles_are_owned() {
    assert!(owns_user_data_dir(&BrowserConfig::default()));
    assert!(!owns_user_data_dir(&BrowserConfig {
        persistent_profile: true,
        ..BrowserConfig::default()
    }));
    assert!(!owns_user_data_dir(&BrowserConfig {
        user_data_dir: Some(PathBuf::from("/tmp/custom-profile")),
        ..BrowserConfig::default()
    }));
}

#[tokio::test]
async fn launch_with_store_uses_the_provided_session_store() {
    let list_calls = Arc::new(AtomicUsize::new(0));
    let load_calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let store = Arc::new(Unimock::new((
        SessionStoreMock::list.next_call(matching!()).answers_arc({
            let list_calls = list_calls.clone();
            Arc::new(move |_| {
                list_calls.fetch_add(1, Ordering::SeqCst);
                Ok(Vec::<SessionMetadata>::new())
            })
        }),
        SessionStoreMock::load
            .next_call(matching!("bsr_custom"))
            .answers_arc({
                let load_calls = load_calls.clone();
                Arc::new(move |_, session_id: &str| {
                    load_calls
                        .lock()
                        .expect("load calls")
                        .push(session_id.to_string());
                    Err(Error::SessionNotFound {
                        session_id: session_id.to_string(),
                    })
                })
            }),
    )));
    let config = BrowserConfig {
        session: SessionConfig {
            id: Some("bsr_custom".to_string()),
            dir: None,
            idle_ttl: std::time::Duration::from_secs(1800),
        },
        ..BrowserConfig::default()
    };

    let err = match Browser::launch_with_store(config, store.clone()).await {
        Ok(_) => panic!("custom store load should fail"),
        Err(err) => err,
    };

    assert!(matches!(
        err,
        Error::SessionNotFound { session_id } if session_id == "bsr_custom"
    ));
    assert_eq!(list_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        load_calls.lock().expect("load calls").as_slice(),
        ["bsr_custom"]
    );
}
