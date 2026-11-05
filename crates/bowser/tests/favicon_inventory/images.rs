//! Bounded real image fixtures for browser inventory tests.

use std::io::Cursor;

use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use image::codecs::ico::IcoEncoder;
use image::{ExtendedColorType, ImageEncoder};
use png::{BitDepth, ColorType, Encoder};

pub(super) async fn red_png_response() -> Response {
    image_response("image/png", solid_png([220, 20, 20, 255]))
}

pub(super) async fn green_png_response() -> Response {
    image_response("image/png", solid_png([20, 180, 40, 255]))
}

pub(super) async fn svg_response() -> Response {
    image_response(
        "image/svg+xml",
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"><rect width="64" height="32" fill="#ff8800"/></svg>"##.to_vec(),
    )
}

pub(super) async fn ico_response() -> Response {
    let pixels = [40, 190, 210, 255].repeat(32 * 32);
    let mut bytes = Vec::new();
    IcoEncoder::new(&mut bytes)
        .write_image(&pixels, 32, 32, ExtendedColorType::Rgba8)
        .expect("encode ICO fixture");
    image_response("image/x-icon", bytes)
}

pub(super) fn image_response(content_type: &'static str, bytes: Vec<u8>) -> Response {
    (
        [(CONTENT_TYPE, content_type), (CACHE_CONTROL, "no-store")],
        bytes,
    )
        .into_response()
}

pub(super) fn solid_png(color: [u8; 4]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(Cursor::new(&mut bytes), 32, 32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().expect("PNG header");
    writer
        .write_image_data(&color.repeat(32 * 32))
        .expect("PNG pixels");
    writer.finish().expect("PNG finish");
    bytes
}
