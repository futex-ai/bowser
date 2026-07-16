use super::{MODIFIER_META, MODIFIER_SHIFT, resolve_pressed_keys, resolve_text_character};

#[test]
fn resolves_aliases_and_literal_shortcuts() {
    let keys = resolve_pressed_keys(&[
        "cmd".to_string(),
        "enter".to_string(),
        "space".to_string(),
        "K".to_string(),
    ])
    .expect("keys");
    assert_eq!(keys[0].key, "Meta");
    assert_eq!(keys[0].modifier_mask, MODIFIER_META);
    assert_eq!(keys[1].key, "Enter");
    assert_eq!(keys[2].code, "Space");
    assert_eq!(keys[3].key, "k");
}

#[test]
fn resolves_shifted_text_characters() {
    let uppercase = resolve_text_character("A").expect("uppercase");
    let punctuation = resolve_text_character("!").expect("punctuation");
    assert_eq!(uppercase.key, "A");
    assert_eq!(uppercase.modifier_mask, 0);
    assert_eq!(punctuation.key, "!");
    assert_ne!(uppercase.code, punctuation.code);
    assert_eq!(super::modifier_mask_for("Shift"), MODIFIER_SHIFT);
}
