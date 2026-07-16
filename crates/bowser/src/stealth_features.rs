//! Internal stealth experiment feature controls.

use crate::config::BrowserConfig;

pub(crate) const STEALTH_FEATURES_ENV: &str = "BOWSER_INTERNAL_STEALTH_FEATURES";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StealthFeature {
    LaunchHeaded,
    LaunchNativeWindow,
    RuntimeDisable,
    AccessibilityCapture,
    SkipRuntimeStability,
    BackendFocusInput,
    EnterSubmit,
    BackendPointerTarget,
}

impl StealthFeature {
    pub(crate) const ALL: [Self; 8] = [
        Self::LaunchHeaded,
        Self::LaunchNativeWindow,
        Self::RuntimeDisable,
        Self::AccessibilityCapture,
        Self::SkipRuntimeStability,
        Self::BackendFocusInput,
        Self::EnterSubmit,
        Self::BackendPointerTarget,
    ];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::LaunchHeaded => "launch-headed",
            Self::LaunchNativeWindow => "launch-native-window",
            Self::RuntimeDisable => "runtime-disable",
            Self::AccessibilityCapture => "accessibility-capture",
            Self::SkipRuntimeStability => "skip-runtime-stability",
            Self::BackendFocusInput => "backend-focus-input",
            Self::EnterSubmit => "enter-submit",
            Self::BackendPointerTarget => "backend-pointer-target",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|feature| feature.id() == value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StealthFeatures {
    launch_headed: bool,
    launch_native_window: bool,
    runtime_disable: bool,
    accessibility_capture: bool,
    skip_runtime_stability: bool,
    backend_focus_input: bool,
    enter_submit: bool,
    backend_pointer_target: bool,
}

impl StealthFeatures {
    pub(crate) fn from_config(config: &BrowserConfig) -> Self {
        let mut features = if config.stealth {
            Self::default_stealth()
        } else {
            Self::disabled()
        };
        if config.disable_runtime_events {
            features.runtime_disable = true;
        }
        if let Ok(value) = std::env::var(STEALTH_FEATURES_ENV) {
            features.apply_override_text(&value);
        }
        features
    }

    pub(crate) fn disabled() -> Self {
        Self {
            launch_headed: false,
            launch_native_window: false,
            runtime_disable: false,
            accessibility_capture: false,
            skip_runtime_stability: false,
            backend_focus_input: false,
            enter_submit: false,
            backend_pointer_target: false,
        }
    }

    pub(crate) fn default_stealth() -> Self {
        Self {
            launch_headed: true,
            launch_native_window: true,
            runtime_disable: false,
            accessibility_capture: false,
            skip_runtime_stability: false,
            backend_focus_input: true,
            enter_submit: true,
            backend_pointer_target: true,
        }
    }

    pub(crate) fn apply_override_text(&mut self, value: &str) {
        for item in value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
        {
            if let Some((feature, enabled)) = parse_override(item) {
                self.set(feature, enabled);
            }
        }
    }

    pub(crate) fn enabled(self, feature: StealthFeature) -> bool {
        match feature {
            StealthFeature::LaunchHeaded => self.launch_headed,
            StealthFeature::LaunchNativeWindow => self.launch_native_window,
            StealthFeature::RuntimeDisable => self.runtime_disable,
            StealthFeature::AccessibilityCapture => self.accessibility_capture,
            StealthFeature::SkipRuntimeStability => self.skip_runtime_stability,
            StealthFeature::BackendFocusInput => self.backend_focus_input,
            StealthFeature::EnterSubmit => self.enter_submit,
            StealthFeature::BackendPointerTarget => self.backend_pointer_target,
        }
    }

    pub(crate) fn launch_headed(self) -> bool {
        self.enabled(StealthFeature::LaunchHeaded)
    }

    pub(crate) fn launch_native_window(self) -> bool {
        self.enabled(StealthFeature::LaunchNativeWindow)
    }

    pub(crate) fn runtime_disable(self) -> bool {
        self.enabled(StealthFeature::RuntimeDisable)
    }

    pub(crate) fn accessibility_capture(self) -> bool {
        self.enabled(StealthFeature::AccessibilityCapture)
    }

    pub(crate) fn skip_runtime_stability(self) -> bool {
        self.enabled(StealthFeature::SkipRuntimeStability)
    }

    pub(crate) fn backend_focus_input(self) -> bool {
        self.enabled(StealthFeature::BackendFocusInput)
    }

    pub(crate) fn enter_submit(self) -> bool {
        self.enabled(StealthFeature::EnterSubmit)
    }

    pub(crate) fn backend_pointer_target(self) -> bool {
        self.enabled(StealthFeature::BackendPointerTarget)
    }

    fn set(&mut self, feature: StealthFeature, enabled: bool) {
        match feature {
            StealthFeature::LaunchHeaded => self.launch_headed = enabled,
            StealthFeature::LaunchNativeWindow => self.launch_native_window = enabled,
            StealthFeature::RuntimeDisable => self.runtime_disable = enabled,
            StealthFeature::AccessibilityCapture => self.accessibility_capture = enabled,
            StealthFeature::SkipRuntimeStability => self.skip_runtime_stability = enabled,
            StealthFeature::BackendFocusInput => self.backend_focus_input = enabled,
            StealthFeature::EnterSubmit => self.enter_submit = enabled,
            StealthFeature::BackendPointerTarget => self.backend_pointer_target = enabled,
        }
    }
}

fn parse_override(value: &str) -> Option<(StealthFeature, bool)> {
    if let Some(feature) = value.strip_prefix('+').and_then(StealthFeature::parse) {
        return Some((feature, true));
    }
    if let Some(feature) = value.strip_prefix('-').and_then(StealthFeature::parse) {
        return Some((feature, false));
    }
    let (feature, enabled) = value.split_once('=')?;
    let feature = StealthFeature::parse(feature.trim())?;
    let enabled = match enabled.trim() {
        "true" | "on" | "enabled" => true,
        "false" | "off" | "disabled" => false,
        _ => return None,
    };
    Some((feature, enabled))
}

#[cfg(test)]
#[path = "_tests_/stealth_features_tests.rs"]
mod stealth_features_tests;
