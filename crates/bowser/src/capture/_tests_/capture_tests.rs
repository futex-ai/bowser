use super::{truncate_capture, truncate_element};
use crate::config::OutputConfig;
use crate::model::{Element, ListItem, ListType, PageCapture, PageContent};

#[test]
fn truncates_long_lists() {
    let full = PageCapture {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        body_id: None,
        obscured_body_id: None,
        content: PageContent::visible_only(vec![Element::List {
            id: 1,
            list_type: ListType::Unordered,
            items: (0..3)
                .map(|index| ListItem {
                    children: vec![Element::Text {
                        text: format!("item {index}"),
                    }],
                })
                .collect(),
            truncation: None,
            focused: false,
        }]),
    };
    let preview = truncate_capture(
        &full,
        &OutputConfig {
            truncate: true,
            include_hidden: false,
            max_children: 50,
            max_list_items: 2,
            max_table_rows: 20,
        },
    );
    match &preview.content.visible[0] {
        Element::List {
            items, truncation, ..
        } => {
            assert_eq!(items.len(), 2);
            assert!(truncation.is_some());
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn truncation_helper_is_safe_on_text() {
    let mut element = Element::Text {
        text: "hello".to_string(),
    };
    truncate_element(
        &mut element,
        &OutputConfig {
            truncate: true,
            include_hidden: false,
            max_children: 1,
            max_list_items: 1,
            max_table_rows: 1,
        },
    );
    assert!(matches!(element, Element::Text { .. }));
}

#[test]
fn wraps_truncated_root_in_body_container() {
    let full = PageCapture {
        url: "https://example.com/orders".to_string(),
        title: "Orders".to_string(),
        body_id: Some(99),
        obscured_body_id: None,
        content: PageContent::visible_only(
            (1..=3)
                .map(|id| Element::Section {
                    id,
                    tag: "article".to_string(),
                    children: vec![Element::Text {
                        text: format!("Order {id}"),
                    }],
                    truncation: None,
                    focused: false,
                })
                .collect(),
        ),
    };
    let preview = truncate_capture(
        &full,
        &OutputConfig {
            truncate: true,
            include_hidden: false,
            max_children: 2,
            max_list_items: 20,
            max_table_rows: 20,
        },
    );
    match preview.content.visible.as_slice() {
        [
            Element::Section {
                id,
                tag,
                children,
                truncation: Some(truncation),
                ..
            },
        ] => {
            assert_eq!(*id, 99);
            assert_eq!(tag, "body");
            assert_eq!(children.len(), 2);
            assert_eq!(truncation.shown, 2);
            assert_eq!(truncation.total, 3);
        }
        other => panic!("expected truncated body wrapper, got {other:?}"),
    }
}

#[test]
fn sets_section_truncation_on_the_container() {
    let full = PageCapture {
        url: "https://example.com/section".to_string(),
        title: "Section".to_string(),
        body_id: None,
        obscured_body_id: None,
        content: PageContent::visible_only(vec![Element::Section {
            id: 10,
            tag: "section".to_string(),
            children: (1..=3)
                .map(|id| Element::Button {
                    id,
                    text: format!("Button {id}"),
                    focused: false,
                })
                .collect(),
            truncation: None,
            focused: false,
        }]),
    };
    let preview = truncate_capture(
        &full,
        &OutputConfig {
            truncate: true,
            include_hidden: false,
            max_children: 2,
            max_list_items: 20,
            max_table_rows: 20,
        },
    );
    match &preview.content.visible[0] {
        Element::Section {
            children,
            truncation: Some(truncation),
            ..
        } => {
            assert_eq!(children.len(), 2);
            assert_eq!(truncation.shown, 2);
            assert_eq!(truncation.total, 3);
        }
        other => panic!("expected truncated section, got {other:?}"),
    }
}
