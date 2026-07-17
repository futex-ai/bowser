use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bowser::ScrollTarget;

use super::completion::suggestions;
use super::hints::inline_hint;
use super::{HelperState, ReplCommand, default_screenshot_path, parse_command, prompt};

#[test]
fn parses_js_and_quoted_type_commands() {
    assert_eq!(
        parse_command("js document.title").expect("js"),
        ReplCommand::Js("document.title".to_string())
    );
    assert_eq!(
        parse_command("type 7 \"hello world\"").expect("type"),
        ReplCommand::Type(7, "hello world".to_string())
    );
    assert_eq!(
        parse_command("type input#7 hello").expect("prefixed type"),
        ReplCommand::Type(7, "hello".to_string())
    );
    assert_eq!(
        parse_command("keypress cmd enter").expect("keypress combo"),
        ReplCommand::KeyPress(vec!["cmd".to_string(), "enter".to_string()])
    );
    assert_eq!(
        parse_command("keypress cmd+enter").expect("keypress joined combo"),
        ReplCommand::KeyPress(vec!["cmd".to_string(), "enter".to_string()])
    );
    assert_eq!(
        parse_command("new page https://example.com").expect("new page"),
        ReplCommand::NewPage(Some("https://example.com".to_string()))
    );
    assert_eq!(
        parse_command("close page pg_2").expect("close page"),
        ReplCommand::ClosePage(Some("pg_2".to_string()))
    );
}

#[test]
fn rejects_extra_arguments_for_fixed_arity_commands() {
    for command in [
        "back now",
        "click 7 now",
        "type 7 hello world",
        "yaml 7 now",
        "scroll down now",
    ] {
        assert!(
            parse_command(command).is_err(),
            "accepted extra command arguments: {command}"
        );
    }
}

#[test]
fn parses_prefixed_ids_for_id_commands() {
    assert_eq!(
        parse_command("click link#9").expect("click"),
        ReplCommand::Click(9)
    );
    assert_eq!(
        parse_command("yaml article#11").expect("yaml"),
        ReplCommand::Yaml(Some(11))
    );
    assert_eq!(
        parse_command("expand table#15").expect("expand"),
        ReplCommand::Expand(15)
    );
    assert_eq!(
        parse_command("meta image#21").expect("meta"),
        ReplCommand::Meta(21)
    );
    assert_eq!(
        parse_command("describe image#22").expect("describe"),
        ReplCommand::Describe(22)
    );
}

#[test]
fn suggests_commands_and_ids() {
    let page_ids = vec!["pg_1".to_string(), "pg_2".to_string()];
    assert_eq!(
        suggestions("cl", &[3, 7], &page_ids),
        vec!["click", "clear", "close"]
    );
    assert_eq!(suggestions("click ", &[3, 7], &page_ids), vec!["3", "7"]);
    assert_eq!(
        suggestions("keypress e", &[3, 7], &page_ids),
        vec!["enter", "escape"]
    );
    assert_eq!(suggestions("expand 7", &[3, 7], &page_ids), vec!["7"]);
    assert_eq!(suggestions("describe ", &[3, 7], &page_ids), vec!["3", "7"]);
    assert_eq!(suggestions("scroll to ", &[12], &page_ids), vec!["12"]);
    assert_eq!(suggestions("page ", &[12], &page_ids), vec!["pg_1", "pg_2"]);
    assert_eq!(suggestions("new ", &[12], &page_ids), vec!["page"]);
}

#[test]
fn unknown_command_includes_suggestion() {
    let error = parse_command("clcik 9").expect_err("invalid");
    assert!(error.to_string().contains("did you mean `"));
}

#[test]
fn parses_scroll_and_screenshot_commands() {
    assert_eq!(
        parse_command("scroll to 5").expect("scroll"),
        ReplCommand::Scroll(ScrollTarget::ToElement(5))
    );
    assert_eq!(
        parse_command("screenshot 8 out.png").expect("screenshot"),
        ReplCommand::Screenshot(Some(8), Some(PathBuf::from("out.png")))
    );
}

#[test]
fn default_screenshot_path_uses_png_extension() {
    let path = default_screenshot_path();
    assert_eq!(
        path.extension().and_then(|value| value.to_str()),
        Some("png")
    );
}

#[test]
fn inline_hint_requires_a_typed_character() {
    let page_ids = vec!["pg_1".to_string(), "pg_2".to_string()];
    assert_eq!(inline_hint("", 0, &[3, 7], &page_ids), None);
    assert_eq!(inline_hint("click ", 6, &[3, 7], &page_ids), None);
    assert_eq!(
        inline_hint("c", 1, &[3, 7], &page_ids),
        Some("lick".to_string())
    );
    assert_eq!(inline_hint("click 7", 7, &[3, 7], &page_ids), None);
    assert_eq!(
        inline_hint("click 3", 7, &[3, 30], &page_ids),
        Some("0".to_string())
    );
    assert_eq!(
        inline_hint("page p", 6, &[3, 30], &page_ids),
        Some("g_1".to_string())
    );
}

#[test]
fn prompt_includes_current_page_url() {
    let helper_state = Arc::new(Mutex::new(HelperState {
        ids: Vec::new(),
        page_ids: Vec::new(),
        current_url: Some("https://example.com/path".to_string()),
        current_page_id: None,
    }));
    let (raw, styled) = prompt(&helper_state);
    assert!(raw.contains("example.com/path"));
    assert!(raw.contains("bowser> "));
    assert!(styled.contains("\x1b[36mexample.com/path\x1b[0m"));
}
