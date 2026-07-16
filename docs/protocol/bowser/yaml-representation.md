## YAML Page Representation

### Design Principles

- Capture **semantic content only** — no CSS, no layout divs, no style attributes.
- Flatten presentational wrappers — a `<div>` that only exists for layout is not represented; its children are promoted.
- Every command-targetable element gets a **numeric ID**.
- The YAML is a **tree** reflecting the logical nesting of the page (e.g., a list item inside a list, a cell inside a table row).
- YAML output should favor a **compact human-readable form** over a direct dump of Rust struct field names.
- Compact YAML is a **display format**. Full-fidelity machine-readable export is provided by JSON and metadata lookup commands.

### Element Types

The following element types are captured:

| Type | HTML sources | Compact YAML form |
|---|---|---|
| `page` | Root | top-level `url`, `title`, `content.visible`, and `content.obscured` |
| `heading` | `<h1>`–`<h6>` | `h1: "..."` through `h6: "..."` |
| `text` | `<p>`, `<span>`, `<label>`, text nodes, `<blockquote>`, `<pre>`, `<code>` | `text: "..."` |
| `link` | `<a>` | `link#12: "..."`; when focused: `link#12: { text: "...", focused: true }`; fetch `href` with `meta 12` |
| `button` | `<button>`, `<input type="submit">`, `[role="button"]` | `button#13: "..."`; when focused: `button#13: { text: "...", focused: true }` |
| `input` | `<input>`, `<textarea>`, `<select>`, `[role="checkbox"]`, `[role="radio"]`, `[role="switch"]` | `input#14: { type: text, name: q, placeholder: Search, value: "" }`; non-empty password values render as the fixed `[redacted]` marker; add `focused: true` when active |
| `image` | `<img>`, `<svg>` (meaningful), `<picture>`, `[role="img"]` | `image#15: "Alt text (filename.png)"`; when AI description is available: `image#15: { label: "Alt text (filename.png)", describable: true }`; when focused add `focused: true`; fetch `src` and full metadata with `meta 15`, and fetch the explicit description with `describe 15` |
| `table` | `<table>` | `table#16: { headers: [...], rows: [{ cells: [...] }], truncated: ... }`; add `focused: true` when active |
| `list` | `<ul>`, `<ol>`, `<dl>` | `ul#17: [...]`, `ol#17: [...]`, or `dl#17: [...]`; when truncated, use `items` plus `truncated`; add `focused: true` when active |
| `nav` | `<nav>` | `nav#18: [...]`; when truncated, use `content` plus `truncated`; add `focused: true` when active |
| `form` | `<form>` | `form#19: { action: "...", content: [...] }`; add `focused: true` when active |
| `section` | `<section>`, `<article>`, `<aside>`, `<main>`, `<header>`, `<footer>`, promoted dialog containers, and the synthetic root `body` preview container when top-level content is truncated | semantic key such as `article#20: [...]`, `main#20: [...]`, `footer#20: [...]`, `dialog#20: [...]`, or `body#20: { content: [...], truncated: ... }`; add `focused: true` when active |
| `iframe` | `<iframe>` (same-origin, same-target cross-origin, or target-backed cross-origin live frames) | `iframe#21: { src: "...", content: [...] }`; add `focused: true` when active |

Table headers and rows preserve nested content. Each header cell and body cell contains `children`, so links, buttons, and inputs inside tables remain represented as elements instead of being flattened to strings.

In compact YAML, `headers` is a list of header-cell payloads and `rows` is a list of `{ cells: [...] }` objects. A cell containing one simple element serializes directly as that element. A cell containing multiple elements uses `{ content: [...] }`.

Collection-bearing nodes may also include an optional `truncated` field when the captured preview omits additional items. `truncated` has:

- `shown`: number of entries included in the preview
- `total`: total number of entries available

Truncation can apply to:

- `list.items`
- `table.rows`
- `nav.children`
- `form.children`
- `section.children`
- `iframe.children`
- root `content.visible`
- root `content.obscured`

### Root Visibility Buckets

Root page content is split under `content`:

- `visible`: rendered semantic content available to a user. Bowser captures the
  full rendered document, so below-the-fold content remains in `visible`.
