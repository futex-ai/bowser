mod support;

use bowser::{Browser, BrowserEngine, Element};
use serde::Deserialize;
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn visible_clicks_follow_pointer_routes_and_reuse_cursor_position() {
    let _guard = support::browser_test_guard();
    let server = support::spawn_server().await;
    let session_dir = tempdir().expect("session dir");
    let config = support::test_config(session_dir.path());

    let browser = Browser::launch(config).await.expect("launch browser");
    let page = browser.current_page().await.expect("page");

    page.navigate(&server.url("/mouse-click"))
        .await
        .expect("navigate pointer fixture");
    let capture = page.capture().await.expect("capture pointer fixture");
    let alpha_id = find_button_id(&capture.content.visible, "Alpha").expect("alpha id");
    let beta_id = find_button_id(&capture.content.visible, "Beta").expect("beta id");

    page.click(alpha_id).await.expect("click alpha");
    let alpha_capture = page.capture().await.expect("capture alpha click");
    let alpha_yaml = bowser::to_yaml(&alpha_capture).expect("alpha yaml");
    assert!(alpha_yaml.contains("Alpha clicked"));

    let first_log: Vec<PointerEvent> = serde_json::from_str(
        &page
            .evaluate_js("window.__bowserPointerLog")
            .await
            .expect("pointer log after alpha"),
    )
    .expect("parse pointer log after alpha");
    let first_down = index_of_mousedown(&first_log, "alpha").expect("first alpha mousedown");
    let first_moves = mousemoves_before(&first_log, first_down);
    assert!(first_moves.len() >= 18);
    let first_move = first_moves.first().expect("first mousemove");
    assert!(first_move.x >= 0.0 && first_move.x < 1280.0);
    assert!(first_move.y >= 0.0 && first_move.y < 800.0);

    page.click(beta_id).await.expect("click beta");
    let beta_capture = page.capture().await.expect("capture beta click");
    let beta_yaml = bowser::to_yaml(&beta_capture).expect("beta yaml");
    assert!(beta_yaml.contains("Beta clicked"));

    let geometry: ButtonGeometry = serde_json::from_str(
        &page
            .evaluate_js("window.__bowserButtonGeometry")
            .await
            .expect("button geometry"),
    )
    .expect("parse button geometry");
    let second_log: Vec<PointerEvent> = serde_json::from_str(
        &page
            .evaluate_js("window.__bowserPointerLog")
            .await
            .expect("pointer log after beta"),
    )
    .expect("parse pointer log after beta");
    let alpha_up = index_of_mouseup(&second_log, "alpha").expect("alpha mouseup");
    let alpha_down = index_of_mousedown(&second_log, "alpha").expect("alpha down");
    assert_padded_click_target(&second_log[alpha_down], geometry.alpha);
    let beta_down = index_of_mousedown_after(&second_log, "beta", alpha_up).expect("beta down");
    assert_padded_click_target(&second_log[beta_down], geometry.beta);
    let second_moves = mousemoves_between(&second_log, alpha_up, beta_down);
    assert!(second_moves.len() >= 4);
    let first_second_move = second_moves.first().expect("first second-sequence move");
    assert!(first_second_move.x > 24.0 && first_second_move.x < 1280.0 - 24.0);
    assert!(first_second_move.y > 24.0 && first_second_move.y < 800.0 - 24.0);
    assert!(
        distance(first_second_move, geometry.alpha.center)
            < distance(first_second_move, geometry.beta.center)
    );

    browser.close().await.expect("close browser");
}

