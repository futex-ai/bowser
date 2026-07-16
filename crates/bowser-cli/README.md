# bowser-cli

CLI wrapper around the `bowser` library.

## Responsibilities

- CLI parsing and top-level dispatch
- Interactive REPL
- Structured output handling
- Session lifecycle commands

## What This Crate Does

CLI wrapper around the `bowser` library.

This crate is Bowser's direct human-facing composition layer. It keeps command
parsing, terminal rendering, and interactive control separate from the reusable
`bowser` library.

## Quick Start

```bash
bowser https://example.com
bowser -i https://example.com
bowser session list
```

## Commands

- `bowser <URL>` and `bowser get <URL>`: single-shot capture with detached-session output
- once `get` creates a session, later navigation, capture, or output errors still detach and print `Session: <ID>` so the running browser remains discoverable and resumable
- resumed `capture`, interaction, and `page` commands attempt detach after every browser operation; operation and render errors retain precedence, while terminal or file output happens only after detach succeeds
- `bowser get --format html <URL>`: print the full current rendered document as raw HTML text while still storing the structural capture in the detached session
- `bowser --all <URL>`: disable truncation for that capture; hidden and obscured semantic content is included by default under `content.obscured`
- `bowser --headed --persistent-profile <URL>`: use a visible Chrome window and Bowser's stable default profile path
- `bowser --no-ai -i https://www.google.com`: run the default stealth Google smoke flow with headed Chrome, full JS DOM capture, backend-node text entry, and Runtime events left enabled
- `bowser capture --session <ID> --format html|json|yaml`: recapture the selected or explicit `--page-id` page without navigating
- `bowser click|clear|submit --session <ID> <ELEMENT_ID>`: interact with a captured element and print an updated capture
- `bowser type --session <ID> <ELEMENT_ID> <TEXT>` and `bowser key --session <ID> <KEY>`: send real browser input and print an updated capture
- `bowser scroll --session <ID> --direction up|down` or `--element-id <ID>`: scroll the selected or explicit page and print an updated capture
- `bowser download <URL> <PATH>`: download a file through the active Chrome session
- `bowser download --session <ID> --link <ELEMENT_ID> --output <PATH>`: download the href from a captured link element in the current page
- `bowser -i [URL]`: interactive mode with resumable exit
- navigation commands accept bare hostnames such as `slack.com` and default them to `https://`; loopback hosts such as `localhost:3000` default to `http://`
- `bowser expand --session <ID> <ELEMENT_ID>`: print a full subtree from the last capture
- `bowser meta --session <ID> <ELEMENT_ID>`: fetch metadata for any ID-bearing element, including current focus, live visibility details, page-space bounds, and `describable: true` for images when AI is available
- `bowser describe --session <ID> <ELEMENT_ID>`: generate or print a cached AI description for an image by ID
- `bowser page list|select|new|close --session <ID>`: inspect and manage multiple pages inside a detached session using stable page IDs such as `pg_1`
- `bowser session list|info|close`: inspect and manage detached sessions
- `bowser pointer-log --output browser-log`: serve a loopback-only local pointer telemetry page with one randomized target at a time and append captured events to a local JSONL file, defaulting to the gitignored `browser-log` in manual mode, without storing session IDs or user-agent strings; each newly placed target also writes a row with its viewport rect
- `bowser pointer-log --demo`: launch headed Chrome, open the pointer telemetry page, and use Bowser's real pointer-click automation to move between targets until `Ctrl-C` or `--demo-clicks <N>`; demo mode only writes JSONL events when `--output` is provided
- interactive `pages`, `page <PAGE_ID>`, `new page [URL]`, and `close page [PAGE_ID]`: inspect, switch, create, and close session pages from the REPL
- interactive `describe <ID>`: print the explicit AI description for an image when available
- interactive `html`: print the full current rendered document as raw HTML text
- interactive `keypress <KEY...>`: press one or more space-separated keys together on the focused element, for example `keypress cmd enter`
- interactive prompts are colorized, include the current page URL, and show gray inline hints that Right Arrow accepts
- interactive `help` prints each command with usage and a one-line description
- invalid REPL commands stay in-session: Bowser prints the error and redraws the prompt
- in-flight REPL commands can be cancelled with `Ctrl-C`, and long-running commands time out back to the prompt instead of trapping the session; state-changing commands keep a small extra outer budget so the required post-action capture is not cut off at the page-load timeout boundary
- REPL ID arguments accept either bare numbers (`12`) or rendered element keys such as `input#12`
- resumed REPL startup prefers the stored selected-page preview immediately and defers any live DOM-ID rebuild until the first ID-based runtime command needs it
- primary YAML only includes `focused: true` when an ID-bearing element is currently focused
- image labels in compact YAML prefer the DOM `alt` text plus the image filename; images only surface `describable: true` when the current AI config can satisfy `describe`
- default capture includes rendered below-the-fold content; if the top-level preview is truncated, `expand` can target the synthetic `body#<ID>` wrapper that Bowser prints in the YAML
- `click` on visible controls now uses a cursor-like pointer path shaped by the pointer telemetry fixture before pressing the mouse button; if low-level pointer input fails or stalls, Bowser falls back to the synthetic DOM click path, and focusable controls still surface `focused: true`
- `type`, `keypress`, and `clear` now use real browser key input rather than synthetic DOM keyboard events
- `submit` prefers Enter on text-like controls or a real click on submit buttons before falling back to DOM form submission helpers
- `scroll down` / `scroll up` now use browser mouse-wheel input; `scroll to <ID>` remains a direct bring-into-view helper
- post-command re-captures use a bounded quiet-window wait, so pages with long-lived background requests do not hold the REPL for the full stability timeout
- REPL page switch and page close output can fall back to the stored selected-page preview when Chrome stalls during the live recapture; `refresh` forces a new capture

