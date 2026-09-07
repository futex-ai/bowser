# Transient Page Favicons

`features.page_favicons: true` advertises opt-in, browser-context favicon
acquisition. Callers must negotiate this flag independently of the binary
version; a missing flag means unsupported.

```sh
bowser --json-envelope session info <SESSION_ID> --include-favicons
```

The result is `result.session.pages`, an ordered array containing `page_id`,
`url`, nullable `title`, `selected`, and nullable `favicon`. A non-null favicon
contains only `png_base64`, canonical padded standard base64 for a transparent,
metadata-free 32-by-32 PNG. The string is at most 6 KiB. Exactly one live page
is selected, using the same passive reconciliation as ordinary live inventory.
Plain terminal output contains page metadata without image bytes.

The library entry point is `BrowserEngine::live_session_pages`, returning
`LiveSessionPage` values. `BrowserFavicon` debug output redacts its image data.
Ordinary `session info`, page summaries, captures, stored session metadata, and
portable checkpoints remain icon-free. Acquired icons are transient and are
never written to Bowser's session store or diagnostic output.

## Discovery

Read the current document URL, loader identity, base URI, and eligible
`link[rel]` declarations. Treat `rel` as case-insensitive whitespace-separated
tokens, accept `icon`, ignore disabled links and nonmatching media, resolve
relative URLs against `document.baseURI`, and deduplicate resolved addresses.
Prefer `sizes="32x32"`, then `any`, then other sizes. Within one preference,
prefer the last declaration in document order. Try at most three declared
candidates, followed by the page origin's real `/favicon.ico` if not already
tried. Failure allows the next candidate within the shared deadline.

Only HTTP(S) documents participate. Candidate URLs may use HTTP(S) or bounded
image data URLs; unsupported schemes and addresses over 1 MiB are ignored.
HTTP(S) resources use Chrome's existing browser context, including applicable
cookies, cache, referrer, and image content policy. No independent HTTP client,
favicon directory, viewer request, hidden tab, or origin-specific exception is
used. Cross-origin icons do not require CORS permission for a script read.

The transport creates an off-DOM image in an isolated world. A private CDP
initiator marker identifies only that acquisition's network request, including
its redirects. Unrelated page requests continue normally. Response interception
reads bounded chunks, allows at most three redirects, and aborts the owned
image request after reading its source. Cleanup clears the isolated image,
closes its stream, and restores interception/debug-stack settings. Protocol
cleanup has reserved time before renderer cleanup, so a busy document cannot
leave page requests intercepted. The isolated world is reused across polls,
with a new owned image marker for each acquisition. Cleanup never
disables the Runtime domain used by ordinary browser operations.

## Normalization And Time Bounds

PNG, ICO, JPEG, GIF, WebP, and the SVG subset below are accepted. Read at most
256 KiB of source, require dimensions no larger than 1,024 by 1,024, and limit
raster decoder allocation to 16 MiB. Use the first animation frame, preserve
aspect ratio and colors, and center the image on a transparent 32-by-32 canvas.
Unsupported, unavailable, malformed, or oversized content produces null.

SVG must be uncompressed UTF-8 XML of at most 64 KiB, with at most 512 nodes,
depth 32, and 1,024 geometry segments/points/transforms within 16 KiB of geometry
attributes. Allowed elements are `svg`, `g`, `defs`, `path`, `rect`, `circle`,
`ellipse`, `line`, `polyline`, `polygon`, `linearGradient`, `radialGradient`,
`stop`, `title`, and `desc`. DTDs, `href`, CSS, dashed strokes, geometry-expanding
references, filters, masks, clipping, patterns, markers, embedded images, and
text are rejected before rendering. Rasterization cannot execute scripts or
resolve external images. This subset prevents small SVG reference graphs from
expanding into unbounded work.

All pages share one additional one-second deadline after ordinary live
metadata reconciliation, with at most four acquisitions in flight. The
transport reserves cleanup time within this deadline; it does not start a new
per-candidate timeout. Cancel unfinished acquisitions and return null for their
icons. Completed pages retain their results in inventory order. Optional icon
failure cannot fail an otherwise valid inventory or alter session selection.

Require the inspected URL to match the inventory metadata. Recheck document
loader identity, URL, base URI, and eligible declarations after acquisition.
Discard images if any of these changed, including a reload at the same URL.
Later inventory reads may retry within the same bounds, without a background
retry loop or additional persistent cache.

## Verification

Pure trait-boundary regressions cover candidate ordering, bounds, invalid
images, race rejection, scheduling, and fallback. `favicon_inventory` runs real
Chromium against local fixtures for relative/base/root icons, background tabs,
SVG/ICO/data sources, cross-origin authenticated resources under a restrictive
connect policy, redirect limits, cancellation, and document changes. CLI tests
verify capability output, opted-in JSON, and icon-free durable/ordinary output.
`cargo xtask check` includes the dedicated integration target.
