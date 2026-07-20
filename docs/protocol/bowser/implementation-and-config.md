## DOM Capture Implementation

The DOM-to-YAML transformation is performed by injecting a JavaScript function into the page via CDP's `Runtime.evaluate`. This approach is chosen over Rust-side DOM traversal because:

1. The JS runs in the page context and has access to computed styles (`getComputedStyle`), bounding rects (`getBoundingClientRect`), and the live DOM — all needed for visibility checks and flattening.
2. It avoids the overhead of serializing the entire DOM to Rust and back.
3. It can recursively traverse live DOM children and text nodes, recurse into accessible same-origin iframes, and hand off cross-origin iframes to stitched follow-up capture paths when Chrome exposes them as same-target frames or separate iframe targets.

The injected script:

1. Walks element and text nodes in document order from the page root.
2. Checks rendered visibility (computed styles + bounding rect) while preserving hidden inputs explicitly and without treating `aria-hidden` as a standalone visual-hide signal.
3. Maps nodes to semantic element types, applies flattening/text merging, or
   skips wrapper-only structure.
4. Splits root semantic output into `content.visible` and
   `content.obscured`; below-the-fold content remains visible, while
   style-hidden and modal-obscured semantic content is captured as obscured.
5. Assigns provisional IDs to interactive elements, semantic container elements, and images in each live document context, and stores that provisional ID on each ID-bearing DOM element for later owner lookup.
6. Returns a JSON object representing the per-frame capture tree plus deferred metadata records.

The Rust side deserializes these per-frame payloads, captures any target-backed
cross-origin iframe documents through their own DevTools websocket endpoints,
uses `Target.parentFrameId` plus `DOM.getFrameOwner` to stitch those subtrees
back under the owning `iframe` elements, rewrites the final IDs into one
session-scoped sequence, and then serializes the merged `PageCapture` to YAML
or JSON with the same visible/obscured root content shape.

### Element-to-ID Mapping

The injected JS stores a mapping of provisional IDs to live DOM element
references in a `Map` on `window.__bowserElements`. After cross-frame stitching,
Bowser rewrites the same-document live maps to the final emitted IDs so normal
same-document and same-origin runtime lookups continue to work without
re-traversing. For target-backed iframe captures, Bowser also keeps a live
final-ID to iframe-target context map so `click` can resolve the iframe-local
element, translate its coordinates through the owning iframe, and dispatch a
trusted pointer click through the main page. The same target context lets
`meta`, `type`, `clear`, `select`, `submit`, `scroll` to element, and element
screenshots evaluate against the iframe target. `describe` can also render
target-backed iframe `<img>` elements to element-local PNGs inside their owning
iframe document instead of clipping the parent page.

The map is cleared and rebuilt on every `capture()` call.

Because IDs are ephemeral, resumed sessions must rebuild the live mapping before any ID-based runtime command. Bowser may satisfy that requirement with a normal fresh capture or with a lightweight structural recapture that only reinstalls the live DOM mapping.

Cross-origin iframe descendants currently participate in capture output and
expansion. Target-backed iframe descendants can also receive frame-aware
`click`, `meta`, `type`, `clear`, `select`, `submit`, `scroll` to element,
element screenshot, and `describe` commands while the live iframe target is
available. Non-ID-routed commands such as free-form JS evaluation, selector
waits, and raw keypresses remain attached to the active page target.

## Screenshots

`PageEngine::screenshot` captures the full document with a CDP clip using the
page's CSS content size and `captureBeyondViewport`. It must not emulate a
larger viewport before capture, because that path can drop compositor surfaces
for target-backed cross-origin iframes such as reCAPTCHA frames.

Element screenshots normally use a page-space CDP clip. For target-backed
iframe descendants, Bowser translates iframe-local bounds through the owning
iframe and clips the parent page. Target-backed iframe images first try to
render the actual `<img>` element to an element-local PNG so `describe` does not
depend on parent-page clipping, then fall back to the translated visual clip.

## Browser Target Attachment

Fresh Bowser sessions create and select a Bowser-owned blank page target before returning the first `PageEngine`. They do not reuse Chrome's startup `about:blank` target for the active page, because Chromium target discovery can expose stale CDP target sessions while Chrome is still settling after launch. Once the owned target exists, Bowser closes any unmanaged startup blank it observed before creation so later page inventory cannot adopt it as a second session page.

