## Overview

Bowser is a Rust CLI tool and library that runs a Chrome instance to render web pages and produce a structured YAML representation of the rendered page content. It strips away CSS and layout noise, presenting only the semantic elements that matter: text, links, buttons, inputs, images, headings, tables, and lists. With stealth enabled, Bowser uses a headed Chrome shape by default because native headless Chrome is rejected by Google-style smoke flows; `--no-stealth` keeps the plain headless automation path available for environments that need it.

Pages are fully rendered (JavaScript executed, SPAs hydrated, dynamic content loaded) before capture. Bowser applies best-effort stealth patches to reduce obvious automation fingerprints and approximate a regular Chrome browser, but it does not guarantee bypassing bot detection.

Bowser supports two modes:

- **Single-shot**: navigate to a URL, output the YAML, detach from the browser, and print a resumable session ID.
- **Interactive (REPL)**: navigate, inspect, click, type, go back/forward, take screenshots, then exit while preserving the session for later reuse.

All CLI commands run against a resumable browser session. If `--session <ID>` is not provided, Bowser creates a new session. If `--session <ID>` is provided, Bowser reattaches to the existing detached session and continues from its current browser state.

## Session Lifecycle

A Bowser session is a detached browser instance plus persisted metadata, identified by an opaque session ID. Generated IDs use `bsr_` followed by ASCII letters, digits, `_`, or `-`, with a maximum total length of 128 bytes. The filesystem store rejects every other shape before constructing a path, and the metadata document's ID must match its `bsr_<ID>.json` filename.

Sessions preserve:

- the browser process
- cookies and storage
- browsing history
- open page state needed to continue later commands across multiple live pages

Sessions do not preserve element IDs across captures. On resume, the next capture rebuilds the DOM mapping and assigns fresh IDs.
IDs for truncated expandable containers are also ephemeral and are rebuilt on every capture.

Session behaviour:

1. `bowser get` and bare-URL invocation (`bowser <URL>`) create or resume a session, perform the requested navigation/capture, then detach while leaving the session available for reuse.
2. `bowser interactive` creates or resumes a session, runs the REPL, then detaches on `quit` / `exit`.
3. On normal command completion, Bowser prints the active session ID to `stderr` as `Session: <ID>` so structured YAML/JSON output on `stdout` remains clean.
4. Detached sessions remain resumable until they are explicitly closed or their idle TTL expires.
5. Resuming a session with a new command continues from the current browser state before applying any new navigation or interaction requested by that command.

## Session Pages

A detached Bowser session may contain multiple live pages at the same time.

Bowser assigns each tracked page a stable session-scoped page ID such as `pg_1`, `pg_2`, and so on. One of those pages is always the selected page. Each record also carries a page type; today Bowser stores `tab`, but the detached-session contract remains page-centric.

Page IDs are distinct from element IDs:

- page IDs persist across captures and detach/resume while that page remains in the session
- element IDs are rebuilt on every capture and are only valid for the most recent capture

The selected page is the default target for:

- `bowser get`
- `bowser interactive`
- `bowser expand`
- `bowser meta`
- `bowser describe`
- all REPL element commands such as `click`, `type`, `submit`, and `yaml`

Resume and page-selection behaviour:

1. If the stored selected page is still live, Bowser reattaches to that page.
2. If the selected page is gone, Bowser chooses the best remaining live non-blank page and marks it selected.
3. If no live non-blank pages remain, Bowser may fall back to the best remaining page, including `about:blank`.
4. If closing the last remaining page leaves the session with no live pages, Bowser creates and selects a replacement `about:blank` page so the session remains resumable.

Live page reconciliation must bound Chrome URL/title metadata probes and fall back to stored page metadata when a target is stale or slow to answer. Page listing, switching, creating, and closing pages must not wait for an unbounded target-state read.

Legacy single-page session metadata should migrate forward to the multi-page metadata shape on first load or list. Loading and listing must use the same migration path so one legacy entry cannot prevent session cleanup or enumeration.

Session close and expiry cleanup treat persisted PIDs as untrusted, reusable operating-system identifiers. Bowser may terminate the stored Chrome PID only when the live command line still contains both the stored remote-debugging port and user-data directory. It may terminate an Xvfb PID only when its executable and display still match the stored session. A newly launched Chrome/Xvfb pair is guarded separately as known-owned: any connection or metadata-persistence failure terminates both before the launch returns an error.
