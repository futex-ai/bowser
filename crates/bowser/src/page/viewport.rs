//! Pointer-targeting and viewport interaction helpers.

use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use chromiumoxide::layout::Point;

use crate::cdp_trace;
use crate::error::{Error, Result};
use crate::model::ScrollTarget;
use crate::mouse::{
    CursorPosition, ViewportArea, click_hold_ms, initial_cursor_position, movement_steps,
};

use super::form_scripts::scroll_to_element_expression;
use super::interaction_scripts::{interaction_target_expression, synthetic_click_expression};
use super::types::{InteractionTarget, LiveElementContext, LivePage, ViewportMetrics};

impl LivePage {
    pub(super) async fn scroll_impl(&self, target: ScrollTarget) -> Result<()> {
        match target {
            ScrollTarget::Down => {
                self.prepare().await?;
                self.wheel_scroll(ScrollTarget::Down).await
            }
            ScrollTarget::Up => {
                self.prepare().await?;
                self.wheel_scroll(ScrollTarget::Up).await
            }
            ScrollTarget::ToElement(id) => {
                self.ensure_live_ids(id).await?;
                if let Some(context @ LiveElementContext::Remote { .. }) =
                    self.live_element_context(id).await
                {
                    return self.scroll_frame_element(context, id).await;
                }
                self.run_action(scroll_to_element_expression(id), id).await
            }
        }
    }

    pub(super) async fn interaction_target(&self, element_id: u32) -> Result<InteractionTarget> {
        let expression = interaction_target_expression(element_id);
        self.evaluate_bowser_value::<Option<InteractionTarget>>(expression)
            .await?
            .ok_or(Error::ElementNotFound { element_id })
    }

    pub(super) async fn pointer_click(
        &self,
        element_id: u32,
        target: InteractionTarget,
    ) -> Result<()> {
        let viewport = ViewportArea {
            width: target.viewport_width.max(1.0),
            height: target.viewport_height.max(1.0),
        };
        let target_position = CursorPosition {
            x: target.x,
            y: target.y,
        };
        let start_position = {
            let state = self.state.lock().await;
            state
                .cursor_position
                .unwrap_or_else(|| initial_cursor_position(viewport, element_id, target_position))
        };
        let entering_viewport = {
            let state = self.state.lock().await;
            state.cursor_position.is_none()
        };
        let route = movement_steps(
            start_position,
            target_position,
            viewport,
            element_id,
            entering_viewport,
        );
        for step in route {
            cdp_trace::record_method("Input.dispatchMouseEvent");
            self.input_cdp(
                "move mouse",
                self.page
                    .move_mouse(Point::new(step.position.x, step.position.y)),
            )
            .await?;
            tokio::time::sleep(Duration::from_millis(step.pause_ms)).await;
        }
        let mut mouse_down =
            DispatchMouseEventParams::new(DispatchMouseEventType::MousePressed, target.x, target.y);
        mouse_down.button = Some(MouseButton::Left);
        mouse_down.click_count = Some(1);
        cdp_trace::record_method("Input.dispatchMouseEvent");
        self.input_cdp("dispatch mouse down", self.page.execute(mouse_down))
            .await?;
        tokio::time::sleep(Duration::from_millis(click_hold_ms(
            element_id,
            target_position,
            viewport,
        )))
        .await;
        let mut mouse_up = DispatchMouseEventParams::new(
            DispatchMouseEventType::MouseReleased,
            target.x,
            target.y,
        );
        mouse_up.button = Some(MouseButton::Left);
        mouse_up.click_count = Some(1);
        cdp_trace::record_method("Input.dispatchMouseEvent");
        self.input_cdp("dispatch mouse up", self.page.execute(mouse_up))
            .await?;
        self.state.lock().await.cursor_position = Some(target_position);
        Ok(())
    }

    pub(super) async fn synthetic_click(&self, element_id: u32) -> Result<()> {
        self.run_action(synthetic_click_expression(element_id), element_id)
            .await
    }

    async fn viewport_anchor(&self) -> Result<CursorPosition> {
        let metrics: ViewportMetrics = self
            .evaluate_value(
                r#"(() => ({
  width: Math.max(window.innerWidth || 0, 1),
  height: Math.max(window.innerHeight || 0, 1)
}))()"#
                    .to_string(),
            )
            .await?;
        let anchor = {
            let state = self.state.lock().await;
            state.cursor_position
        };
        Ok(anchor
            .filter(|position| {
                position.x >= 0.0
                    && position.y >= 0.0
                    && position.x < metrics.width
                    && position.y < metrics.height
            })
            .unwrap_or(CursorPosition {
                x: metrics.width * 0.5,
                y: metrics.height * 0.4,
            }))
    }

    async fn wheel_scroll(&self, direction: ScrollTarget) -> Result<()> {
        let anchor = self.viewport_anchor().await?;
        let delta_sign = match direction {
            ScrollTarget::Down => 1.0,
            ScrollTarget::Up => -1.0,
            ScrollTarget::ToElement(_) => 0.0,
        };
        let viewport_height = self
            .evaluate_value::<ViewportMetrics>(
                r#"(() => ({
  width: Math.max(window.innerWidth || 0, 1),
  height: Math.max(window.innerHeight || 0, 1)
}))()"#
                    .to_string(),
            )
            .await?
            .height;
        let deltas = wheel_deltas(delta_sign, viewport_height.max(1.0));
        for delta_y in deltas {
            let mut event = DispatchMouseEventParams::new(
                DispatchMouseEventType::MouseWheel,
                anchor.x,
                anchor.y,
            );
            event.delta_x = Some(0.0);
            event.delta_y = Some(delta_y);
            cdp_trace::record_method("Input.dispatchMouseEvent");
            self.input_cdp("dispatch wheel event", self.page.execute(event))
                .await?;
            tokio::time::sleep(Duration::from_millis(18)).await;
        }
        self.state.lock().await.cursor_position = Some(anchor);
        Ok(())
    }
}

pub(crate) fn wheel_deltas(direction_sign: f64, viewport_height: f64) -> Vec<f64> {
    let base = viewport_height * 0.92 * direction_sign;
    vec![base * 0.37, base * 0.33, base * 0.22]
}
