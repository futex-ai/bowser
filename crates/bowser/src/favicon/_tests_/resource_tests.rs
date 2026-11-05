//! Bounded data URL resource decoding regressions.

use base64::{Engine, engine::general_purpose::STANDARD};

use super::{MAX_SOURCE_BYTES, decode_data_url};

#[test]
fn decodes_base64_and_percent_encoded_data_images() {
    assert_eq!(decode_data_url("data:image/png;base64,AA=="), Some(vec![0]));
    assert_eq!(
        decode_data_url("data:image/svg+xml,%3Csvg%20xmlns%3D%22x%22%2F%3E"),
        Some(br#"<svg xmlns="x"/>"#.to_vec())
    );
}

#[test]
fn rejects_oversized_or_non_image_data_resources() {
    let oversized = STANDARD.encode(vec![0; MAX_SOURCE_BYTES + 1]);

    assert!(decode_data_url(&format!("data:image/png;base64,{oversized}")).is_none());
    assert!(decode_data_url("data:text/html,%3Cb%3Eno%3C%2Fb%3E").is_none());
    assert!(decode_data_url("https://site.test/icon.png").is_none());
}

#[test]
fn rejects_invalid_base64_without_panicking() {
    assert!(decode_data_url("data:image/png;base64,!not-base64!").is_none());
}
