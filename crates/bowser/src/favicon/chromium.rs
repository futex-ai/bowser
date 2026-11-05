//! Chromium-backed favicon document inspection and resource reads.

use std::sync::Arc;

use async_trait::async_trait;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::page::GetFrameTreeParams;
use serde::Deserialize;
use tokio::time::Instant;

use crate::favicon::acquisition::FaviconResourceReader;
use crate::favicon::candidate::{FaviconDeclaration, FaviconDocument};
use crate::favicon::resource::decode_data_url;
use crate::favicon::transport::loader::BrowserResourceLoader;

const SNAPSHOT_SCRIPT: &str = r#"(() => {
  const declarations = [];
  let order = 0;
  for (const link of document.querySelectorAll('link')) {
    const rel = (link.getAttribute('rel') || '').toLowerCase().split(/\s+/);
    const media = (link.getAttribute('media') || '').trim();
    if (!rel.includes('icon') || link.disabled === true) continue;
    if (media && !matchMedia(media).matches) continue;
    const href = (link.getAttribute('href') || '').trim();
    if (!href) continue;
    let address;
    try { address = new URL(href, document.baseURI).href; } catch { continue; }
    if (!/^(https?:|data:)/.test(address) || address.length > 1048576) continue;
    const sizes = link.getAttribute('sizes') || '';
    const tokens = sizes.toLowerCase().split(/\s+/);
    const rank = tokens.includes('32x32') ? 0 : tokens.includes('any') ? 1 : 2;
    const candidate = { href: address, sizes, rank, order: order++ };
    const duplicate = declarations.findIndex((item) => item.href === address);
    if (duplicate >= 0 && declarations[duplicate].rank < rank) continue;
    if (duplicate >= 0) declarations.splice(duplicate, 1);
    declarations.push(candidate);
    declarations.sort((left, right) => left.rank - right.rank || right.order - left.order);
    if (declarations.length > 3) declarations.pop();
  }
  declarations.sort((left, right) => left.order - right.order);
  return { url: location.href, base_uri: document.baseURI, declarations };
})()"#;

pub(crate) struct ChromiumFaviconReader {
    deadline: Instant,
    expected_url: String,
    loader: Arc<dyn BrowserResourceLoader>,
    page: Page,
}

impl ChromiumFaviconReader {
    pub(crate) fn new(
        page: Page,
        expected_url: String,
        deadline: Instant,
        loader: Arc<dyn BrowserResourceLoader>,
    ) -> Self {
        Self {
            deadline,
            expected_url,
            loader,
            page,
        }
    }
}

#[async_trait]
impl FaviconResourceReader for ChromiumFaviconReader {
    async fn snapshot(&self) -> Option<FaviconDocument> {
        let frame_tree = self
            .page
            .execute(GetFrameTreeParams::default())
            .await
            .ok()?;
        let snapshot = self
            .page
            .evaluate(SNAPSHOT_SCRIPT)
            .await
            .ok()?
            .into_value::<Snapshot>()
            .ok()?;
        if snapshot.url != self.expected_url {
            return None;
        }
        Some(FaviconDocument {
            loader_id: frame_tree.frame_tree.frame.loader_id.as_ref().to_string(),
            url: snapshot.url,
            base_uri: snapshot.base_uri,
            declarations: snapshot.declarations,
        })
    }

    async fn read(&self, address: &str) -> Option<Vec<u8>> {
        if Instant::now() >= self.deadline {
            return None;
        }
        if address.starts_with("data:") {
            return decode_data_url(address);
        }
        self.loader.load(address).await
    }
}

#[derive(Deserialize)]
struct Snapshot {
    url: String,
    base_uri: String,
    declarations: Vec<FaviconDeclaration>,
}