#[derive(Clone, Debug, Deserialize)]
struct PointerEvent {
    #[serde(rename = "type")]
    event_type: EventType,
    x: f64,
    y: f64,
    target: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum EventType {
    Mousemove,
    Mousedown,
    Mouseup,
    Click,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct PointValue {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct ButtonGeometry {
    alpha: ElementGeometry,
    beta: ElementGeometry,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct ElementGeometry {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
    center: PointValue,
}

fn assert_padded_click_target(event: &PointerEvent, geometry: ElementGeometry) {
    const PADDING: f64 = 10.0;
    const EVENT_COORDINATE_TOLERANCE: f64 = 1.0;
    assert!(
        event.x + EVENT_COORDINATE_TOLERANCE >= geometry.left + PADDING,
        "expected x >= left padding: event={event:?} geometry={geometry:?}"
    );
    assert!(
        event.x - EVENT_COORDINATE_TOLERANCE <= geometry.right - PADDING,
        "expected x <= right padding: event={event:?} geometry={geometry:?}"
    );
    assert!(
        event.y + EVENT_COORDINATE_TOLERANCE >= geometry.top + PADDING,
        "expected y >= top padding: event={event:?} geometry={geometry:?}"
    );
    assert!(
        event.y - EVENT_COORDINATE_TOLERANCE <= geometry.bottom - PADDING,
        "expected y <= bottom padding: event={event:?} geometry={geometry:?}"
    );
    assert!(
        event.x != geometry.center.x || event.y != geometry.center.y,
        "expected randomized click instead of exact center, got {event:?}"
    );
}

fn distance(event: &PointerEvent, point: PointValue) -> f64 {
    let dx = event.x - point.x;
    let dy = event.y - point.y;
    (dx * dx + dy * dy).sqrt()
}

fn mousemoves_before(events: &[PointerEvent], index: usize) -> Vec<PointerEvent> {
    events[..index]
        .iter()
        .filter(|event| event.event_type == EventType::Mousemove)
        .cloned()
        .collect()
}

fn mousemoves_between(events: &[PointerEvent], start: usize, end: usize) -> Vec<PointerEvent> {
    events[start + 1..end]
        .iter()
        .filter(|event| event.event_type == EventType::Mousemove)
        .cloned()
        .collect()
}

fn index_of_mousedown(events: &[PointerEvent], target: &str) -> Option<usize> {
    events
        .iter()
        .position(|event| event.event_type == EventType::Mousedown && event.target == target)
}

fn index_of_mousedown_after(events: &[PointerEvent], target: &str, after: usize) -> Option<usize> {
    events
        .iter()
        .enumerate()
        .skip(after + 1)
        .find(|(_, event)| event.event_type == EventType::Mousedown && event.target == target)
        .map(|(index, _)| index)
}

fn index_of_mouseup(events: &[PointerEvent], target: &str) -> Option<usize> {
    events
        .iter()
        .rposition(|event| event.event_type == EventType::Mouseup && event.target == target)
}

fn find_button_id(elements: &[Element], target_text: &str) -> Option<u32> {
    find(elements, &|element| match element {
        Element::Button { id, text, .. } if text == target_text => Some(*id),
        _ => None,
    })
}

fn find<T>(elements: &[Element], predicate: &dyn Fn(&Element) -> Option<T>) -> Option<T> {
    for element in elements {
        if let Some(found) = predicate(element) {
            return Some(found);
        }
        match element {
            Element::Table { headers, rows, .. } => {
                for header in headers {
                    if let Some(found) = find(&header.children, predicate) {
                        return Some(found);
                    }
                }
                for row in rows {
                    for cell in &row.cells {
                        if let Some(found) = find(&cell.children, predicate) {
                            return Some(found);
                        }
                    }
                }
            }
            Element::List { items, .. } => {
                for item in items {
                    if let Some(found) = find(&item.children, predicate) {
                        return Some(found);
                    }
                }
            }
            Element::Nav { children, .. }
            | Element::Form { children, .. }
            | Element::Section { children, .. }
            | Element::Iframe { children, .. } => {
                if let Some(found) = find(children, predicate) {
                    return Some(found);
                }
            }
            Element::Heading { .. }
            | Element::Link { .. }
            | Element::Button { .. }
            | Element::Input { .. }
            | Element::Text { .. }
            | Element::Image { .. } => {}
        }
    }
    None
}
