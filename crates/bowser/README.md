# bowser

Core library for launching Chrome, capturing rendered pages into Bowser’s structured model, and interacting with resumable browser sessions.

## Responsibilities

- Browser lifecycle and detached sessions
- Multi-page session selection and creation
- DOM capture and structured page model
- Compact YAML and JSON serialization
- Deferred metadata, runtime element introspection, and truncated-node expansion
- Conservative stealth hooks, optional headed mode/profile reuse, browser-native downloads, and optional AI-backed image descriptions

## What This Crate Does

Core library for launching Chrome, capturing rendered pages into Bowser’s structured model, and interacting with resumable browser sessions.

Applications should depend on this crate when they need programmatic browser
sessions, page capture, or interaction. The neighboring `bowser-cli` crate is
the human-facing composition layer and keeps terminal concerns out of this
library.

## Quick Start

```rust
use bowser::{Browser, BrowserConfig, BrowserEngine, PageEngine, to_yaml};

# async fn example() -> bowser::Result<()> {
let browser = Browser::launch(BrowserConfig::default()).await?;
let page = browser.current_page().await?;
page.navigate("https://example.com").await?;
let capture = page.capture().await?;
println!("{}", to_yaml(&capture)?);
# Ok(())
# }
```

## Public Surface

- `Browser` / `BrowserEngine`: launch, resume, list/select/create/close session pages, detach, and close browser sessions
- `PageEngine`: navigation, capture, full rendered HTML, interaction, screenshots, browser-native downloads, metadata, on-demand image description, and expansion
- `BrowserConfig`: merged runtime configuration used by both CLI and tests
- `SessionStore` / `SessionMetadata`: detached-session persistence boundary and stored session shape for callers that want DB-backed metadata
- `to_yaml`, `to_json`, `metadata_to_yaml`, `image_description_to_yaml`: display and machine-readable serialization helpers

## Development

```sh
cargo test -p bowser
cargo clippy -p bowser --all-targets --all-features -- -D warnings
cargo xtask check
```

