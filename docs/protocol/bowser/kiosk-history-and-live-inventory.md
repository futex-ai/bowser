# Kiosk Launch, History, And Live Inventory

This page defines Bowser's integration boundary for caller-requested desktop
Chrome kiosk launch, one-shot history commands, and live session inventory.

## Capability Handshake

`bowser capabilities` reports these features independently:

| Feature | Current value | Meaning |
|---|---:|---|
| `features.kiosk` | `false` | Desktop Chrome does not yet meet the required locked-keyboard kiosk contract. Callers must not append `--kiosk` based on Bowser capability detection. |
| `features.history` | `true` | The one-shot `back`, `forward`, and `reload` commands and typed exhaustion result are available. |
| `features.live_inventory` | `true` | `session info` reads the current live page inventory without activating a page. |

Feature values describe the installed binary, are additive, and may change
independently in later releases.

## Caller-Requested Desktop Kiosk Launch

Bowser forwards caller Chrome arguments after its own baseline and stealth
arguments. Chrome's normal last-occurrence behavior therefore gives caller
values precedence. Bowser rejects only the remote-debugging and user-data-dir
switches it owns for session identity.

When caller arguments contain `--kiosk`, Bowser does not add any of these
window geometry or mode defaults:

- `--start-maximized`
- `--window-size`
- `--window-position`
- `--start-fullscreen`
- `--app`

On a fixed 1024×768 X display with a window manager, the browser-backed kiosk
test verifies that Chrome fills the display and reports matching `screen`,
`inner`, and `outer` dimensions. It also verifies capture, full-page PNG
screenshot, `page new`, `page select`, and `page close` behavior. Bowser's
stealth script does not synthesize window geometry and does not assume a
decorated window, so the native fullscreen metrics remain internally
consistent.

Chrome-native dialogs, including authentication and permission prompts, remain
browser UI layered above the page. Bowser neither suppresses nor replaces
them. A content-opened tab is discovered during the next live inventory read
and receives a stable Bowser page ID. Bowser reads focus without activating
either page. A foreground content-opened tab becomes `selected_page_id` when
it is the one visible, focused page; `page select` also explicitly changes
that target and raises the selected page. Exactly one live page record matches
`selected_page_id`.

### Desktop Chrome accelerator matrix

The matrix below records native Linux desktop Chrome behavior under `--kiosk`
with raw X keyboard input:

| Accelerator | Native result |
|---|---|
| F5 | Reloads the page. |
| Ctrl+R | Reloads the page. |
| Alt+Left | Traverses backward when an entry exists. |
| Alt+Right | Traverses forward when an entry exists. |
| Ctrl+F | Opens Chrome's find overlay. |
| Ctrl+L | Does not expose or focus the hidden omnibox. |
| Ctrl+T | Opens a new tab. |
| Ctrl+W | Closes the active tab or window. |
| Ctrl+N | Opens another Chrome window. |

Chromium's
[desktop switch definition](https://chromium.googlesource.com/chromium/src/+/HEAD/chrome/common/chrome_switches.cc)
explicitly describes `--kiosk` as distinct from ChromeOS kiosk mode. Because
Ctrl+T, Ctrl+W, and Ctrl+N remain active, Bowser must keep
`features.kiosk` false even though launch passthrough and fullscreen page
workflows work. A platform that filters raw takeover input may use the launch
behavior deliberately, but it must not infer the locked accelerator contract
from this release's capability response.

## One-Shot History Commands

The commands require `--session <ID>` and act on the logical selected page by
default:

```text
bowser back [--page-id <PAGE_ID>] [--output <PATH>] [--format yaml|json|html]
bowser forward [--page-id <PAGE_ID>] [--output <PATH>] [--format yaml|json|html]
bowser reload [--page-id <PAGE_ID>] [--output <PATH>] [--format yaml|json|html]
```

Each command marks cached document IDs stale, performs the browser action,
waits for the resulting document to settle, captures the page, and detaches.
Its plain output and JSON-envelope result use the same capture or output-file
shape as other capture-bearing commands.

Before `back` or `forward`, Bowser reads Chrome's navigation-history index. If
the requested direction has no entry, the command returns the stable envelope
code `history_exhausted` with `detail.direction` set to `back` or `forward`.
This check returns immediately instead of waiting for a page timeout. Reload
has no exhaustion state and currently uses Chrome's normal cache behavior.

## Live `session info`

With `features.live_inventory: true`, `session info <SESSION_ID>` resumes the
debug connection and reads Chrome's current page targets, URLs, and titles at
invocation time. It includes human navigation, client-side URL changes,
redirects, and content-opened tabs even when no Bowser action caused them.
Closed targets are omitted. Existing `pages[].last_url` and
`pages[].last_title` field names remain unchanged, but their values in this
response are live observations.

The read does not call Chrome target activation, change focus, navigate, or
send input. URL, title, and document-focus reads for all pages run concurrently,
each with a one-second bound; URL/title failures use stored-value fallback.
When exactly one page reports that it is visible and focused, the returned
selection follows that native active tab. If focus is unavailable or ambiguous,
Bowser retains the stored live selection. Bowser may atomically
persist the observed inventory and stable page-ID mappings; this metadata
maintenance does not mutate browser or page state. Selection reconciliation is
metadata only and gives the response exactly one selected page.

The JSON-envelope payload remains:

```json
{
  "result": {
    "session": {
      "selected_page_id": "pg_1",
      "pages": [
        { "id": "pg_1", "last_url": "https://example.test/", "last_title": "Example" }
      ]
    }
  }
}
```
