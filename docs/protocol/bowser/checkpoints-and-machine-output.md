# Checkpoints And Machine Output

## Portable Session Checkpoints

Bowser checkpoint files are plaintext, self-contained JSON documents. Callers
that require encryption must encrypt the file outside Bowser. Version 1 has
this top-level shape:

```json
{
  "checkpoint": 1,
  "created_at": "2026-08-13T12:00:00Z",
  "cookies": [],
  "origins": [],
  "pages": [
    { "url": "https://example.com/account" }
  ],
  "selected_page": 0
}
```

`cookies` contains Bowser's typed portable subset of Chrome DevTools Protocol
cookie records, including HttpOnly cookies. `origins` contains one record per
successfully read HTTP(S) origin of an open tab, with that origin's complete
localStorage key/value map. `pages`
preserves the ordered open-tab URLs, and `selected_page` is a zero-based index
into that array. A checkpoint must contain at least one page and the selected
index must be in range.

`bowser session export --session <ID> --to <PATH>` attaches to the live
session, snapshots state through CDP, atomically writes one checkpoint, and
detaches. It does not copy live Chrome profile files and leaves the original
session usable. Export rejects pages whose current URL cannot be determined;
non-HTTP(S) pages are preserved as tabs but have no localStorage record.

Cookie restore retains name, value, domain, path, expiry, Secure, HttpOnly,
SameSite, and partition-key state. Priority and source metadata remain in the
v1 document for diagnostics but are not forced onto a newer Chrome, allowing
Chrome to derive compatible values during restore.

`bowser session restore --from <PATH>` validates the complete checkpoint
before launching Chrome, always creates a fresh `bsr_*` session with a fresh
Bowser-owned profile, installs cookies, visits each recorded page, restores
localStorage for its origin before a final reload, and selects the recorded
page. Chrome-created startup tabs are normalized down to the one page reused
for restore before saved tabs are opened. The new session uses the caller's
current Chrome, Bowser configuration, session directory, and launch flags. No
PID, CDP endpoint, source profile path,
page target ID, page ID, or element ID is stored or trusted. Restored page IDs
are newly allocated.

Checkpoint v1 is designed to cross hosts and Chrome/Bowser upgrades. Readers
reject unknown checkpoint versions. Within version 1, new optional fields may
be added, but existing field meanings do not change. CDP may decline individual
expired or policy-invalid cookies during restore; any such rejection fails the
restore rather than silently weakening login state.

Version 1 intentionally excludes IndexedDB, sessionStorage, service workers,
Cache Storage, HTTP cache, browsing-history stacks, downloads, permissions,
extensions, and arbitrary profile files. It preserves login state only to the
extent represented by cookies and localStorage.

## JSON Command Envelope

`--json-envelope` is an opt-in global flag for non-interactive commands. It
makes Bowser write exactly one compact JSON object followed by a newline to
stdout. Logs and human diagnostics remain on stderr. The existing process exit
status is retained; the envelope is the machine-readable authority.

```json
{
  "envelope": 1,
  "ok": true,
  "session": "bsr_...",
  "page": "pg_...",
  "result": {},
  "error": null
}
```

`session` and `page` are nullable and are populated whenever Bowser has learned
them, including on a later command failure. Success has a command-specific
object in `result` and a null `error`. Failure has a null `result` and:

```json
{
  "code": "session_not_found",
  "message": "[bowser/session] session not found: bsr_missing",
  "detail": { "session_id": "bsr_missing" }
}
```

Envelope version 1 is additive: consumers must ignore unknown result/detail
fields. Stable error codes are:

- `browser_launch`, `browser_disconnected`, `page_target_invalid`,
  `page_close`, `page_close_timeout`
- `session_not_found`, `session_expired`, `invalid_session_id`,
  `session_identity_mismatch`, `session_page_not_found`,
  `session_page_not_live`, `session_persist`, `session_detach`
- `checkpoint_read`, `checkpoint_write`, `checkpoint_invalid`,
  `checkpoint_unsupported`, `checkpoint_capture`, `checkpoint_restore`
- `navigation`, `history_exhausted`, `timeout`, `element_not_found`, `element_not_interactable`,
  `key_press`, `capture`, `expand_not_found`, `metadata_not_found`,
  `describe_not_image`, `describe_unavailable`, `ai`, `screenshot`,
  `download`, `javascript`, `configuration`, `output_write`,
  `invalid_arguments`, `invalid_command`, `io`, `cdp`, and `internal`

Closely related implementation variants intentionally collapse to one public
code when a caller cannot act differently. `detail` provides typed identifiers
and paths where available and otherwise remains an empty object.

### Result payloads

- Capture-bearing `get`, `capture`, interaction, and `page select|new|close`
  commands return `{ "capture": <PageCapture> }`. With `--output`, they return
  `{ "output": { "path": "...", "format": "..." } }` instead. HTML output
  is represented as `{ "html": "..." }` when not written to a file.
- Capture-bearing `back`, `forward`, and `reload` use the same result shapes.
  An exhausted back or forward fails with `history_exhausted` and
  `{ "direction": "back" | "forward" }` in `error.detail`.
- `get --screenshot` additionally returns `screenshot.path` and
  `screenshot.bytes`.
- `download` returns `path` and `bytes`.
- `expand`, `meta`, and `describe` embed their existing JSON models, or an
  output path/format reference when written to a file.
- `page list` returns `pages`; `session list` returns `sessions`; `session info`
  returns `session`; and `session close` returns `closed: true`.
- `session export` returns checkpoint path/version and state counts. `session
  restore` returns restored state counts; the envelope's `session` and `page`
  fields identify the fresh session and selected page.
- `capabilities` returns the capability object described below.

Interactive mode and the developer-only pointer telemetry server reject
`--json-envelope` because they are streaming interfaces rather than one-shot
commands.

## Capabilities

`bowser capabilities --json-envelope` returns its capability data inside the
normal envelope. Without the flag it prints the same capability object as
plain JSON for easy template inspection:

```json
{
  "version": "0.4.0",
  "envelope_versions": [1],
  "checkpoint_versions": [1],
  "features": {
    "checkpoint": true,
    "kiosk": false,
    "kiosk_launch": true,
    "history": true,
    "live_inventory": true,
    "page_favicons": true,
    "window_target": false
  }
}
```

`version` is the installed CLI package version. Feature booleans describe the
current binary honestly. Verified caller-requested kiosk launch, history
verbs, read-only live inventory, and [transient page favicons](./page-favicons.md)
are available. Desktop Chrome's unlocked
tab/window kiosk accelerators and deferred window-target reporting keep
`kiosk` and `window_target` false respectively. Consumers must detect
`kiosk_launch` directly before appending `--kiosk`; package versions are not a
feature gate across mixed template releases.
