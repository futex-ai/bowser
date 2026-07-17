//! Keyboard key normalization and resolution helpers.

use chromiumoxide::keys::get_key_definition;

use crate::error::{Error, Result};

pub(crate) const MODIFIER_ALT: i64 = 1;
pub(crate) const MODIFIER_CTRL: i64 = 2;
pub(crate) const MODIFIER_META: i64 = 4;
pub(crate) const MODIFIER_SHIFT: i64 = 8;

/// A resolved browser key definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedKey {
    pub key: String,
    pub code: String,
    pub key_code: i64,
    pub text: Option<String>,
    pub modifier_mask: i64,
}

impl ResolvedKey {
    pub fn is_modifier(&self) -> bool {
        self.modifier_mask != 0
    }

    pub fn event_text(&self) -> Option<String> {
        self.text
            .clone()
            .or_else(|| (self.key.chars().count() == 1).then(|| self.key.clone()))
    }
}

pub(crate) fn resolve_pressed_keys(keys: &[String]) -> Result<Vec<ResolvedKey>> {
    keys.iter()
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .map(resolve_pressed_key)
        .collect()
}

pub(crate) fn resolve_text_character(value: &str) -> Result<ResolvedKey> {
    resolve_key(&canonical_text_key(value), modifier_mask_for(value))
}

fn resolve_pressed_key(value: &str) -> Result<ResolvedKey> {
    let canonical = canonical_pressed_key(value);
    let modifier_mask = modifier_mask_for(&canonical);
    resolve_key(&canonical, modifier_mask)
}

fn resolve_key(value: &str, modifier_mask: i64) -> Result<ResolvedKey> {
    let definition = get_key_definition(value).ok_or(Error::KeyPress {
        reason: format!("unsupported key: {value}"),
    })?;
    Ok(ResolvedKey {
        key: definition.key.to_string(),
        code: definition.code.to_string(),
        key_code: definition.key_code,
        text: definition.text.map(ToString::to_string),
        modifier_mask,
    })
}

fn canonical_pressed_key(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "cmd" | "command" | "meta" => "Meta".to_string(),
        "ctrl" | "control" => "Control".to_string(),
        "alt" | "option" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "space" | "spacebar" => " ".to_string(),
        "tab" => "Tab".to_string(),
        "esc" | "escape" => "Escape".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" | "del" => "Delete".to_string(),
        "up" | "arrowup" => "ArrowUp".to_string(),
        "down" | "arrowdown" => "ArrowDown".to_string(),
        "left" | "arrowleft" => "ArrowLeft".to_string(),
        "right" | "arrowright" => "ArrowRight".to_string(),
        "home" => "Home".to_string(),
        "end" => "End".to_string(),
        "pageup" => "PageUp".to_string(),
        "pagedown" => "PageDown".to_string(),
        _ if value.chars().count() == 1 && value.chars().all(|ch| ch.is_ascii_alphabetic()) => {
            value.to_ascii_lowercase()
        }
        _ => value.to_string(),
    }
}

fn canonical_text_key(value: &str) -> String {
    match value {
        "\n" | "\r" => "Enter".to_string(),
        "\t" => "Tab".to_string(),
        _ => value.to_string(),
    }
}

fn modifier_mask_for(value: &str) -> i64 {
    match value {
        "Alt" => MODIFIER_ALT,
        "Control" => MODIFIER_CTRL,
        "Meta" => MODIFIER_META,
        "Shift" => MODIFIER_SHIFT,
        _ => 0,
    }
}

#[cfg(test)]
#[path = "_tests_/keyboard_tests.rs"]
mod keyboard_tests;
