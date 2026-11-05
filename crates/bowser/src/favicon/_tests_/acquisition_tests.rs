//! Favicon acquisition, fallback, and identity regressions.

use std::sync::Arc;

use png::{BitDepth, ColorType, Encoder};
use tokio::time::{Duration, Instant};
use unimock::{MockFn, Unimock, matching};

use super::{DefaultFaviconAcquirer, FaviconAcquirer, ReadFaviconMock, SnapshotFaviconMock};
use crate::favicon::candidate::{FaviconDeclaration, FaviconDocument};
use crate::model::BrowserFavicon;

#[tokio::test]
async fn tries_failed_declarations_before_a_successful_candidate() {
    let snapshot = document(&[("missing.png", "32x32"), ("working.png", "any")]);
    let reader = Unimock::new((
        SnapshotFaviconMock
            .next_call(matching!())
            .returns(Some(snapshot.clone())),
        ReadFaviconMock
            .next_call(matching!("https://site.test/missing.png"))
            .returns(None),
        ReadFaviconMock
            .next_call(matching!("https://site.test/working.png"))
            .returns(Some(one_pixel_png())),
        SnapshotFaviconMock
            .next_call(matching!())
            .returns(Some(snapshot)),
    ));

    assert!(acquire(&reader).await.is_some());
}

#[tokio::test]
async fn rejects_icons_when_the_document_or_declaration_changes() {
    let original = document(&[("icon.png", "32x32")]);
    let mut changed = original.clone();
    changed.declarations[0].href = "replacement.png".to_string();
    let reader = Unimock::new((
        SnapshotFaviconMock
            .next_call(matching!())
            .returns(Some(original)),
        ReadFaviconMock
            .next_call(matching!("https://site.test/icon.png"))
            .returns(Some(one_pixel_png())),
        SnapshotFaviconMock
            .next_call(matching!())
            .returns(Some(changed)),
    ));

    assert!(acquire(&reader).await.is_none());
}

#[tokio::test]
async fn broken_candidates_and_root_fallback_fail_soft() {
    let snapshot = document(&[("broken.png", "32x32")]);
    let reader = Unimock::new((
        SnapshotFaviconMock
            .next_call(matching!())
            .returns(Some(snapshot)),
        ReadFaviconMock
            .next_call(matching!("https://site.test/broken.png"))
            .returns(None),
        ReadFaviconMock
            .next_call(matching!("https://site.test/favicon.ico"))
            .returns(None),
    ));

    assert!(acquire(&reader).await.is_none());
}

async fn acquire(reader: &Unimock) -> Option<BrowserFavicon> {
    DefaultFaviconAcquirer
        .acquire(
            Arc::new(reader.clone()),
            Instant::now() + Duration::from_secs(1),
        )
        .await
}

fn document(declarations: &[(&str, &str)]) -> FaviconDocument {
    FaviconDocument {
        loader_id: "loader".to_string(),
        url: "https://site.test/page".to_string(),
        base_uri: "https://site.test/".to_string(),
        declarations: declarations
            .iter()
            .map(|(href, sizes)| FaviconDeclaration {
                href: (*href).to_string(),
                sizes: (*sizes).to_string(),
            })
            .collect(),
    }
}

fn one_pixel_png() -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = Encoder::new(&mut bytes, 1, 1);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header");
        writer
            .write_image_data(&[20, 40, 60, 255])
            .expect("PNG pixels");
        writer.finish().expect("PNG finish");
    }
    bytes
}
