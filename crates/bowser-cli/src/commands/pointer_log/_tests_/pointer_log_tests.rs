use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use bowser::{Element, InputType, PageCapture, PageContent};
use tempfile::tempdir;

use crate::error::CliError;

use super::demo::target_element_id;
use super::event::{
    PointerEvent, PointerEventBatch, PointerEventKind, PointerEventValidationError, PointerTarget,
    PointerTargetRect, PointerViewport,
};
use super::html::{APP_JS, INDEX_HTML, STYLE_CSS};
use super::log::{FilePointerEventLog, NoopPointerEventLog, PointerEventLog};
use super::server::{PointerLogState, append_events};
use super::{effective_output, validate_loopback_bind};

#[test]
fn validate_loopback_bind_rejects_public_addresses() {
    let addr: SocketAddr = "0.0.0.0:8765".parse().expect("socket address");
    assert!(matches!(
        validate_loopback_bind(addr),
        Err(CliError::PointerLogNonLoopback { .. })
    ));
}

#[test]
fn effective_output_defaults_only_for_manual_mode() {
    assert_eq!(
        effective_output(false, None).expect("manual default"),
        PathBuf::from("browser-log")
    );
    assert_eq!(effective_output(true, None), None);
}

#[test]
fn batch_validation_rejects_non_finite_coordinates() {
    let mut batch = sample_batch();
    batch.events[0].x = Some(f64::NAN);

    assert!(matches!(
        batch.validate(),
        Err(PointerEventValidationError::NonFiniteNumber { field: "x" })
    ));
}

#[test]
fn batch_validation_rejects_non_finite_target_rect() {
    let mut batch = sample_batch();
    batch.events[0]
        .target_rect
        .as_mut()
        .expect("target rect")
        .left = f64::INFINITY;

    assert!(matches!(
        batch.validate(),
        Err(PointerEventValidationError::NonFiniteNumber {
            field: "target_rect.left"
        })
    ));
}

#[tokio::test]
async fn append_events_accepts_valid_batches() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("route-events.jsonl");
    let log = Arc::new(FilePointerEventLog::open(path.clone()).expect("open pointer log"));
    let state = PointerLogState { log };
    let response = append_events(State(state), Json(sample_batch())).await;
    let contents = fs::read_to_string(path).expect("read route log");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(contents.lines().count(), 1);
    assert!(contents.contains("\"kind\":\"pointer_down\""));
}

#[test]
fn file_log_appends_one_json_line_per_event() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("pointer-events.jsonl");
    let log = FilePointerEventLog::open(path.clone()).expect("open pointer log");
    let mut batch = sample_batch();
    batch.events.push(PointerEvent {
        kind: PointerEventKind::PointerUp,
        time_ms: 1_702_000_001_000.0,
        elapsed_ms: 120.0,
        x: Some(31.0),
        y: Some(42.0),
        movement_x: Some(1.0),
        movement_y: Some(2.0),
        delta_x: None,
        delta_y: None,
        button: Some(0),
        buttons: Some(0),
        pointer_type: Some("mouse".to_string()),
        target: sample_target(),
        target_rect: Some(sample_target_rect()),
    });

    let written = log.append_batch(&batch).expect("append batch");
    let contents = fs::read_to_string(path).expect("read pointer log");
    let lines: Vec<&str> = contents.lines().collect();

    assert_eq!(written, 2);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("\"schema_version\":2"));
    assert!(!lines[0].contains("\"session_id\""));
    assert!(!lines[0].contains("\"user_agent\""));
    assert!(lines[0].contains("\"target_rect\""));
    assert!(lines[1].contains("\"kind\":\"pointer_up\""));
}

#[test]
fn noop_log_accepts_without_writing() {
    let log = NoopPointerEventLog;

    let written = log.append_batch(&sample_batch()).expect("append batch");

    assert_eq!(written, 1);
}

#[test]
fn static_page_loads_random_replacing_target_assets() {
    assert!(INDEX_HTML.contains("/pointer-log.css"));
    assert!(INDEX_HTML.contains("/pointer-log.js"));
    assert!(STYLE_CSS.contains("#stage"));
    assert!(STYLE_CSS.contains(".cursor-trail-segment"));
    assert!(STYLE_CSS.contains(".click-dot"));
    assert!(APP_JS.contains("randomItem"));
    assert!(APP_JS.contains("stage.replaceChildren()"));
    assert!(APP_JS.contains("window.setTimeout(spawnTarget, 0)"));
    assert!(APP_JS.contains("updatePointerVisual"));
    assert!(APP_JS.contains("animateClick"));
    assert!(APP_JS.contains("__pointerLogFlush"));
    assert!(APP_JS.contains("kind: \"target_spawn\""));
    assert!(APP_JS.contains("target_rect: targetRectInfo"));
    assert!(!APP_JS.contains("navigator.userAgent"));
    assert!(!APP_JS.contains("session_id"));
}

#[test]
fn demo_target_lookup_finds_nested_random_target() {
    let capture = PageCapture {
        url: "http://127.0.0.1:8765/".to_string(),
        title: "Bowser Pointer Log".to_string(),
        body_id: Some(1),
        obscured_body_id: None,
        content: PageContent::visible_only(vec![Element::Section {
            id: 2,
            tag: "main".to_string(),
            children: vec![Element::Input {
                id: Some(7),
                name: None,
                input_type: InputType::Textarea,
                placeholder: None,
                value: "Textarea".to_string(),
                label: Some("textarea".to_string()),
                options: Vec::new(),
                focused: false,
            }],
            truncation: None,
            focused: false,
        }]),
    };

    assert_eq!(target_element_id(&capture), Some(7));
}

fn sample_batch() -> PointerEventBatch {
    PointerEventBatch {
        page_started_at_ms: 1_702_000_000_000,
        viewport: PointerViewport {
            width: 1280,
            height: 720,
            device_pixel_ratio: 2.0,
        },
        events: vec![PointerEvent {
            kind: PointerEventKind::PointerDown,
            time_ms: 1_702_000_000_900.0,
            elapsed_ms: 20.0,
            x: Some(30.0),
            y: Some(40.0),
            movement_x: Some(0.0),
            movement_y: Some(0.0),
            delta_x: None,
            delta_y: None,
            button: Some(0),
            buttons: Some(1),
            pointer_type: Some("mouse".to_string()),
            target: sample_target(),
            target_rect: Some(sample_target_rect()),
        }],
    }
}

fn sample_target() -> PointerTarget {
    PointerTarget {
        tag: "button".to_string(),
        id: Some("primary".to_string()),
        role: Some("button".to_string()),
        label: Some("Primary".to_string()),
        classes: Some("action".to_string()),
    }
}

fn sample_target_rect() -> PointerTargetRect {
    PointerTargetRect {
        left: 20.0,
        top: 32.0,
        width: 84.0,
        height: 42.0,
    }
}