## Development

```sh
cargo test -p bowser-cli
cargo clippy -p bowser-cli --all-targets --all-features -- -D warnings
cargo xtask check

# manual smoke paths
# from a directory containing report.csv
python3 -m http.server 8765 &
bowser download http://127.0.0.1:8765/report.csv /tmp/report.csv
bowser --headed --timeout 10 https://example.com
BOWSER_GOOGLE_SMOKE=1 cargo test -p bowser-cli --test interactive_repl google_search_flow_avoids_captcha -- --ignored --nocapture
```

- Top-level CLI parsing and dispatch now live in [`src/args.rs`](./src/args.rs), [`src/cli.rs`](./src/cli.rs), and [`src/dispatch.rs`](./src/dispatch.rs), while [`src/main.rs`](./src/main.rs) stays as the thin binary entrypoint
- The local pointer telemetry command lives in [`src/commands/pointer_log/`](./src/commands/pointer_log), shows one randomized target element at a time, appends one JSON object per event to a configurable JSONL file that defaults to the repo-root gitignored `browser-log` in manual mode, omits session IDs and user-agent strings, and records a `target_spawn` row with target bounds each time a new target is placed
- Pointer-log demo mode starts the same loopback server, launches headed Chrome through the `bowser` library, drives the page with `PageEngine::click`, and renders an in-page cursor trail plus click pulse so CDP pointer movement is visible.
- Stealth is conservative: Bowser cleans obvious automation leaks and keeps native Chrome fingerprint values unless a complete browser identity profile is designed and tested as one system.
- Default stealth uses the Google-safe headed path and avoids native headless Chrome without requiring a separate mode flag; see the protocol ledger for the current Google captcha control status.
- Bowser does not expose proxy transport. Use a VPN outside Bowser when browser traffic must leave through a different network route or IP address.
- Browser-native downloads preserve Chrome session state such as redirects, cookies, authentication, and `Content-Disposition` handling.
- Headed Linux sessions start Xvfb only when neither `DISPLAY` nor `WAYLAND_DISPLAY` is available; `session close` and expiry cleanup validate the stored Xvfb display before terminating the helper PID and remove only Bowser-owned ephemeral profiles.
- Interactive runtime flow is split across [`src/commands/interactive/`](./src/commands/interactive), with command dispatch, page actions, capture fallback, and interrupt handling separated into focused modules
- REPL parsing, suggestions, and prompt helpers live under [`src/commands/repl/`](./src/commands/repl), with command parsing separated from editor/completion behavior
- Browser-independent CLI coverage lives in [`tests/cli_flows/`](./tests/cli_flows), with shared command helpers in [`tests/cli_flows/support.rs`](./tests/cli_flows/support.rs)
- PTY-backed REPL coverage lives in [`tests/interactive_repl/`](./tests/interactive_repl), with shared prompt/session helpers in [`tests/interactive_repl/support.rs`](./tests/interactive_repl/support.rs)
- Automated CLI browser tests force `BOWSER_HEADLESS=true` while keeping stealth enabled and disabling the headed launch features through `BOWSER_INTERNAL_STEALTH_FEATURES=-launch-headed,-launch-native-window`; the live Google smoke is the only headed CLI test and remains ignored unless `BOWSER_GOOGLE_SMOKE=1` is set.
- PTY-backed prompt assertions tolerate ANSI color sequences because Linux terminals render Bowser's colorized prompt in CI
- REPL page inventory is sourced from stored session metadata so listing/switching pages does not destabilize the active Chromium page handle
- REPL page switching and page close flows rely on bounded browser URL/title probes and stored preview fallback when Chrome still reports a stale target
- REPL page close requests are bounded and can fall back to Chrome target-level close so a stalled page close does not trap the prompt
- REPL `type` commands flow through the library typing engine, which now sends real per-character key input with very short randomized pauses
- REPL `keypress` commands flow through the library keyboard engine and support multi-key combos with special-key aliases

### Key Code

- `src/main.rs`
- `src/dispatch.rs`
- `src/commands/capture.rs`
- `src/commands/interactions.rs`
- `src/commands/interactive/`
- `src/commands/repl/`

### Related Docs

- [Workspace README](../../README.md)
- [Bowser protocol spec](../../docs/protocol/bowser/README.md)
- [Library crate](../bowser/README.md)
