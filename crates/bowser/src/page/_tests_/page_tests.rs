use std::time::Duration;

use super::evaluate::{INPUT_CDP_TIMEOUT, mix_typing_seed, typing_delay_ms};
use super::lifecycle::should_refresh_live_ids;
use super::viewport::wheel_deltas;
use crate::error::Error;
use crate::keyboard::resolve_pressed_keys;

async fn test_input_cdp<F, T, E>(
    operation: &str,
    timeout: Duration,
    future: F,
) -> crate::error::Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => Err(Error::cdp(format!("failed to {operation}: {err}"))),
        Err(_) => Err(Error::cdp(format!(
            "timed out while trying to {operation} after {}ms",
            timeout.as_millis()
        ))),
    }
}

#[test]
fn typing_delays_are_seeded_but_not_constant() {
    let mut seed = mix_typing_seed(42);
    let delays = [
        typing_delay_ms(&mut seed),
        typing_delay_ms(&mut seed),
        typing_delay_ms(&mut seed),
        typing_delay_ms(&mut seed),
    ];
    assert!(delays.into_iter().all(|delay| (14..=30).contains(&delay)));
    assert!(delays.windows(2).any(|window| window[0] != window[1]));
}

#[test]
fn refreshes_live_ids_only_when_capture_is_stale_and_idle() {
    assert!(should_refresh_live_ids(true, false));
    assert!(!should_refresh_live_ids(true, true));
    assert!(!should_refresh_live_ids(false, false));
}

#[test]
fn resolves_modifier_and_special_key_aliases() {
    let keys = resolve_pressed_keys(&[
        "cmd".to_string(),
        "enter".to_string(),
        "space".to_string(),
        "k".to_string(),
    ])
    .expect("keys");
    assert_eq!(keys[0].key, "Meta");
    assert!(keys[0].is_modifier());
    assert_eq!(keys[1].key, "Enter");
    assert_eq!(keys[1].event_text().as_deref(), Some("\r"));
    assert_eq!(keys[2].code, "Space");
    assert_eq!(keys[3].code, "KeyK");
}

#[test]
fn wheel_scroll_uses_multiple_bursts() {
    let deltas = wheel_deltas(1.0, 900.0);
    assert_eq!(deltas.len(), 3);
    assert!(deltas.iter().all(|delta| *delta > 0.0));
    assert!(deltas.windows(2).all(|window| window[0] != window[1]));
}

#[tokio::test]
async fn input_cdp_returns_success_before_timeout() {
    let result = test_input_cdp("dispatch mouse down", INPUT_CDP_TIMEOUT, async {
        Ok::<_, &'static str>(7_u8)
    })
    .await
    .expect("success");
    assert_eq!(result, 7);
}

#[tokio::test]
async fn input_cdp_times_out_bounded_operations() {
    let err = test_input_cdp("move mouse", Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<_, &'static str>(())
    })
    .await
    .expect_err("timeout");
    assert!(matches!(err, Error::Cdp { .. }));
    assert!(
        err.to_string()
            .contains("timed out while trying to move mouse")
    );
}
