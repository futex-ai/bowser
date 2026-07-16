//! Main-frame navigation state helpers.

use chromiumoxide::cdp::browser_protocol::page::GetFrameTreeParams;

use super::navigation_state::{NAVIGATION_READY_POLL, NavigationDeadline};
use super::types::LivePage;

impl LivePage {
    pub(super) async fn main_frame_loader_id_within(
        &self,
        deadline: &NavigationDeadline,
    ) -> Option<String> {
        let timeout = deadline.state_read_timeout();
        if timeout.is_zero() {
            return None;
        }
        let frame_tree =
            tokio::time::timeout(timeout, self.page.execute(GetFrameTreeParams::default()))
                .await
                .ok()?
                .ok()?
                .frame_tree
                .clone();
        Some(frame_tree.frame.loader_id.as_ref().to_string())
    }

    pub(super) async fn wait_for_ready_document_with_loader_change_within(
        &self,
        previous_loader_id: &str,
        deadline: &NavigationDeadline,
    ) -> bool {
        while let Some(remaining) = deadline.remaining() {
            if self
                .main_frame_loader_id_within(deadline)
                .await
                .is_some_and(|loader_id| loader_id != previous_loader_id)
                && self.ready_document_sample_within(deadline).await
            {
                return true;
            }
            tokio::time::sleep(NAVIGATION_READY_POLL.min(remaining)).await;
        }
        false
    }

    pub(super) async fn ready_document_sample_within(&self, deadline: &NavigationDeadline) -> bool {
        while let Some(remaining) = deadline.remaining() {
            if self
                .navigation_sample_within(deadline)
                .await
                .as_ref()
                .is_some_and(|sample| sample.ready)
            {
                return true;
            }
            tokio::time::sleep(NAVIGATION_READY_POLL.min(remaining)).await;
        }
        false
    }
}
