//! State retained across bounded transport work and cleanup.

use chromiumoxide::cdp::browser_protocol::fetch::RequestId;
use chromiumoxide::cdp::browser_protocol::io::StreamHandle;
use chromiumoxide::cdp::js_protocol::runtime::ExecutionContextId;
use tokio::sync::Mutex;

pub(super) struct ImageHandle {
    pub(super) context_id: ExecutionContextId,
    pub(super) marker: String,
}

#[derive(Default)]
pub(super) struct CleanupState {
    pub(super) active_requests: Mutex<Vec<RequestId>>,
    pub(super) image: Mutex<Option<ImageHandle>>,
    pub(super) stream: Mutex<Option<StreamHandle>>,
}
