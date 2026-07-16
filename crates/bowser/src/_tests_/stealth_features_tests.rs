use crate::config::BrowserConfig;

use super::{StealthFeature, StealthFeatures};

#[test]
fn default_stealth_features_keep_google_safe_automation() {
    let features = StealthFeatures::from_config(&BrowserConfig::default());

    assert!(features.launch_headed());
    assert!(features.launch_native_window());
    assert!(!features.runtime_disable());
    assert!(!features.accessibility_capture());
    assert!(!features.skip_runtime_stability());
    assert!(features.backend_focus_input());
    assert!(features.enter_submit());
    assert!(features.backend_pointer_target());
}

#[test]
fn disabled_stealth_disables_the_stealth_feature_set() {
    let config = BrowserConfig {
        stealth: false,
        ..BrowserConfig::default()
    };
    let features = StealthFeatures::from_config(&config);

    assert!(
        StealthFeature::ALL
            .iter()
            .all(|feature| !features.enabled(*feature))
    );
}

#[test]
fn disable_runtime_events_only_enables_runtime_disable() {
    let before = StealthFeatures::from_config(&BrowserConfig::default());
    let config = BrowserConfig {
        disable_runtime_events: true,
        ..BrowserConfig::default()
    };
    let after = StealthFeatures::from_config(&config);

    assert_eq!(
        changed_features(before, after),
        vec![StealthFeature::RuntimeDisable]
    );
    assert!(after.runtime_disable());
}

#[test]
fn each_override_enables_exactly_one_feature() {
    for feature in StealthFeature::ALL {
        let before = StealthFeatures::disabled();
        let mut after = before;
        after.apply_override_text(&format!("+{}", feature.id()));

        assert_eq!(changed_features(before, after), vec![feature]);
        assert!(after.enabled(feature), "{} should be enabled", feature.id());
    }
}

#[test]
fn each_override_disables_exactly_one_feature() {
    for feature in StealthFeature::ALL {
        let mut before = StealthFeatures::disabled();
        before.apply_override_text(&format!("{}=on", feature.id()));
        let mut after = before;
        after.apply_override_text(&format!("{}=off", feature.id()));

        assert_eq!(changed_features(before, after), vec![feature]);
        assert!(
            !after.enabled(feature),
            "{} should be disabled",
            feature.id()
        );
    }
}

#[test]
fn overrides_accept_mixed_csv_forms() {
    let mut features = StealthFeatures::disabled();
    features
        .apply_override_text("+launch-headed, runtime-disable=true, accessibility-capture=enabled");

    assert!(features.launch_headed());
    assert!(features.runtime_disable());
    assert!(features.accessibility_capture());
    assert!(!features.launch_native_window());
}

fn changed_features(before: StealthFeatures, after: StealthFeatures) -> Vec<StealthFeature> {
    StealthFeature::ALL
        .iter()
        .copied()
        .filter(|feature| before.enabled(*feature) != after.enabled(*feature))
        .collect()
}
