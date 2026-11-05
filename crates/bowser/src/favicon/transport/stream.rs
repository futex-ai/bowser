//! Size-bounded response stream collection.

use base64::{Engine, engine::general_purpose::STANDARD};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::fetch::{
    EventRequestPaused, FailRequestParams, TakeResponseBodyAsStreamParams,
};
use chromiumoxide::cdp::browser_protocol::io::{
    CloseParams as CloseStreamParams, ReadParams, StreamHandle,
};
use chromiumoxide::cdp::browser_protocol::network::ErrorReason;

use crate::favicon::resource::MAX_SOURCE_BYTES;
use crate::favicon::transport::types::CleanupState;

const STREAM_CHUNK_BYTES: i64 = 64 * 1024;

pub(super) async fn read_response_stream(
    page: &Page,
    event: &EventRequestPaused,
    state: &CleanupState,
) -> Option<Vec<u8>> {
    let stream = page
        .execute(TakeResponseBodyAsStreamParams::new(
            event.request_id.clone(),
        ))
        .await
        .ok()?
        .stream
        .clone();
    *state.stream.lock().await = Some(stream.clone());
    let result = read_stream(page, &stream).await;
    let _ = page.execute(CloseStreamParams::new(stream)).await;
    *state.stream.lock().await = None;
    let _ = page
        .execute(FailRequestParams::new(
            event.request_id.clone(),
            ErrorReason::Aborted,
        ))
        .await;
    result
}

async fn read_stream(page: &Page, stream: &StreamHandle) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    loop {
        let params = ReadParams::builder()
            .handle(stream.clone())
            .size(STREAM_CHUNK_BYTES)
            .build()
            .ok()?;
        let chunk = page.execute(params).await.ok()?;
        let bytes = if chunk.base64_encoded == Some(true) {
            STANDARD.decode(&chunk.data).ok()?
        } else {
            chunk.data.as_bytes().to_vec()
        };
        if result.len().saturating_add(bytes.len()) > MAX_SOURCE_BYTES {
            return None;
        }
        result.extend_from_slice(&bytes);
        if chunk.eof {
            return Some(result);
        }
    }
}