- `obscured`: semantic content that is present but not currently visible because
  it is style-hidden, zero-sized, or covered by a blocking modal/backdrop.

The split is root-level only. Nested containers keep their normal compact
shape. Hidden inputs remain inside their owning form when the form itself is
captured.

### Deferred Metadata

To keep compact YAML small, default YAML output omits:

- link `href`
- image `src`
- image `description`

Those fields, along with per-element runtime visibility details, current focus state, and current page-space bounds, are retrieved on demand with the metadata command:

- CLI: `bowser meta --session <SESSION_ID> <ELEMENT_ID>`
- REPL: `meta <ID>`

Explicit image descriptions are retrieved on demand with:

- CLI: `bowser describe --session <SESSION_ID> <ELEMENT_ID>`
- REPL: `describe <ID>`

Reference form:

- element ID, such as `12`

Runtime visibility in `meta` uses these fields:

- `present`: element still exists in the live DOM
- `in_viewport`: some part of the rendered box intersects the current viewport
- `obscured`: the rendered center point is covered by another element
- `enabled`: the element is not disabled or `aria-disabled`
- `visible`: the element is rendered, in viewport, and not obscured
- `bounds.top_left`: the current top-left page-space coordinate `[x, y]`
- `bounds.bottom_right`: the current bottom-right page-space coordinate `[x, y]`
- `clickable`: the element is currently visible and its center point can receive pointer input

Metadata output also includes:

- `focused`: whether the element is the active focused element in its document right now
- `describable`: for images only, whether the current Bowser AI configuration can satisfy `describe` right now

Cross-origin iframe descendants are included in the numbered capture tree and
work with subtree-oriented capture features such as `expand`. Target-backed
iframe descendants support frame-aware `click`, `meta`, `type`, `clear`,
`select`, `submit`, `scroll` to element, element screenshot, and `describe`
routing when the live iframe target is still available. Non-ID-routed commands
such as free-form JS evaluation, selector waits, and raw keypresses remain
attached to the active page target. Same-target cross-origin iframe descendants
still only operate on the current document and same-origin descendants until
broader frame-aware runtime routing is implemented.

Top-level screenshots preserve target-backed iframe compositor surfaces by
capturing a full-content CDP clip rather than resizing the viewport before
capture. This is separate from structural YAML capture; the YAML tree is still
semantic, while screenshots remain visual.

### Compact YAML Syntax

The compact YAML display output uses these conventions:

1. The root document is:

```yaml
url: "https://example.com"
title: "Example"
content:
  visible:
    - ...
  obscured:
    - ...
```

2. Each entry in `content.visible` and `content.obscured` is a single-key mapping.
3. Every interactive, semantic-container, or metadata-addressable element encodes its ephemeral ID in the key as `<type>#<id>`, for example `link#6`, `article#7`, `table#8`, `image#9`, or `input#10`.
4. Primary YAML only includes `focused: true` when an ID-bearing element is currently focused. Unfocused elements omit the field entirely.
5. Link and button values serialize as plain text by default. When focused, they serialize as mappings with `text` plus `focused: true`. Image labels are built from captured `alt` text plus the source filename when available. Images serialize as a mapping whenever `focused: true` or `describable: true` must be shown, using `label`.
6. Container elements remain in the compact form whenever possible. If a focused container needs to show focus state, or if extra fields such as `action` or `truncated` are required, it serializes as a mapping with `content`, `items`, `cells`, or `rows` plus the extra metadata.
7. In tables, `headers` is a list of cell payloads and each entry in `rows` is a `{ cells: [...] }` object to avoid ambiguous nested lists in display YAML.

This compact YAML is the default CLI output and the output of `to_yaml`. It is intentionally lossy. Full-fidelity serialization and deserialization are provided by JSON.

### Element ID Assignment

Sequential integer IDs are assigned in compact output order, with
`content.visible` emitted before `content.obscured`, to:

- interactive elements: `link`, `button`, non-hidden `input`, and ARIA checkable controls
- semantic container elements: `table`, `list`, `nav`, `form`, `section`, `iframe`
- images, so non-interactive visual content can still be inspected with `meta <ID>`

When Bowser stitches an accessible cross-origin iframe subtree into the parent
capture, those descendants participate in the same single document-order ID
sequence as the rest of the page instead of restarting numbering inside the
iframe.

