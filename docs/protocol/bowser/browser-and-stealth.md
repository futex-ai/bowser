## Chrome Interaction Crate

### Recommendation: `chromiumoxide`

Bowser uses [`chromiumoxide`](https://github.com/mattsse/chromiumoxide) for browser automation via the Chrome DevTools Protocol (CDP).

### Evaluated Options

| Criteria | `chromiumoxide` | `headless_chrome` | `fantoccini` |
|---|---|---|---|
| **Protocol** | CDP (full) | CDP (partial) | WebDriver |
| **Async** | Yes (tokio) | No | Yes (tokio) |
| **CDP coverage** | Complete — auto-generated from Chrome PDL spec | Partial — missing frame handling, WebSocket inspection | None — WebDriver only |
| **Screenshots** | Full CDP control | Element + full page | Basic (WebDriver) |
| **Network interception** | Yes (Fetch + Network domains) | Yes | No |
| **Stealth** | No built-in, but supports JS injection via `Page.addScriptToEvaluateOnNewDocument` | No built-in | Worst — sets `navigator.webdriver = true` by spec |
| **Maintenance** | Active (mattsse, Foundry/Alloy author), 36 open issues | 142 open issues, sync-only architecture | Stable but WebDriver-limited |
| **Stars / Downloads** | ~1,200 stars / 917K recent downloads | ~2,900 stars / 822K recent | ~2,000 stars / 2.6M total |

### Why `chromiumoxide`

- **Async-native**: integrates cleanly with tokio, which Bowser uses throughout.
- **Full CDP**: every CDP domain is available as typed Rust structs, generated from Chrome's protocol definition. This gives Bowser access to DOM capture, JS execution, screenshots, network interception, and script injection — all required.
- **Stealth via injection**: although no built-in stealth, Bowser injects stealth patches at page creation time via `Page.addScriptToEvaluateOnNewDocument`. This is the same mechanism puppeteer-extra-plugin-stealth uses and is well-proven.
- **Active maintenance**: lowest issue count, highest recent download velocity, maintained by a prolific Rust ecosystem contributor.

### Why not the others

- **`headless_chrome`**: no async support is a fundamental limitation. Every browser operation blocks an OS thread. The 142 open issues suggest maintenance debt. Not suitable as a foundation for new projects.
- **`fantoccini`**: WebDriver protocol cannot intercept network requests, cannot inject scripts before page load, and actively advertises automation via `navigator.webdriver`. Architecturally wrong for a tool that needs stealth.

### Future consideration: `chaser-oxide`

[`chaser-oxide`](https://github.com/nicholasgasior/chaser-oxide) is a `chromiumoxide` fork with protocol-level stealth (fingerprint profiles, Bezier mouse movement, realistic typing). As of early 2026 it is at 0.1.x with 238 stars. If it matures, Bowser could migrate with minimal API changes since it shares the `chromiumoxide` API surface. For now, the risk of depending on a 3-month-old fork outweighs the benefit.

## Chrome Launch and Debug Endpoint

Fresh sessions launch Chrome with `--remote-debugging-port=0` so Chrome owns the ephemeral-port allocation without a bind-and-release race. Bowser snapshots any existing profile-local `DevToolsActivePort` file before launch, waits for a different complete handshake containing a non-zero port and browser WebSocket path, and then verifies the selected loopback endpoint through `/json/version` within the configured launch timeout. A stale, incomplete, oversized, symlinked, or non-file handshake must not be accepted.

Detached metadata persists the assigned HTTP and WebSocket endpoints. A dynamic-port session may be resumed or terminated only when the stored PID still has the dynamic-port and exact profile arguments and the profile handshake's current port matches the persisted HTTP endpoint. Legacy metadata whose Chrome process has the exact persisted fixed-port argument remains resumable.

Caller Chrome arguments are forwarded after Bowser defaults. When the caller
includes `--kiosk`, Bowser omits its own window geometry and window-mode flags
so Chrome and the display own fullscreen sizing. The fixed-display behavior,
native accelerator matrix, `features.kiosk_launch` handshake, and reason the
separate `features.kiosk` flag remains false are defined in
[Kiosk Launch, History, And Live Inventory](./kiosk-history-and-live-inventory.md).

## Stealth

Bowser applies best-effort stealth measures to reduce common headless-browser fingerprints. The following patches are applied via `Page.addScriptToEvaluateOnNewDocument` before any page navigation:

1. **Suppress `navigator.webdriver`**: override the property so pages observe `false`, matching gstack's narrow webdriver-mask behavior.
2. **Launch with the narrow gstack-style control flags**: when stealth is enabled, launch Chrome with `--hide-crash-restore-bubble` and `--disable-blink-features=AutomationControlled`.
3. **Clean User-Agent**: strip `HeadlessChrome` from the User-Agent string via CDP and replace the `Chrome/` version with the actual browser product version from `Browser.getVersion`.
4. **Clean automation globals**: remove automation-only globals with names that start with `cdc_` or contain `__webdriver`.
5. **Patch notification permission reads narrowly**: `navigator.permissions.query({ name: 'notifications' })` returns `prompt`, matching the narrow Permissions API cleanup gstack documents without synthesizing broader permission state.

These patches are bundled as a JS file within the Bowser binary and injected as a single script. They are applied on every new page and every new frame.

Stealth patches must not globally replace browser primitives in ways that break normal site behavior. In particular, Bowser must not override `HTMLIFrameElement.prototype.contentWindow` with the top-level `window`, because iframe-heavy flows such as reCAPTCHA depend on real per-frame window objects.

Bowser must not invent fixed values for broad fingerprint surfaces such as
`window.chrome.runtime`, WebGL vendor/renderer, `navigator.plugins`,
`navigator.mimeTypes`, languages, platform, canvas, audio, timezone, or locale
as independent patches. These surfaces form one browser identity and are
cross-checked by modern sites. If Bowser ever synthesizes non-native values for
them, it must design the whole identity as one tested system.

### Browser Identity

Bowser treats these fingerprint surfaces as one coupled browser identity:

- User-Agent and User-Agent client hints
- `navigator.platform`, languages, locale, and timezone
- viewport, screen metrics, and device scale factor
- WebGL vendor and renderer
- plugins and MIME types
- `window.chrome` and extension runtime shape
- Permissions API behavior
- automation-only globals

The default identity is native Chrome plus narrow automation-leak cleanup.
Native Chrome values are preferred over synthetic values. Any explicit
non-native profile must generate all coupled fields from one typed profile and
must update the browser identity diagnostic tests.

Kiosk launch does not add a synthetic viewport identity or alter these patches.
Native fullscreen `screen`, `inner`, and `outer` dimensions remain the browser
identity source; no patch assumes that Chrome has a decorated window.

With stealth enabled, Bowser uses the Google-safe headed feature set by
default: headed Chrome, native window sizing on a real display, the
gstack-aligned launch/init identity, accessibility-tree capture, backend-node
focus for text entry, and Enter submit for search forms. If Bowser creates an
Xvfb display because no real Linux display exists, it applies the configured
viewport as Chrome's window size to keep the synthetic display usable. Bowser
leaves Runtime-domain events enabled by default; `Runtime.disable` is a
diagnostic/config-only behavior, not a public mode users need to select.

The Google smoke ledger distinguishes Bowser-native behavior from external
controls. Stock/manual Chrome, gstack headed mode, direct Playwright/CDP headed
controls, and Bowser's default stealth flow pass on the same machine and
network. Native headless Chrome still reaches Google's `/sorry/` page for this
flow.

Proxy transport is not a Bowser feature. Users who need network-level routing or
IP changes should use a VPN outside Bowser.

Stealth support is an implementation goal, not a guarantee of indistinguishability. Acceptance criteria are limited to the concrete behaviours specified in this document and the associated tests.
