//! Deterministic cleanup for canceled or completed favicon loads.

use std::time::Duration;

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::fetch::{
    DisableParams as FetchDisableParams, FailRequestParams,
};
use chromiumoxide::cdp::browser_protocol::io::CloseParams as CloseStreamParams;
use chromiumoxide::cdp::browser_protocol::network::{ErrorReason, SetAttachDebugStackParams};
use chromiumoxide::cdp::js_protocol::runtime::{EvaluateParams, SetAsyncCallStackDepthParams};
use futures::future::join_all;

use crate::favicon::transport::types::{CleanupState, ImageHandle};

const PROTOCOL_CLEANUP_SLICE: Duration = Duration::from_millis(50);

pub(super) async fn cleanup(page: &Page, state: &CleanupState) {
    let stream = { state.stream.lock().await.take() };
    let requests = { state.active_requests.lock().await.clone() };
    let abort_requests =
        join_all(requests.into_iter().map(|request_id| {
            page.execute(FailRequestParams::new(request_id, ErrorReason::Aborted))
        }));
    let close_stream = async {
        if let Some(stream) = stream {
            let _ = page.execute(CloseStreamParams::new(stream)).await;
        }
    };
    let _ = tokio::time::timeout(PROTOCOL_CLEANUP_SLICE, async {
        tokio::join!(abort_requests, close_stream);
    })
    .await;
    let _ = tokio::time::timeout(
        PROTOCOL_CLEANUP_SLICE,
        page.execute(FetchDisableParams::default()),
    )
    .await;
    let image = { state.image.lock().await.take() };
    let clear = async {
        if let Some(image) = image {
            clear_image(page, image).await;
        }
    };
    let _ = tokio::join!(
        clear,
        page.execute(SetAttachDebugStackParams::new(false)),
        page.execute(SetAsyncCallStackDepthParams::new(0)),
    );
}

async fn clear_image(page: &Page, image: ImageHandle) {
    let Ok(marker) = serde_json::to_string(&image.marker) else {
        return;
    };
    let expression = format!(
        "(() => {{ const icon = globalThis[{marker}]; if (icon) icon.src = ''; delete globalThis[{marker}]; }})()"
    );
    let Ok(params) = EvaluateParams::builder()
        .expression(expression)
        .context_id(image.context_id)
        .await_promise(false)
        .build()
    else {
        return;
    };
    let _ = page.execute(params).await;
}
