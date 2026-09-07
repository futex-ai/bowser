//! Deadline-bound browser-context resource loader.

use std::time::Duration;

use async_trait::async_trait;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::fetch::{
    ContinueRequestParams, EnableParams as FetchEnableParams, EventRequestPaused,
    FailRequestParams, RequestId, RequestPattern, RequestStage,
};
use chromiumoxide::cdp::browser_protocol::network::{
    EnableParams as NetworkEnableParams, ErrorReason, EventRequestWillBeSent,
    SetAttachDebugStackParams,
};
use chromiumoxide::cdp::browser_protocol::page::{CreateIsolatedWorldParams, GetFrameTreeParams};
use chromiumoxide::cdp::js_protocol::runtime::{
    EvaluateParams, SetAsyncCallStackDepthParams, StackTrace,
};
use futures::StreamExt;
use tokio::time::Instant;
use uuid::Uuid;

use crate::favicon::resource::MAX_SOURCE_BYTES;
use crate::favicon::transport::cleanup::cleanup;
use crate::favicon::transport::stream::read_response_stream;
use crate::favicon::transport::types::{CleanupState, ImageHandle};

const CLEANUP_RESERVE: Duration = Duration::from_millis(200);
const IMAGE_CLEAR_MILLIS: u64 = 800;
const MAX_REDIRECTS: usize = 3;

#[cfg_attr(test, unimock::unimock(api = BrowserResourceLoaderMock))]
#[async_trait]
pub(crate) trait BrowserResourceLoader: Send + Sync {
    async fn load(&self, address: &str) -> Option<Vec<u8>>;
}

pub(crate) struct ChromiumResourceLoader {
    deadline: Instant,
    page: Page,
}

impl ChromiumResourceLoader {
    pub(crate) fn new(page: Page, deadline: Instant) -> Self {
        Self { deadline, page }
    }
}

#[async_trait]
impl BrowserResourceLoader for ChromiumResourceLoader {
    async fn load(&self, address: &str) -> Option<Vec<u8>> {
        let address = address.to_string();
        let deadline = self.deadline;
        let page = self.page.clone();
        tokio::spawn(async move { load_with_cleanup(page, address, deadline).await })
            .await
            .ok()
            .flatten()
    }
}

async fn load_with_cleanup(page: Page, address: String, deadline: Instant) -> Option<Vec<u8>> {
    if Instant::now() >= deadline {
        return None;
    }
    let state = CleanupState::default();
    let work_deadline = deadline.checked_sub(CLEANUP_RESERVE).unwrap_or(deadline);
    let result = tokio::time::timeout_at(work_deadline, load_resource(&page, &address, &state))
        .await
        .ok()
        .flatten();
    let _ = tokio::time::timeout_at(deadline, cleanup(&page, &state)).await;
    result
}

async fn load_resource(page: &Page, address: &str, state: &CleanupState) -> Option<Vec<u8>> {
    let marker = format!("bowser-favicon-{}", Uuid::new_v4());
    let mut network_events = page.event_listener::<EventRequestWillBeSent>().await.ok()?;
    let mut fetch_events = page.event_listener::<EventRequestPaused>().await.ok()?;
    enable_interception(page).await?;
    start_isolated_image(page, address, &marker, state).await?;

    let mut owned_network_id = None;
    let mut redirects = 0;
    loop {
        tokio::select! {
            biased;
            network_event = network_events.next() => {
                let event = network_event?;
                if stack_contains_marker(event.initiator.stack.as_ref(), &marker) {
                    owned_network_id = Some(event.request_id.as_ref().to_string());
                }
            }
            fetch_event = fetch_events.next() => {
                let event = fetch_event?;
                let is_owned = event.network_id.as_ref().is_some_and(|network_id| {
                    owned_network_id.as_deref() == Some(network_id.as_ref())
                });
                if !is_owned {
                    page.execute(ContinueRequestParams::new(event.request_id.clone()))
                        .await
                        .ok()?;
                    continue;
                }
                remember_request(state, event.request_id.clone()).await;
                if event.response_status_code.is_none() {
                    let params = ContinueRequestParams::builder()
                        .request_id(event.request_id.clone())
                        .intercept_response(true)
                        .build()
                        .ok()?;
                    page.execute(params).await.ok()?;
                    continue;
                }
                let status = event.response_status_code?;
                if matches!(status, 301 | 302 | 303 | 307 | 308) {
                    redirects += 1;
                    if redirects > MAX_REDIRECTS {
                        abort_request(page, event.request_id.clone()).await?;
                        return None;
                    }
                    page.execute(ContinueRequestParams::new(event.request_id.clone()))
                        .await
                        .ok()?;
                    continue;
                }
                if !(200..300).contains(&status) || response_too_large(&event) {
                    abort_request(page, event.request_id.clone()).await?;
                    return None;
                }
                return read_response_stream(page, &event, state).await;
            }
        }
    }
}