Before a page is returned to callers, activation must validate the page target by sending a real CDP runtime evaluation. A cached URL or title read is not enough to prove that the target session can accept later navigation, capture, and interaction commands.
If Chrome reports that the target session is missing during that validation, Bowser must reacquire a fresh page handle for the same target and retry before returning an error.

## Session Persistence and Process Ownership

`FileSessionStore` confines metadata access to validated `bsr_` identifiers and requires each deserialized document ID to match its filename. Both `load` and `list` deserialize through the legacy-aware raw metadata shape before exposing current `SessionMetadata`.

Freshly spawned Chrome and Xvfb processes remain behind an armed ownership guard while Bowser connects to Chrome and persists session metadata. A generated ephemeral profile remains behind a parallel directory guard. An error at either boundary terminates every newly owned process and removes the unfinished profile; both guards are disarmed only after metadata persistence succeeds.

Fresh Chrome processes receive `--remote-debugging-port=0`. Bowser snapshots any old profile-local `DevToolsActivePort` content before spawning, waits for a different complete handshake, and verifies Chrome's chosen loopback endpoint before connecting. This keeps port allocation owned by Chrome for the entire bind operation rather than releasing a temporary reservation before launch.

Persisted PIDs are not proof of ownership because operating systems reuse them. Resume, close, and expiry inspect the live process command line. Dynamic-port Chrome must retain `--remote-debugging-port=0` and the exact `--user-data-dir`, while the profile handshake's selected port must match the persisted HTTP endpoint before Bowser connects or sends a termination signal. Legacy fixed-port sessions must retain the exact persisted port and profile arguments. Resume fails with `SessionProcessIdentityMismatch` when inspection fails or the identity differs. Cleanup skips a mismatched PID. A matching Chrome process receives a graceful termination request and a bounded exit wait; if it retains the snapshotted session identity, cleanup force-terminates it and waits again before profile deletion. Xvfb must retain both the Xvfb program identity and stored display argument. Metadata remains available for retry if the validated process still does not exit.

Persisted profile ownership is also explicit rather than inferred from an arbitrary path. `SessionMetadata::owns_user_data_dir` defaults to `false` for legacy documents and is set only for fresh, non-persistent, non-explicit profiles. Cleanup requires the configured session root and rejects an owned path unless it is one UUID-named direct child of `<session-root>/profiles`; metadata remains available for a later retry when profile removal fails.

## Configuration

### Config File

Located at `~/.config/bowser/config.yaml` (overridable via `--config`). The platform-default file is optional when no path is selected explicitly. An explicit `--config` path must exist; Bowser returns a path-bearing `ConfigFileNotFound` error instead of silently continuing with environment variables and defaults.

```yaml
sessions:
  dir: null                    # platform state dir if null
  idle_ttl: 1800               # detached session expiry in seconds

output:
  truncate: true
  include_hidden: false       # legacy/no-op; obscured content is included by default
  max_children: 50
  max_list_items: 20
  max_table_rows: 20

chrome:
  path: null                   # auto-detect if null
  args: []                     # additional Chrome args
  headless: true               # false = headed/visible window
  persistent_profile: false    # true = reuse the stable default profile path
  user_data_dir: null          # highest-precedence explicit profile path

viewport:
  width: 1920
  height: 1080

timeout: 30                    # per-step browser wait timeout in seconds
stealth: true                  # enable stealth patches
disable_runtime_events: false  # advanced diagnostic: also send CDP Runtime.disable

ai:
  enabled: true
  provider: "anthropic"        # anthropic | openai | ollama
  model: "claude-sonnet-4-20250514"
  api_key_env: "ANTHROPIC_API_KEY"
  endpoint: null               # for ollama only
```

### Environment Variables

The following environment variables are recognized:

