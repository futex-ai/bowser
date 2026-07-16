## Error Handling

Bowser's public error contract is the typed `Error` enum below. Because
`bowser` is a reusable public library crate, external consumers are part of the
caller contract, so browser, session, page, and interaction failures remain
stable variants that consumers can match. Backend failures are translated at
their boundary into the closest Bowser-owned variant, such as `Cdp`, `Io`, or
`AiSummarization`, with enough context to diagnose the failed operation.

### Library Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("[bowser/browser] failed to launch Chrome: {reason}")]
    BrowserLaunch { reason: String },

    #[error("[bowser/browser] Chrome connection lost")]
    BrowserDisconnected,

    #[error("[bowser/browser] page target session invalid: {reason}")]
    PageTargetSessionInvalid { reason: String },

    #[error("[bowser/browser] failed to close page {page_id}: {reason}")]
    PageClose { page_id: String, reason: String },

    #[error("[bowser/browser] timed out closing page {page_id} after {seconds}s")]
    PageCloseTimeout { page_id: String, seconds: u64 },

    #[error("[bowser/session] session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("[bowser/session] session expired: {session_id}")]
    SessionExpired { session_id: String },

    #[error("[bowser/session] invalid session identifier: {session_id}")]
    InvalidSessionId { session_id: String },

    #[error(
        "[bowser/session] metadata identifier mismatch: expected {expected_session_id}, found {actual_session_id}"
    )]
    SessionIdMismatch {
        expected_session_id: String,
        actual_session_id: String,
    },

    #[error("[bowser/session] page not found: {page_id}")]
    SessionPageNotFound { page_id: String },

    #[error("[bowser/session] page is no longer live: {page_id}")]
    SessionPageNotLive { page_id: String },

    #[error("[bowser/session] failed to persist session metadata: {reason}")]
    SessionPersist { reason: String },

    #[error("[bowser/session] failed to detach session: {reason}")]
    SessionDetach { reason: String },

    #[error("[bowser/expand] element not expandable: {element_id}")]
    ExpandNotFound { element_id: u32 },

    #[error("[bowser/meta] metadata unavailable for element: {element_id}")]
    MetadataNotFound { element_id: u32 },

    #[error("[bowser/describe] element is not an image: {element_id}")]
    DescribeNotImage { element_id: u32 },

    #[error("[bowser/describe] image description unavailable for element: {element_id}")]
    DescribeUnavailable { element_id: u32 },

    #[error("[bowser/page] navigation failed: {url}")]
    Navigation { url: String },

    #[error("[bowser/page] page load timed out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("[bowser/page] element not found: {element_id}")]
    ElementNotFound { element_id: u32 },

    #[error("[bowser/page] element {element_id} is not interactable")]
    ElementNotInteractable { element_id: u32 },

    #[error("[bowser/page] key press failed: {reason}")]
    KeyPress { reason: String },

    #[error("[bowser/capture] DOM capture script failed: {reason}")]
    CaptureScript { reason: String },

    #[error("[bowser/capture] failed to parse capture result")]
    CaptureParse,

    #[error("[bowser/stealth] failed to inject stealth patches: {reason}")]
    StealthInjection { reason: String },

    #[error("[bowser/ai] image summarization failed: {reason}")]
    AiSummarization { reason: String },

    #[error("[bowser/config] invalid configuration: {reason}")]
    Config { reason: String },

    #[error("[bowser/screenshot] screenshot failed: {reason}")]
    Screenshot { reason: String },

    #[error("[bowser/download] invalid destination {path}: {reason}")]
    DownloadDestination { path: String, reason: String },

    #[error("[bowser/download] download failed: {reason}")]
    Download { reason: String },

    #[error("[bowser/download] download timed out after {seconds}s")]
    DownloadTimeout { seconds: u64 },

    #[error("[bowser/js] JavaScript evaluation failed: {reason}")]
    JsEvaluation { reason: String },

    #[error("[bowser/io] {operation}: {source}")]
    Io {
        operation: String,
        source: std::io::Error,
    },

    #[error("[bowser/chrome] CDP protocol error: {reason}")]
    Cdp { reason: String },
}

