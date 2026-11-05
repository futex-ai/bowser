//! Bounded favicon decoding and 32-pixel PNG normalization.

use std::io::Cursor;

use base64::{Engine, engine::general_purpose::STANDARD};
use image::codecs::png::PngEncoder;
use image::imageops::{FilterType, overlay};
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageFormat, ImageReader, Limits};
use resvg::{tiny_skia, usvg};

use crate::favicon::resource::MAX_SOURCE_BYTES;
use crate::favicon::svg_guard::bounded_svg;
use crate::model::BrowserFavicon;

const OUTPUT_SIZE: u32 = 32;
const MAX_INPUT_DIMENSION: u32 = 1024;
const MAX_DECODE_ALLOCATION_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const MAX_FAVICON_BASE64_BYTES: usize = 6 * 1024;

pub(crate) fn normalize_favicon(source: &[u8]) -> Option<BrowserFavicon> {
    if source.is_empty() || source.len() > MAX_SOURCE_BYTES {
        return None;
    }
    let png = decode_raster(source)
        .and_then(letterbox_raster)
        .or_else(|| render_svg(source))?;
    let png_base64 = STANDARD.encode(png);
    (png_base64.len() <= MAX_FAVICON_BASE64_BYTES).then_some(BrowserFavicon { png_base64 })
}

fn decode_raster(source: &[u8]) -> Option<DynamicImage> {
    let mut reader = ImageReader::new(Cursor::new(source))
        .with_guessed_format()
        .ok()?;
    let format = reader.format()?;
    if !matches!(
        format,
        ImageFormat::Png
            | ImageFormat::Ico
            | ImageFormat::Jpeg
            | ImageFormat::Gif
            | ImageFormat::WebP
    ) {
        return None;
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_INPUT_DIMENSION);
    limits.max_image_height = Some(MAX_INPUT_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOCATION_BYTES);
    reader.limits(limits);
    reader.decode().ok()
}

fn letterbox_raster(source: DynamicImage) -> Option<Vec<u8>> {
    let width = source.width();
    let height = source.height();
    if width == 0 || height == 0 || width > MAX_INPUT_DIMENSION || height > MAX_INPUT_DIMENSION {
        return None;
    }
    let (target_width, target_height) = fit_dimensions(width, height);
    let resized = source
        .resize_exact(target_width, target_height, FilterType::Lanczos3)
        .to_rgba8();
    let mut canvas = image::RgbaImage::new(OUTPUT_SIZE, OUTPUT_SIZE);
    overlay(
        &mut canvas,
        &resized,
        i64::from((OUTPUT_SIZE - target_width) / 2),
        i64::from((OUTPUT_SIZE - target_height) / 2),
    );
    encode_rgba_png(canvas.as_raw())
}

fn fit_dimensions(width: u32, height: u32) -> (u32, u32) {
    if width >= height {
        (OUTPUT_SIZE, (height * OUTPUT_SIZE / width).max(1))
    } else {
        ((width * OUTPUT_SIZE / height).max(1), OUTPUT_SIZE)
    }
}

fn encode_rgba_png(pixels: &[u8]) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    PngEncoder::new(&mut output)
        .write_image(pixels, OUTPUT_SIZE, OUTPUT_SIZE, ExtendedColorType::Rgba8)
        .ok()?;
    Some(output)
}

fn render_svg(source: &[u8]) -> Option<Vec<u8>> {
    bounded_svg(source)?;
    let options = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(source, &options).ok()?;
    let size = tree.size();
    let width = size.width();
    let height = size.height();
    if width <= 0.0
        || height <= 0.0
        || width > MAX_INPUT_DIMENSION as f32
        || height > MAX_INPUT_DIMENSION as f32
    {
        return None;
    }
    let scale = (OUTPUT_SIZE as f32 / width).min(OUTPUT_SIZE as f32 / height);
    let translated_x = (OUTPUT_SIZE as f32 - width * scale) / 2.0;
    let translated_y = (OUTPUT_SIZE as f32 - height * scale) / 2.0;
    let transform =
        tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, translated_x, translated_y);
    let mut pixmap = tiny_skia::Pixmap::new(OUTPUT_SIZE, OUTPUT_SIZE)?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap.encode_png().ok()
}

#[cfg(test)]
#[path = "_tests_/image_tests.rs"]
mod image_tests;
