### Browser Trait

The browser and page functionality is exposed behind traits to support testing:

```rust
#[async_trait]
pub trait BrowserEngine: Send + Sync {
    async fn session_info(&self) -> Result<SessionInfo>;
    async fn current_page(&self) -> Result<Box<dyn PageEngine>>;
    async fn list_pages(&self) -> Result<Vec<SessionPageSummary>>;
    async fn select_page(&self, page_id: &str) -> Result<Box<dyn PageEngine>>;
    async fn new_page(&self, url: Option<&str>) -> Result<Box<dyn PageEngine>>;
    async fn close_page(&self, page_id: Option<&str>) -> Result<Box<dyn PageEngine>>;
    async fn export_checkpoint(&self) -> Result<SessionCheckpoint>;
    async fn detach(&self) -> Result<()>;
    async fn close(&self) -> Result<()>;
}

#[async_trait]
pub trait PageEngine: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn back(&self) -> Result<()>;
    async fn forward(&self) -> Result<()>;
    async fn reload(&self) -> Result<()>;
    async fn url(&self) -> Result<String>;
    async fn title(&self) -> Result<String>;
    async fn rendered_html(&self) -> Result<String>;

    async fn capture(&self) -> Result<PageCapture>;
    async fn capture_subtree(&self, element_id: u32) -> Result<Element>;
    async fn expand(&self, element_id: u32) -> Result<Element>;
    async fn metadata(&self, element_id: u32) -> Result<MetadataRecord>;
    async fn describe(&self, element_id: u32) -> Result<ImageDescription>;

    async fn click(&self, element_id: u32) -> Result<()>;
    async fn press_keys(&self, keys: &[String]) -> Result<()>;
    async fn type_text(&self, element_id: u32, text: &str) -> Result<()>;
    async fn clear(&self, element_id: u32) -> Result<()>;
    async fn select_option(&self, element_id: u32, value: &str) -> Result<()>;
    async fn submit(&self, element_id: u32) -> Result<()>;
    async fn scroll(&self, target: ScrollTarget) -> Result<()>;

    async fn screenshot(&self) -> Result<Vec<u8>>;
    async fn screenshot_element(&self, element_id: u32) -> Result<Vec<u8>>;
    async fn download(&self, url: &str, destination: &Path) -> Result<DownloadResult>;

    async fn wait_for_selector(&self, selector: &str, timeout: Duration) -> Result<()>;
    async fn wait_for_stable(&self, timeout: Duration) -> Result<()>;

    async fn evaluate_js(&self, expression: &str) -> Result<String>;
}

pub enum ScrollTarget {
    Down,
    Up,
    ToElement(u32),
}
```

`Browser::restore(config, checkpoint)` is the fresh-session composition
boundary. It validates checkpoint v1, ignores source-host identity, forces a
new Bowser-owned ephemeral profile, and returns the newly created `Browser`.
`read_checkpoint` and `write_checkpoint` provide validated, atomic plaintext
JSON file I/O; coverage and exclusions are defined in
[Checkpoints And Machine Output](checkpoints-and-machine-output.md).

`PageEngine::download` is browser-native. It runs through Chrome's current
session state, so redirects, cookies, authentication, and
`Content-Disposition` are handled by the browser rather than by a direct HTTP
client.

### Session Store Trait

Detached-session persistence is exposed as a public trait so library users can supply a custom backing store:

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn load(&self, session_id: &str) -> Result<SessionMetadata>;
    async fn save(&self, metadata: &SessionMetadata) -> Result<()>;
    async fn list(&self) -> Result<Vec<SessionMetadata>>;
    async fn remove(&self, session_id: &str) -> Result<()>;
}
```

`SessionMetadata` and `SessionPageMetadata` are public serializable structs.
Callers may persist them directly in a DB and pass a custom `SessionStore`
implementation to `Browser::launch_with_store(...)`. When Bowser starts Xvfb
for a headed Linux session, `SessionMetadata` also stores the Xvfb PID and
display string so close and expiry cleanup can validate the helper process
before terminating it. Fresh ephemeral sessions also set
`SessionMetadata::owns_user_data_dir`; legacy documents default that field to
`false`. `cleanup_expired_sessions(store, session_root, ttl)` and
`remove_owned_session_profile(session_root, metadata)` require the trusted
session root so arbitrary persisted paths cannot become recursive-deletion
targets.

### AI Summarization Trait

```rust
use std::time::Duration;