pub type Result<T> = std::result::Result<T, Error>;
```

### CLI Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("[bowser-cli] {0}")]
    Bowser(#[from] bowser::Error),

    #[error("[bowser-cli/io] failed to write output: {path}")]
    OutputWrite { path: String },

    #[error("[bowser-cli/config] failed to load config: {path}")]
    ConfigLoad { path: String },

    #[error("[bowser-cli/repl] invalid command: {input}")]
    InvalidCommand { input: String },

    #[error("[bowser-cli/repl] readline failed: {reason}")]
    Readline { reason: String },

    #[error("[bowser-cli/repl] failed to install signal handler: {reason}")]
    SignalHandler { reason: String },

    #[error("[bowser-cli/pointer-log] bind address must be loopback: {addr}")]
    PointerLogNonLoopback { addr: SocketAddr },

    #[error("[bowser-cli/pointer-log] failed to bind server at {addr}")]
    PointerLogBind {
        addr: SocketAddr,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to read bound server address")]
    PointerLogLocalAddr { source: std::io::Error },

    #[error("[bowser-cli/pointer-log] server failed at {addr}")]
    PointerLogServe {
        addr: SocketAddr,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to create log directory: {path}")]
    PointerLogCreateDir {
        path: String,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to open log file: {path}")]
    PointerLogOpen {
        path: String,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to lock log file: {path}")]
    PointerLogLock { path: String },

    #[error("[bowser-cli/pointer-log] failed to serialize event for log file: {path}")]
    PointerLogSerialize {
        path: String,
        source: serde_json::Error,
    },

    #[error("[bowser-cli/pointer-log] failed to append log file: {path}")]
    PointerLogAppend {
        path: String,
        source: std::io::Error,
    },

    #[error("[bowser-cli/pointer-log] demo target was not found in the current capture")]
    PointerLogDemoTargetMissing,

    #[error("[bowser-cli/pointer-log] demo browser config was not loaded")]
    PointerLogDemoConfigMissing,

    #[error("[bowser-cli/pointer-log] failed to listen for Ctrl-C")]
    PointerLogSignal { source: std::io::Error },
}
```

The CLI preserves Bowser library errors through `CliError::Bowser` and adds
typed command-layer failures for output, REPL, and pointer-log handling.

## Dependencies

### Library Crate (`bowser`)

| Crate | Purpose |
|---|---|
| `chromiumoxide` | Chrome DevTools Protocol client |
| `tokio` | Async runtime |
| `serde`, `serde_json`, `serde_yaml` | Serialization |
| `thiserror` | Error types |
| `reqwest` | HTTP client for AI API calls |
| `tracing` | Structured logging |
| `which` | Chrome binary auto-detection |
| `dirs` | Platform config directory resolution |

### CLI Crate (`bowser-cli`)

| Crate | Purpose |
|---|---|
| `bowser` (workspace) | Core library |
| `clap` (derive) | CLI argument parsing |
| `tokio` | Async runtime |
| `rustyline` | REPL line editing and history |
| `tracing-subscriber` | Log output |
| `thiserror` | Error types |

## Testing Strategy

### Unit Tests

