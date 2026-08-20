# Kiosk Launch, History, And Live Inventory

This page defines Bowser's integration boundary for caller-requested desktop
Chrome kiosk launch, one-shot history commands, and live session inventory.

## Capability Handshake

`bowser capabilities` reports these features independently:

| Feature | Current value | Meaning |
|---|---:|---|
| `features.kiosk_launch` | `true` | Caller Chrome-argument precedence, kiosk conflict suppression, fixed-display fullscreen page workflows, and content-tab discovery through live inventory are verified. A platform that filters takeover input may append `--kiosk` when this exact flag is true. |
| `features.kiosk` | `false` | Desktop Chrome does not yet meet the separate locked-accelerator contract. This flag is reserved for a future launch mode that provides that lock itself. |
| `features.history` | `true` | The one-shot `back`, `forward`, and `reload` commands and typed exhaustion result are available. |
| `features.live_inventory` | `true` | `session info` reads the current live page inventory without activating a page. |

Feature values describe the installed binary, are additive, and may change
independently in later releases. Consumers must gate kiosk launch on
`features.kiosk_launch` itself, never on the package version or the value of a
different feature. An older mixed-template binary that omits the field does
not advertise this contract.

## Caller-Requested Desktop Kiosk Launch

Bowser forwards caller Chrome arguments after its own baseline and stealth
arguments. Chrome's normal last-occurrence behavior therefore gives caller
values precedence. Bowser rejects only the remote-debugging and user-data-dir
switches it owns for session identity.

`features.kiosk_launch: true` covers this forwarding and precedence rule, the
geometry-conflict suppression below, the verified fullscreen page workflows,
and content-tab discovery through `features.live_inventory`. It does not claim
that native Chrome blocks takeover accelerators.

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
with raw X keyboard input. The chords were injected into real headed Chrome on
the same fixed 1024×768 X display used by the kiosk integration test. Bowser
was detached while input was sent; subsequent live inventory reads observed
tab creation, removal, focus, and selection. X root-window state and byte-for-
byte screen captures checked F11.

The filter action is the stable takeover policy paired with
`features.kiosk_launch`. `Block` means the platform must intercept the chord
before Chrome. `Allow` means the chord does not break containment in the
verified workflow:

| Accelerator | Native result | Filter action |
|---|---|---|
| F5 | Reloads the page. | Allow. |
| Ctrl+R | Reloads the page. | Allow. |
| Alt+Left | Traverses backward when an entry exists. | Allow. |
| Alt+Right | Traverses forward when an entry exists. | Allow. |
| Ctrl+F | Opens Chrome's find overlay. | Allow. |
| Ctrl+L | Does not expose or focus the hidden omnibox. | Allow; observed inert. |
| Ctrl+T | Opens and selects a new tab. | Block. |
| Ctrl+Shift+T | Reopens and selects the most recently closed tab. Live inventory removes the closed target and discovers it again after reopening. | Block. |
| Ctrl+W | Closes the active tab, or the window when it is the last tab. | Block. |
| Ctrl+Shift+W | Closes the complete kiosk window and ends the browser session. | Block. |
| Ctrl+N | Opens another Chrome window. | Block. |
| Ctrl+Shift+N | Opens and focuses a second, non-fullscreen incognito window whose new-tab target appears in live inventory. | Block; this is the highest-risk browser escape. |
| Ctrl+Tab | Selects the next tab; the next live inventory read reconciles `selected_page_id` to it without activation. | Allow; safe when user tab switching is intended. |
| Ctrl+Shift+Tab | Selects the previous tab with the same live-inventory reconciliation. | Allow; safe when user tab switching is intended. |
| F11 | Has no observed effect: the window remains fullscreen and the screen capture is unchanged. | Block defensively because it is a window-mode command outside the launch capability contract. |
| Ctrl+P | Opens and selects Chrome's `chrome://print/` preview target over the page. | Block. |

Allowing Ctrl+Tab and Ctrl+Shift+Tab is coherent with
`features.live_inventory`: the active content tab changes, and the next
side-effect-free `session info` poll reports that tab as selected. A platform
that intentionally freezes takeover to one tab may still block the chords as
a product policy; blocking is not required for kiosk containment or Bowser
selection correctness.

Chromium's
[desktop switch definition](https://chromium.googlesource.com/chromium/src/+/HEAD/chrome/common/chrome_switches.cc)
explicitly describes `--kiosk` as distinct from ChromeOS kiosk mode. Because
tab and window accelerators remain active, Bowser keeps `features.kiosk`
false. The independently verified launch and page-workflow contract is
reported as `features.kiosk_launch: true`. A platform that applies the filter
policy above may deliberately use that launch behavior, but it must not infer
a locked accelerator contract or gate behavior on Bowser version numbers.

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