#[async_trait]
pub trait ImageSummarizer: Send + Sync {
    async fn describe(
        &self,
        image_bytes: &[u8],
        format: ImageFormat,
        timeout: Duration,
    ) -> Result<String>;
}

pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}
```

Implementations are provided for Anthropic, OpenAI, and Ollama.
Each implementation validates the provider HTTP status before parsing its JSON
payload, so authentication, throttling, and server failures remain visible in
the returned `AiSummarization` error. Implementations must apply the supplied
deadline to the complete request and return `AiSummarizationTimeout` when it
expires; `PageEngine::describe` also enforces the same outer deadline.

If the selected provider requires an API key and the configured key environment variable is unset or empty, Bowser keeps structural capture working and simply leaves explicit image description unavailable unless a cached description already exists.

Compact page YAML still labels images from `alt` plus the source filename when available. When AI is currently available, image elements add `describable: true` so callers know `describe` can be used.

`PageEngine::describe` is best-effort. If clipped screenshot capture fails, provider calls fail or time out, or image descriptions cannot be produced for any other reason, Bowser still returns the structural page capture unchanged. Regular captures never call the provider automatically. `PageEngine::describe` may return a cached description immediately or perform the screenshot-and-provider call on demand when no cached description exists.

### Library Usage Example

```rust
use bowser::{Browser, BrowserConfig, Viewport};

#[tokio::main]
async fn main() -> bowser::Result<()> {
    let config = BrowserConfig {
        viewport: Viewport { width: 1920, height: 1080 },
        stealth: true,
        ..Default::default()
    };

    let browser = Browser::launch(config).await?;
    eprintln!("Session: {}", browser.session_info().await?.id);
    let page = browser.current_page().await?;

    page.navigate("https://example.com").await?;
    page.wait_for_stable(Duration::from_secs(10)).await?;

    // get structured YAML
    let capture = page.capture().await?;
    let yaml = bowser::to_yaml(&capture)?;
    println!("{yaml}");

    // or fetch the full rendered DOM as raw HTML
    let html = page.rendered_html().await?;
    println!("{html}");

    // interact
    page.click(3).await?;    // visible targets use a pointer-driven click first; focusable controls become active
    page.wait_for_stable(Duration::from_secs(10)).await?;

    let combo = vec!["cmd".to_string(), "enter".to_string()];
    page.press_keys(&combo).await?;

    page.type_text(8, "search query").await?;
    page.submit(8).await?;
    page.wait_for_stable(Duration::from_secs(10)).await?;

    // screenshot
    let png = page.screenshot().await?;
    std::fs::write("page.png", &png)?;

    // re-capture after interactions
    let updated = page.capture().await?;
    let yaml = bowser::to_yaml(&updated)?;
    println!("{yaml}");

    let image = page.describe(15).await?;
    println!("{}", bowser::image_description_to_yaml(&image)?);

    browser.detach().await?;
    Ok(())
}
```

### Serialization

The library provides:

```rust
/// Serialize a PageCapture to compact display YAML.
pub fn to_yaml(capture: &PageCapture) -> Result<String>;

/// Serialize a PageCapture to explicit JSON mirroring the Rust capture types.
pub fn to_json(capture: &PageCapture) -> Result<String>;

/// Deserialize a PageCapture from explicit JSON.
pub fn from_json(json: &str) -> Result<PageCapture>;

/// Read or atomically write a validated checkpoint v1 JSON document.
pub fn read_checkpoint(path: &Path) -> Result<SessionCheckpoint>;
pub fn write_checkpoint(path: &Path, checkpoint: &SessionCheckpoint) -> Result<()>;

/// Serialize an image description to YAML.
pub fn image_description_to_yaml(description: &ImageDescription) -> Result<String>;
```