- **Capture/flattening logic**: test the DOM-to-element-tree transformation with mock DOM JSON inputs. Verify flattening rules, root visible/obscured splitting, wrapper promotion, and text merging.
- **YAML/JSON serialization**: snapshot tests for compact YAML output and full-fidelity round-trip tests for JSON.
- **Config parsing**: test precedence rules, defaults, and invalid config handling.
- **Session lifecycle**: test session ID generation, path confinement, filename/document identity, metadata persistence, TTL expiry, resume behaviour, fresh-launch rollback, and stale-PID-safe cleanup through mocked process control.
- **Session page management**: test stable page-ID assignment, selected-page persistence, legacy single-page metadata migration, and page-summary generation.
- **Truncation logic**: test preview limits for lists, tables, and container children, plus emitted `truncation` metadata.
- **Stealth patch generation**: verify the JS patches are syntactically valid.
- **Expansion handling**: test expandable-element ID assignment, invalid-element-ID errors, and full-node expansion after truncated capture.
- **Metadata and describe handling**: test `meta` lookups for links, images, and non-interactive containers, including live visibility details, `describable` capability, explicit `describe` output, and invalid-element-ID errors.
- **REPL command parsing**: test all interactive commands parse correctly, including edge cases (quoted strings, missing arguments), plus prompt interrupt and suggestion behaviour.

Unit tests focus on pure helpers, serialization, config merging, session persistence, REPL parsing/completion, and AI provider plumbing with mocked local HTTP endpoints.

### Integration Tests

- Launch a real headless Chrome against a local test HTML server.
- Keep routine Bowser CLI browser tests non-headed by setting `BOWSER_INTERNAL_STEALTH_FEATURES=-launch-headed,-launch-native-window` in the test command helpers; this preserves the rest of the default stealth behavior while avoiding headed CI windows.
- Verify full pipeline: navigate → capture → verify YAML structure.
- Verify `bowser get` prints a resumable session ID and a later command can resume the same browser session.
- Verify multi-page flows: list pages, create a new page, switch pages, close pages, and preserve the selected page across detach/resume.
- Verify long lists/tables are truncated in normal capture output and can be fully retrieved with `bowser expand`.
- Verify compact YAML omits link/image deferred metadata in default output, labels images from `alt` plus filename, and only shows `describable: true` when AI is currently available.
- Verify `bowser meta` retrieves deferred metadata on demand while also reporting live visibility details.
- Verify `bowser describe` returns cached or freshly generated image descriptions from an image ID.
- Test interactions: click a link → verify navigation occurred → verify updated YAML.
- Test `expand <ID>` in interactive mode against a truncated node.
- Test `meta <ID>` in interactive mode for both interactive and non-interactive element IDs.
- Test that an invalid interactive command prints an error and returns to the prompt without exiting.
- Test first-`Ctrl-C` clears interactive input without exiting, and second `Ctrl-C` on an empty prompt exits cleanly.
- Test command and element-ID suggestions against a known captured page.
- Test repeated full-page interactive renders are separated by `---`.
- Test interactive exit/resume: exit the REPL, reattach with `--session`, and verify cookies/history/page state persist.
- Test dynamic content: page with JS that adds elements after 1s → `refresh` → verify new elements appear.
- Test stability detection: page with ongoing network requests → verify timeout behavior.
- Test stealth: verify stable best-effort patches are active, including `navigator.webdriver !== true` and the User-Agent override removing `Headless`.
- Test screenshots: verify PNG output is valid and non-empty.
- Mock AI providers in automated tests so the suite does not require external credentials or network access.
- Provision Chrome/Chromium in CI so the browser-backed integration suite runs consistently, and allow local override with `BOWSER_TEST_CHROME_PATH` during development.
- Keep routine Bowser doctests and browser-running tests headless by forcing `BOWSER_HEADLESS=true` and `BOWSER_INTERNAL_STEALTH_FEATURES=-launch-headed,-launch-native-window` at the check-plan, CI, and shared CLI test-helper boundaries.
- Keep the live Google captcha regression test ignored by default. Run it explicitly with `BOWSER_GOOGLE_SMOKE=1 cargo test -p bowser-cli --test interactive_repl google_search_flow_avoids_captcha -- --ignored --nocapture`; that test passes `--headed` explicitly.

### Test HTML Server

Integration tests spin up a local HTTP server (using `axum` or similar) serving static HTML pages that exercise all element types, dynamic content patterns, and interaction targets.
