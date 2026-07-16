//! Pointer telemetry request and log-record types.

use serde::{Deserialize, Serialize};

/// Maximum accepted events in one browser POST.
pub(super) const MAX_BATCH_EVENTS: usize = 2048;
const MAX_TARGET_FIELD_LEN: usize = 160;

/// Browser viewport metadata captured with each event batch.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PointerViewport {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) device_pixel_ratio: f64,
}

/// Event kind emitted by the local capture page.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PointerEventKind {
    PointerMove,
    PointerDown,
    PointerUp,
    Click,
    Wheel,
    TargetSpawn,
    VisibilityChange,
    PageHide,
}

/// DOM target summary for a recorded pointer event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PointerTarget {
    pub(super) tag: String,
    pub(super) id: Option<String>,
    pub(super) role: Option<String>,
    pub(super) label: Option<String>,
    pub(super) classes: Option<String>,
}

/// Bounding box for the target element in viewport coordinates.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PointerTargetRect {
    pub(super) left: f64,
    pub(super) top: f64,
    pub(super) width: f64,
    pub(super) height: f64,
}

/// A single browser input or lifecycle event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PointerEvent {
    pub(super) kind: PointerEventKind,
    pub(super) time_ms: f64,
    pub(super) elapsed_ms: f64,
    pub(super) x: Option<f64>,
    pub(super) y: Option<f64>,
    pub(super) movement_x: Option<f64>,
    pub(super) movement_y: Option<f64>,
    pub(super) delta_x: Option<f64>,
    pub(super) delta_y: Option<f64>,
    pub(super) button: Option<i16>,
    pub(super) buttons: Option<u16>,
    pub(super) pointer_type: Option<String>,
    pub(super) target: PointerTarget,
    pub(super) target_rect: Option<PointerTargetRect>,
}

/// Batch posted by the local capture page.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PointerEventBatch {
    pub(super) page_started_at_ms: u64,
    pub(super) viewport: PointerViewport,
    pub(super) events: Vec<PointerEvent>,
}

/// JSON response returned after accepting a batch.
#[derive(Debug, Serialize)]
pub(super) struct PointerLogAccepted {
    pub(super) accepted: usize,
}

/// One newline-delimited record persisted to the event log.
#[derive(Debug, Serialize)]
pub(super) struct PointerLogRecord<'a> {
    pub(super) schema_version: u8,
    pub(super) page_started_at_ms: u64,
    pub(super) viewport: &'a PointerViewport,
    pub(super) event: &'a PointerEvent,
}

/// Pointer telemetry validation failures.
#[derive(Debug, thiserror::Error)]
pub(super) enum PointerEventValidationError {
    #[error("[bowser-cli/pointer-log] event batch cannot be empty")]
    EmptyBatch,

    #[error("[bowser-cli/pointer-log] event batch has {count} events, max is {max}")]
    BatchTooLarge { count: usize, max: usize },

    #[error("[bowser-cli/pointer-log] field {field} has length {len}, max is {max}")]
    FieldTooLong {
        field: &'static str,
        len: usize,
        max: usize,
    },

    #[error("[bowser-cli/pointer-log] field {field} must be finite")]
    NonFiniteNumber { field: &'static str },

    #[error("[bowser-cli/pointer-log] viewport width and height must be positive")]
    EmptyViewport,
}

impl PointerEventBatch {
    /// Validates the batch before it is persisted.
    pub(super) fn validate(&self) -> std::result::Result<(), PointerEventValidationError> {
        if self.events.is_empty() {
            return Err(PointerEventValidationError::EmptyBatch);
        }
        if self.events.len() > MAX_BATCH_EVENTS {
            return Err(PointerEventValidationError::BatchTooLarge {
                count: self.events.len(),
                max: MAX_BATCH_EVENTS,
            });
        }
        self.viewport.validate()?;
        for event in &self.events {
            event.validate()?;
        }
        Ok(())
    }
}

impl PointerViewport {
    fn validate(&self) -> std::result::Result<(), PointerEventValidationError> {
        if self.width == 0 || self.height == 0 {
            return Err(PointerEventValidationError::EmptyViewport);
        }
        ensure_finite("viewport.device_pixel_ratio", Some(self.device_pixel_ratio))
    }
}

impl PointerEvent {
    fn validate(&self) -> std::result::Result<(), PointerEventValidationError> {
        ensure_finite("time_ms", Some(self.time_ms))?;
        ensure_finite("elapsed_ms", Some(self.elapsed_ms))?;
        ensure_finite("x", self.x)?;
        ensure_finite("y", self.y)?;
        ensure_finite("movement_x", self.movement_x)?;
        ensure_finite("movement_y", self.movement_y)?;
        ensure_finite("delta_x", self.delta_x)?;
        ensure_finite("delta_y", self.delta_y)?;
        if let Some(pointer_type) = self.pointer_type.as_deref() {
            ensure_len("pointer_type", pointer_type, MAX_TARGET_FIELD_LEN)?;
        }
        self.target.validate()?;
        if let Some(target_rect) = &self.target_rect {
            target_rect.validate()?;
        }
        Ok(())
    }
}

impl PointerTarget {
    fn validate(&self) -> std::result::Result<(), PointerEventValidationError> {
        ensure_len("target.tag", &self.tag, MAX_TARGET_FIELD_LEN)?;
        ensure_optional_len("target.id", self.id.as_deref())?;
        ensure_optional_len("target.role", self.role.as_deref())?;
        ensure_optional_len("target.label", self.label.as_deref())?;
        ensure_optional_len("target.classes", self.classes.as_deref())
    }
}

impl PointerTargetRect {
    fn validate(&self) -> std::result::Result<(), PointerEventValidationError> {
        ensure_finite("target_rect.left", Some(self.left))?;
        ensure_finite("target_rect.top", Some(self.top))?;
        ensure_finite("target_rect.width", Some(self.width))?;
        ensure_finite("target_rect.height", Some(self.height))?;
        Ok(())
    }
}

impl<'a> PointerLogRecord<'a> {
    /// Creates a persisted record for one event inside a posted batch.
    pub(super) fn new(batch: &'a PointerEventBatch, event: &'a PointerEvent) -> Self {
        Self {
            schema_version: 2,
            page_started_at_ms: batch.page_started_at_ms,
            viewport: &batch.viewport,
            event,
        }
    }
}

fn ensure_len(
    field: &'static str,
    value: &str,
    max: usize,
) -> std::result::Result<(), PointerEventValidationError> {
    let len = value.chars().count();
    if len > max {
        Err(PointerEventValidationError::FieldTooLong { field, len, max })
    } else {
        Ok(())
    }
}

fn ensure_optional_len(
    field: &'static str,
    value: Option<&str>,
) -> std::result::Result<(), PointerEventValidationError> {
    if let Some(value) = value {
        ensure_len(field, value, MAX_TARGET_FIELD_LEN)
    } else {
        Ok(())
    }
}

fn ensure_finite(
    field: &'static str,
    value: Option<f64>,
) -> std::result::Result<(), PointerEventValidationError> {
    if let Some(value) = value
        && !value.is_finite()
    {
        return Err(PointerEventValidationError::NonFiniteNumber { field });
    }
    Ok(())
}