- Browser-backed integration coverage lives in the dedicated `browser_flows` test target at [`tests/browser_flows/mod.rs`](./tests/browser_flows/mod.rs), alongside focused suites such as [`tests/document_capture.rs`](./tests/document_capture.rs), [`tests/input_flows.rs`](./tests/input_flows.rs), and [`tests/pointer_click.rs`](./tests/pointer_click.rs)
- `FileSessionStore` confines metadata filenames to opaque `bsr_` identifiers, validates document IDs against filenames, atomically replaces saved metadata, skips invalid individual documents during enumeration, and migrates legacy metadata consistently on both load and list
- Fresh launches let Chrome choose its loopback debugging port, require a new profile-local `DevToolsActivePort` handshake before connecting, and terminate their known-owned Chrome and Xvfb processes plus unfinished ephemeral profiles on failure; resume, close, and expiry validate the persisted endpoint and profile identity, and cleanup removes only UUID-shaped profiles marked as Bowser-owned
- Browser-backed integration tests share a 60-second launch and page timeout and disable GPU compositing so cold Chrome startup on Linux CI and headless iframe screenshots stay stable; timeout-specific tests override lower values locally when they are asserting deadline behavior
- Test fixtures and local HTTP routes live under [`tests/support/`](./tests/support/mod.rs), with bulky HTML fixtures stored as standalone files in [`tests/fixtures/`](./tests/fixtures/)
- AI provider tests are mocked and do not require external network access or real credentials
- Capture visibility is render-first: `aria-hidden` alone does not suppress visibly rendered content, and script/style markup is excluded from text fallback
- Capture is full-document by default, so rendered below-the-fold content remains eligible even when it is outside the current viewport
- Capture splits root output into `content.visible` and `content.obscured`: blocking modal overlays keep the dialog visible while the background remains available as obscured content
- When the top-level preview itself is truncated, the library exposes that omission explicitly through a synthetic expandable `body#<ID>` section instead of silently dropping later root entries
- Stealth patches stay conservative around iframe primitives; the library does not override `iframe.contentWindow`, which avoids breaking iframe-based flows such as reCAPTCHA
- Same-target and target-backed cross-origin iframe content is stitched into compact captures and shares the same document-order ID sequence as the outer page; top-level screenshots preserve target-backed iframe surfaces, and target-backed iframe descendants support frame-aware metadata, clicks, text entry, clearing, select, submit, scroll-to-element, and element screenshots
- ARIA checkable controls such as `role="checkbox"`, `role="radio"`, and `role="switch"` are captured as actionable inputs, including labels resolved from `aria-label` or `aria-labelledby`
- Compact YAML image labels prefer `alt` plus the source filename; when AI is currently available, image nodes add `describable: true` so callers can opt into `PageEngine::describe`
- Regular captures and post-action REPL re-captures stay structural; Bowser only calls the AI provider during explicit `PageEngine::describe` requests
- `PageEngine::describe` is best-effort: missing credentials, screenshot failures, provider failures, or provider timeouts do not affect structural page capture; HTTP failures retain their status code, uncached provider calls use `BrowserConfig.timeout`, and a later describe call may still return a cached explicit description on demand
- Hidden or obscured semantic content is included by default under `content.obscured`; `BrowserConfig.output.include_hidden` is retained as a legacy no-op, and the CLI `--all` shorthand disables truncation
- `PageEngine::type_text`, `PageEngine::press_keys`, and `PageEngine::clear` now use real CDP keyboard input with fast randomized pauses instead of synthetic DOM keyboard events
- `PageEngine::submit` prefers Enter on text-like controls or a real click on submit buttons before falling back to DOM form submission helpers
- `PageEngine::scroll` sends browser mouse-wheel input for `Down` and `Up`, while `ScrollTarget::ToElement` remains a direct bring-into-view helper and routes through target-backed iframe documents when needed
- `PageEngine::navigate` uses one bounded navigation deadline, marks cached captures and live element IDs stale before requesting document-changing navigation, uses browser navigation first on fresh pages so redirects can be correlated by main-frame loader, uses top-level location assignment first on resumed pages to avoid stale target-session navigation hangs, and verifies the destination document is ready before returning; back, forward, and reload use the same conservative stale-state boundary before they request history-script or reload work
- `PageEngine::wait_for_stable` starts with a network-aware quiet check but relaxes the network-idle requirement after a short grace period, so captures do not sit on pages with long-lived background requests
- `PageEngine::click` prefers a pointer-driven browser click at a sampled 10px-inset target point for visible targets, shapes movement from the pointer telemetry fixture with randomized idle drift, coarse travel, approach, and micro-correction phases, carries the cursor across later visible clicks in the same live session, keeps text-entry targets focused for later keypresses, and falls back to `el.click()` when a target is not visibly clickable or the pointer path fails or stalls
- `PageEngine::metadata` now works for any ID-bearing element and attaches current focus, live visibility/clickability state, page-space bounds, and current image-description capability from the current page
- `PageEngine::screenshot` captures the full document with a CDP full-content clip rather than resizing the viewport, so out-of-process iframe surfaces remain visible in the main screenshot
- `PageEngine::describe` accepts an image ID, screenshots that element, and returns a structured description payload with `alt`, `src`, `filename`, and the generated description text
- `PageEngine::rendered_html` returns the full current rendered document as raw HTML text, which lets callers preserve script-driven DOM changes without routing through JSON-escaped `evaluate_js`
- After detach/resume, the first ID-based metadata or interaction lookup may rebuild the live element-ID map lazily before probing runtime state, using a lightweight structural recapture that skips the normal stability wait because its only purpose is to rebuild the live DOM-to-ID map
- Resumed `current_page` and `select_page` attachment validates the selected page target and reacquires a fresh CDP page handle when Chrome returns a stale target session during reattach
- `to_yaml` only emits `focused: true` for ID-bearing elements that are currently focused
- `BrowserConfig` now supports explicit `headless` control and `persistent_profile` reuse, with `user_data_dir` remaining the highest-precedence profile override; every launch validates that viewport width and height are both positive after all config sources are merged
- `Browser::launch_with_store` lets callers supply a custom `Arc<dyn SessionStore>` so session metadata can live outside the filesystem; `config.session.dir` still provides the default profile-root path for browser data when `user_data_dir` is not set
- `cleanup_expired_sessions` accepts that trusted session root explicitly, while `SessionMetadata::owns_user_data_dir` keeps explicit and persistent profiles outside Bowser's deletion boundary
- Fresh sessions create and select a Bowser-owned blank page target instead of reusing Chrome's startup target, close the unmanaged startup blank once the owned target exists, and validate the CDP target session before returning a live page
- Detached session metadata now stores per-page records with stable Bowser page IDs such as `pg_1` plus a typed page record (`tab` today); legacy single-page metadata is migrated forward on load
- The selected page ID is persisted in session metadata so resume, `page select`, `page new`, and `page close` all reattach to the expected page
- Live page inventory bounds URL/title probes and falls back to stored metadata, so stale Chromium targets after page close do not stall page selection
- Config loading now lives under `src/config/`, session persistence under `src/session/`, runtime page orchestration under `src/page/`, page model types under `src/model/`, and YAML/JSON rendering under `src/yaml/`, so those support areas can evolve without growing new monolith files
- The platform-default config file remains optional, while a path explicitly passed to `load_config` must exist and returns `ConfigFileNotFound` when it does not
- Bowser's stealth behavior is intentionally conservative: native Chrome values are preferred, obvious automation leaks are cleaned, and any future non-native fingerprint profile must be designed and tested as one complete browser identity
- Bowser's default stealth launch flags match the gstack-style control baseline: `--hide-crash-restore-bubble` plus `--disable-blink-features=AutomationControlled`, without synthetic locale or device-scale launch flags
- Bowser's default stealth path is the Google-safe headed path: full JS DOM capture, backend-node focus, Enter submit for search forms, and Runtime-domain events left enabled; see the protocol ledger for current live smoke evidence
- `BOWSER_INTERNAL_STEALTH_FEATURES` is a developer-only experiment override used by the Google smoke matrix to toggle one stealth behavior at a time; it is not part of the CLI contract
- Bowser does not expose proxy transport; use a VPN outside Bowser when browser traffic must leave through a different network route or IP address
- Browser-native downloads use the active Chrome session so redirects, cookies, and `Content-Disposition` behavior are handled by the browser rather than by a separate HTTP client; Chrome navigation failures return immediately as `DownloadStart`, while `DownloadTimeout` is reserved for a started download that never completes
- DOM capture replaces every non-empty password value with a fixed `[redacted]` marker before page models, metadata, YAML, or JSON are produced
- Headed Linux sessions start Xvfb only when neither `DISPLAY` nor `WAYLAND_DISPLAY` is available, apply the configured viewport as Chrome's window size on that synthetic display, and validate the stored Xvfb display before terminating the helper PID during cleanup

### Key Code

- `src/lib.rs`
- `src/ai/mod.rs`
- `src/browser/mod.rs`
- `src/config/mod.rs`
- `src/model/mod.rs`
- `src/page/mod.rs`
- `src/capture/mod.rs`
- `src/mouse/mod.rs`
- `src/session/mod.rs`
- `src/yaml/mod.rs`

### Related Docs

- [Workspace README](../../README.md)
- [Bowser protocol spec](../../docs/protocol/bowser/README.md)
- [CLI crate](../bowser-cli/README.md)
