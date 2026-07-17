use super::{from_json, to_json, to_yaml};
use crate::model::{Element, InputType, PageCapture, PageContent};

#[test]
fn renders_compact_yaml() {
    let capture = PageCapture {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        body_id: None,
        obscured_body_id: None,
        content: PageContent::visible_only(vec![
            Element::Link {
                id: 1,
                text: "Home".to_string(),
                href: "https://example.com".to_string(),
                focused: false,
            },
            Element::Input {
                id: Some(2),
                name: Some("q".to_string()),
                input_type: InputType::Text,
                placeholder: Some("Search".to_string()),
                value: String::new(),
                label: None,
                options: Vec::new(),
                focused: false,
            },
        ]),
    };

    let yaml = to_yaml(&capture).expect("yaml");
    assert!(yaml.contains("visible:"));
    assert!(yaml.contains("obscured: []"));
    assert!(yaml.contains("link#1"));
    assert!(yaml.contains("input#2"));
    assert!(yaml.contains("link#1: Home"));
    assert!(!yaml.contains("focused: false"));
}

#[test]
fn renders_focused_elements_only_when_active() {
    let capture = PageCapture {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        body_id: None,
        obscured_body_id: None,
        content: PageContent::visible_only(vec![Element::Button {
            id: 1,
            text: "Search".to_string(),
            focused: true,
        }]),
    };

    let yaml = to_yaml(&capture).expect("yaml");
    assert!(yaml.contains("button#1:"));
    assert!(yaml.contains("focused: true"));
    assert!(yaml.contains("text: Search"));

    let json = to_json(&capture).expect("json");
    assert!(json.contains("\"kind\": \"button\""));
}

#[test]
fn renders_image_labels_from_alt_and_filename() {
    let capture = PageCapture {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        body_id: None,
        obscured_body_id: None,
        content: PageContent::visible_only(vec![Element::Image {
            id: 3,
            alt: "Checkerboard".to_string(),
            src: "https://cdn.example.com/images/checkerboard.png?size=2".to_string(),
            description: Some("AI summary".to_string()),
            describable: true,
            focused: false,
        }]),
    };

    let yaml = to_yaml(&capture).expect("yaml");
    assert!(yaml.contains("image#3:"));
    assert!(yaml.contains("label: Checkerboard (checkerboard.png)"));
    assert!(yaml.contains("describable: true"));
    assert!(!yaml.contains("AI summary"));
}

#[test]
fn json_round_trips_visible_and_obscured_content() {
    let capture = PageCapture {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        body_id: Some(9),
        obscured_body_id: Some(10),
        content: PageContent::new(
            vec![Element::Text {
                text: "Visible".to_string(),
            }],
            vec![Element::Button {
                id: 2,
                text: "Hidden action".to_string(),
                focused: false,
            }],
        ),
    };

    let json = to_json(&capture).expect("json");
    assert!(json.contains("\"visible\""));
    assert!(json.contains("\"obscured\""));
    assert!(!json.contains("\"children\""));

    let round_trip = from_json(&json).expect("round trip");
    assert_eq!(round_trip, capture);
}
