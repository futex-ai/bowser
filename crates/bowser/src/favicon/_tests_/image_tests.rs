//! Bounded image normalization and SVG rendering regressions.

use std::io::Cursor;

use base64::{Engine, engine::general_purpose::STANDARD};
use png::{BitDepth, ColorType, Decoder, Encoder};

use super::{MAX_FAVICON_BASE64_BYTES, normalize_favicon};

#[test]
fn normalizes_raster_images_to_bounded_rgba_png() {
    let source = solid_png(64, 32, [200, 20, 10, 255]);
    let favicon = normalize_favicon(&source).expect("normalized favicon");

    assert!(favicon.png_base64.len() <= MAX_FAVICON_BASE64_BYTES);
    let bytes = STANDARD
        .decode(&favicon.png_base64)
        .expect("canonical favicon base64");
    let decoder = Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("favicon PNG info");
    let mut pixels = vec![0; reader.output_buffer_size().expect("PNG buffer")];
    let info = reader.next_frame(&mut pixels).expect("favicon PNG frame");
    assert_eq!((info.width, info.height), (32, 32));
    assert_eq!(info.color_type, ColorType::Rgba);
    let top_center = ((16 * 4) as usize)..((16 * 4 + 4) as usize);
    let middle_center = (((16 * 32 + 16) * 4) as usize)..(((16 * 32 + 16) * 4 + 4) as usize);
    assert_eq!(&pixels[top_center], &[0, 0, 0, 0]);
    assert_eq!(&pixels[middle_center], &[200, 20, 10, 255]);
}

#[test]
fn rejects_oversized_and_malformed_rasters() {
    assert!(normalize_favicon(&solid_png(1025, 1, [1, 2, 3, 255])).is_none());
    assert!(normalize_favicon(b"not an image").is_none());
}

#[test]
fn rejects_expanding_svg_references_before_rendering() {
    let mut svg = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' width='32' height='32'><defs><g id='a0'><rect width='1' height='1'/></g>",
    );
    for depth in 1..=4 {
        let parent = depth - 1;
        svg.push_str(&format!(
            "<g id='a{depth}'><use href='#a{parent}'/><use href='#a{parent}'/></g>",
        ));
    }
    svg.push_str("</defs><use href='#a4'/></svg>");
    assert!(normalize_favicon(svg.as_bytes()).is_none());
}

#[test]
fn rejects_svg_with_active_or_embedded_content() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32">
      <script>throw new Error('must not run')</script>
      <image href="https://invalid.test/tracker.png" width="64" height="32"/>
      <rect width="64" height="32" fill="#ff0000"/>
    </svg>"##;

    assert!(normalize_favicon(svg).is_none());
}

#[test]
fn rasterizes_basic_svg_shapes_and_gradients() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32">
      <defs><linearGradient id="a"><stop stop-color="red"/>
      <stop offset="1" stop-color="blue"/></linearGradient></defs>
      <path d="M0 0H64V32H0Z" fill="url(#a)"/>
    </svg>"##;
    let favicon = normalize_favicon(svg).expect("normalized SVG gradient");
    assert!(favicon.png_base64.len() <= MAX_FAVICON_BASE64_BYTES);
}

fn solid_png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().expect("PNG header");
    writer
        .write_image_data(&color.repeat((width * height) as usize))
        .expect("PNG pixels");
    writer.finish().expect("PNG finish");
    bytes
}