- `BOWSER_CHROME_PATH`
- `BOWSER_CHROME_ARGS`
- `BOWSER_SESSION`
- `BOWSER_SESSION_DIR`
- `BOWSER_SESSION_IDLE_TTL`
- `BOWSER_OUTPUT_TRUNCATE`
- `BOWSER_OUTPUT_INCLUDE_HIDDEN`
- `BOWSER_OUTPUT_MAX_CHILDREN`
- `BOWSER_OUTPUT_MAX_LIST_ITEMS`
- `BOWSER_OUTPUT_MAX_TABLE_ROWS`
- `BOWSER_HEADLESS`
- `BOWSER_PERSISTENT_PROFILE`
- `BOWSER_USER_DATA_DIR`
- `BOWSER_VIEWPORT`
- `BOWSER_TIMEOUT`
- `BOWSER_STEALTH`
- `BOWSER_DISABLE_RUNTIME_EVENTS`
- `BOWSER_AI_ENABLED`
- `BOWSER_AI_PROVIDER`
- `BOWSER_AI_MODEL`
- `BOWSER_AI_API_KEY_ENV`
- `BOWSER_AI_ENDPOINT`

Boolean environment variables accept `true` / `false`. `BOWSER_VIEWPORT` uses the same `WIDTHxHEIGHT` format as the CLI flag. Viewport width and height must both be greater than zero. Bowser validates the final merged configuration, including direct library overrides, before launching Chrome or Xvfb.

Profile-path precedence is:

1. explicit `--user-data-dir` / `BOWSER_USER_DATA_DIR` / `chrome.user_data_dir`
2. `--persistent-profile` / `BOWSER_PERSISTENT_PROFILE` / `chrome.persistent_profile` using Bowser's stable default profile path
3. a fresh per-session profile directory under the Bowser session state root

Deployments that persist detached sessions must include both the configured
session metadata directory and Chrome profile root in the same durability
boundary. Backups may skip transient Chrome lock files, but shutdown snapshots
should flush metadata and profile state before replacing the previous usable
snapshot. Bowser does not own a deployment's backup or environment-routing
policy.

The local engine does not expose proxy transport configuration. Hosted egress
is an external sandbox-provider concern and is configured before this engine
launches Chrome; see
[Managed Agent And Browser Runtime](./managed-agent-and-browser.md).

### Headed Linux Containers

When `chrome.headless` is `false`, Bowser should prefer the user's existing
display server:

1. if `DISPLAY` is set, launch Chrome with that display;
2. if `WAYLAND_DISPLAY` is set, do not start Xvfb;
3. on Linux only, if neither display variable is set, start Xvfb on a free
   display number and launch Chrome with that `DISPLAY` value.

Any Xvfb process started by Bowser is part of the browser session. Detached
sessions keep Xvfb alive with Chrome, and closing or expiring the Bowser session
must terminate both the Chrome process and the Xvfb process. Session metadata
stores the Xvfb PID and display string; both the Xvfb program identity and stored
display must still match the live command line before Bowser terminates the
helper process.

When `chrome.headless` remains true and stealth is enabled, Bowser intentionally
omits `--headless=new` and launches Chrome as a headed browser without forcing
`--window-size` or an offscreen `--window-position` on an existing user display.
On Linux this follows the same display planning rules as headed mode, including
Xvfb when no display server is available. If Bowser owns that synthetic Xvfb
display, it must pass the configured viewport as Chrome's `--window-size` so the
first page has a non-zero viewport even without a window manager. It still
avoids `--window-position`. `disable_runtime_events` is not required for the
default stealth path; it is an advanced diagnostic that additionally sends CDP
`Runtime.disable` after page preparation.

### Browser-Native Downloads

`PageEngine::download` performs browser-native downloads through the active page
session. It configures Chrome to allow downloads to a Bowser-controlled
temporary directory, navigates or clicks through the browser context, waits for
the resulting file to complete, and moves the completed file to the caller's
destination path.

The destination path must name a file, not a directory. Parent directories must
already exist. Download failures return structured Bowser errors instead of
silently falling back to a direct HTTP client, because direct fetches do not
preserve browser cookies, redirects, challenges, or other page state.
Failures to configure Chrome or start the download navigation return
`Download` immediately. `DownloadTimeout` is reserved for navigation that
started successfully but did not produce a completed file before the deadline.

### Precedence

CLI flags > environment variables > config file > defaults.

For Anthropic and OpenAI providers, `api_key_env` names the environment variable that contains the actual provider API key. If that environment variable is missing or empty, compact YAML omits `describable: true`, regular captures stay structural, and explicit `describe` requests fail unless a cached description already exists. For Ollama, enabled local configuration is enough for `describable: true`. All provider clients apply `BrowserConfig.timeout` to the complete request, reject non-success HTTP responses before parsing the response body, retain the HTTP status in `AiSummarization`, and return `AiSummarizationTimeout` when the deadline expires.
