use super::help_text;

#[test]
fn help_text_includes_descriptions() {
    let help = help_text();
    assert!(help.contains("new page [URL]"));
    assert!(help.contains("click <ID>"));
    assert!(help.contains("Activate a clickable element"));
    assert!(help.contains("keypress <KEY...>"));
    assert!(help.contains("describe <ID>"));
    assert!(help.contains("quit | exit"));
    assert!(help.contains("input#12"));
}