async fn enable_interception(page: &Page) -> Option<()> {
    page.execute(NetworkEnableParams::default()).await.ok()?;
    page.execute(SetAsyncCallStackDepthParams::new(32))
        .await
        .ok()?;
    page.execute(SetAttachDebugStackParams::new(true))
        .await
        .ok()?;
    let pattern = RequestPattern::builder()
        .url_pattern("*")
        .request_stage(RequestStage::Request)
        .build();
    page.execute(FetchEnableParams::builder().pattern(pattern).build())
        .await
        .ok()?;
    Some(())
}

async fn start_isolated_image(
    page: &Page,
    address: &str,
    marker: &str,
    state: &CleanupState,
) -> Option<()> {
    let frame_tree = page.execute(GetFrameTreeParams::default()).await.ok()?;
    let world = page
        .execute(
            CreateIsolatedWorldParams::builder()
                .frame_id(frame_tree.frame_tree.frame.id.clone())
                .world_name("bowser-favicon-acquisition")
                .build()
                .ok()?,
        )
        .await
        .ok()?;
    *state.image.lock().await = Some(ImageHandle {
        context_id: world.execution_context_id,
        marker: marker.to_string(),
    });
    let address = serde_json::to_string(address).ok()?;
    let key = serde_json::to_string(marker).ok()?;
    let expression = format!(
        "(() => {{ const icon = new Image(); globalThis[{key}] = icon; icon.src = {address}; setTimeout(() => {{ if (globalThis[{key}] === icon) {{ icon.src = ''; delete globalThis[{key}]; }} }}, {IMAGE_CLEAR_MILLIS}); }})()\n//# sourceURL={marker}"
    );
    page.execute(
        EvaluateParams::builder()
            .expression(expression)
            .context_id(world.execution_context_id)
            .await_promise(false)
            .build()
            .ok()?,
    )
    .await
    .ok()?;
    Some(())
}

fn stack_contains_marker(stack: Option<&StackTrace>, marker: &str) -> bool {
    let Some(stack) = stack else {
        return false;
    };
    stack.call_frames.iter().any(|frame| frame.url == marker)
        || stack_contains_marker(stack.parent.as_deref(), marker)
}

fn response_too_large(event: &EventRequestPaused) -> bool {
    event
        .response_headers
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case("content-length"))
        .filter_map(|header| header.value.parse::<usize>().ok())
        .any(|length| length > MAX_SOURCE_BYTES)
}

async fn remember_request(state: &CleanupState, request_id: RequestId) {
    let mut requests = state.active_requests.lock().await;
    if !requests.contains(&request_id) {
        requests.push(request_id);
    }
}

async fn abort_request(page: &Page, request_id: RequestId) -> Option<()> {
    page.execute(FailRequestParams::new(request_id, ErrorReason::Aborted))
        .await
        .ok()?;
    Some(())
}
