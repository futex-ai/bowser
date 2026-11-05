//! Bounded data-resource decoding.

use base64::{Engine, engine::general_purpose::STANDARD};
use percent_encoding::percent_decode_str;

pub(crate) const MAX_SOURCE_BYTES: usize = 256 * 1024;
const MAX_BASE64_PAYLOAD_BYTES: usize = MAX_SOURCE_BYTES.div_ceil(3) * 4;
const MAX_PERCENT_PAYLOAD_BYTES: usize = MAX_SOURCE_BYTES * 3;

pub(crate) fn decode_data_url(address: &str) -> Option<Vec<u8>> {
    let payload = address.strip_prefix("data:")?;
    let (metadata, data) = payload.split_once(',')?;
    let mut metadata_parts = metadata.split(';');
    let media_type = metadata_parts.next()?.trim();
    if !media_type.to_ascii_lowercase().starts_with("image/") {
        return None;
    }
    let base64_encoded = metadata_parts.any(|part| part.eq_ignore_ascii_case("base64"));
    let bytes = if base64_encoded {
        if data.len() > MAX_BASE64_PAYLOAD_BYTES {
            return None;
        }
        STANDARD.decode(data).ok()?
    } else {
        if data.len() > MAX_PERCENT_PAYLOAD_BYTES {
            return None;
        }
        percent_decode_str(data).collect()
    };
    (bytes.len() <= MAX_SOURCE_BYTES).then_some(bytes)
}

#[cfg(test)]
#[path = "_tests_/resource_tests.rs"]
mod resource_tests;