Text nodes, headings, and hidden inputs are captured without IDs. Container IDs are always available for `yaml <ID>` and other subtree-targeted commands, while `expand <ID>` is only meaningful when that container was truncated in the preview capture. The internal full capture also assigns an optional `body_id` to `document.body` so top-level truncation can be surfaced as an expandable `body#<ID>` preview node when needed.

Interactive commands such as `click`, `type`, `select`, and `submit` only work for compatible elements. If a command targets an element that exists but does not support that interaction, Bowser returns `ElementNotInteractable`. For visible targets, `click` should prefer a browser-level pointer sequence over a synthetic DOM click: Bowser should scroll the target into view when needed, move a remembered cursor toward a sampled point inside the target's 10px-inset rectangle using the CLI pointer telemetry fixture as the general movement pattern, dispatch a real mouse press/release there, and preserve normal activation behavior while also focusing focusable controls so post-click captures can report `focused: true` for the active element. The first visible click in a live attached page may start from a randomized in-viewport idle position. Movement should remain randomized while following the fixture-shaped phases of idle drift, coarse travel, slower approach, and final micro-corrections. If the target is too small for a full 10px inset, Bowser may use the nearest valid interior point. If the target is not visibly clickable, or the pointer path fails or stalls, Bowser may fall back to the synthetic DOM click path.

Hidden elements (`display: none`, `visibility: hidden`, zero-size) are emitted under `content.obscured` when they produce semantic content. `<input type="hidden">` is always preserved inside its owning form instead of becoming a separate root entry. `aria-hidden="true"` alone does not suppress capture if the subtree is visibly rendered. Rendered content below the fold remains under `content.visible` even when it is currently outside the viewport.

`-a` / `--all` mode disables truncation for that command. Hidden and obscured semantic elements are included by default through `content.obscured`.

### Collection Truncation

Large collections are previewed by default so page output stays readable. The capture itself is full-document and includes rendered content beyond the current viewport. Preview truncation then keeps the leading portion of large collections and records `truncated` metadata on the truncated element itself.

Default preview limits:

- `list.items`: first 20 items
- `table.rows`: first 20 rows
- `children` arrays on container elements: first 50 children

If either root bucket exceeds the configured child limit, Bowser wraps that bucket's preview in a synthetic `body#<ID>` section containing the first `max_children` root entries plus `truncated: { shown, total }`. `expand <ID>` on that `body#<ID>` returns the full bucket subtree.

Expanding a truncated node always targets the node's own ephemeral element ID, which is only valid for the most recent capture in the active session.

### Image AI Summarization

Compact page YAML always labels images from the captured DOM-facing metadata:

1. Prefer the captured `alt` text.
2. When the `src` has a visible filename, append it as `Alt text (filename.png)`.
3. If there is no usable `alt`, fall back to the filename, then the cached AI `description`, then `[image]`.

If the current AI configuration can actively describe images, compact YAML may also add `describable: true` to that image node. For Anthropic and OpenAI providers, this requires the configured API-key environment variable to be present and non-empty. For Ollama, enabled local configuration is sufficient.

The explicit `describe` command:

1. Resolves the image by captured element ID.
2. Returns any cached `description` if one already exists for that image in the current session state.
3. Otherwise screenshots the image element, sends it to the configured AI model endpoint, and caches the returned description back into session state. Same-document images use a page-space CDP clip; target-backed iframe images render the actual `<img>` to an element-local PNG inside the iframe target.
4. Returns a structured payload containing `element_id`, `alt`, `src`, `filename`, and `description`.

If AI summarization is disabled or unavailable and no cached description exists, `describe` returns an error instead of silently fabricating a description from `alt`.

Configured in `~/.config/bowser/config.yaml`, via environment variables, or via CLI flags where supported:

```yaml
# ~/.config/bowser/config.yaml or via CLI flags
ai:
  provider: "anthropic"        # anthropic | openai | ollama
  model: "claude-sonnet-4-20250514"
  api_key_env: "ANTHROPIC_API_KEY"  # env var name containing the API key
  endpoint: null               # Ollama endpoint URL
  enabled: true                # set false to disable AI summarization
```
