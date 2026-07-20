# Managed Agent And Browser Runtime

This page defines how managed browsers run in E2B and how the Bowser agent
controls them.

## Execution Split

Each managed browser uses one E2B sandbox created from an immutable Bowser
template build. The sandbox contains Chrome, a display server, noVNC, the
`bowser` library, and a small HTTP/WebSocket runtime. It contains no agent loop
and no durable product database.

The GCP worker owns task orchestration, model calls, tool selection, durable
checkpoints, cancellation, quotas, and provider credentials. It calls the
sandbox runtime through a trait-backed E2B provider adapter.

## E2B Template Contract

The template must:

- pin Chrome and the Bowser sandbox-runtime binary to tested build identities;
- start Xvfb, the browser runtime, and noVNC before the template readiness
  check succeeds;
- run browser and runtime processes as an unprivileged user;
- expose the runtime and noVNC only through E2B restricted traffic;
- include health, readiness, and version endpoints that disclose no secrets;
- contain no environment-specific API keys or proxy credentials;
- produce its template/build ID in CI and require explicit promotion between
  development, staging, and production tags.

The runtime begins uninitialized. The worker authenticates through E2B secure
access, sends one session configuration, and establishes a second random
session capability. Later calls require both protected E2B transport and that
capability. Initialization is idempotent for an identical browser ID and
rejects all other reuse.

E2B IDs, access tokens, traffic tokens, runtime capabilities, and endpoint
hosts are encrypted at rest and never sent to a public client.

## Browser Lifecycle

Creation performs these steps in order:

1. reserve project concurrency and create the Bowser browser row;
2. resolve an immutable E2B template build and egress configuration;
3. create a secure sandbox with restricted public traffic and metadata that
   contains only the Bowser environment and non-secret internal browser ID;
4. initialize the runtime, restore an optional profile, and launch Chrome;
5. verify the runtime version, browser identity, viewport, DNS path, and exit
   IP/application-level response;
6. persist provider state and transition the browser to `active`;
7. emit a ready event and release the creation lease.

The initial E2B timeout is five minutes with full-memory auto-pause and
auto-resume. Bowser owns a separate absolute lifetime deadline, defaulting to
four hours, which E2B activity cannot extend. A worker refreshes product state
after a provider resume. Stopping closes Chrome, persists an attached profile,
collects final metrics, kills the sandbox, and only then marks the browser
stopped. Cleanup is retried and reconciled if a provider call fails.

## Runtime Commands

The sandbox runtime accepts typed commands matching the hosted API union and
returns typed observations. It serializes mutations for a browser. Every call
includes command ID, deadline, cancellation generation, and trace context.

Capture results receive a durable `capture_id` at the worker boundary. The
sandbox tracks the matching current DOM generation. Element actions require
both values and fail closed after navigation, reload, page switch, or any
capture-changing operation.

The runtime streams bounded progress and stores temporary downloads only under
a per-command directory. The worker copies final artifacts to GCS, verifies a
digest, records metadata, and asks the sandbox to delete the temporary bytes.

## Egress Routing

“VPN support” in the hosted product means verifiable browser egress routing,
not an unverified claim that a full tunnel runs inside the sandbox.

- `direct` uses E2B's normal outbound route.
- `country` selects a Bowser-managed proxy endpoint for an ISO 3166-1 alpha-2
  country.
- `profile` uses an encrypted project-owned HTTP(S) or SOCKS5 egress profile.
- E2B's sandbox-level `network.egressProxy` is the preferred enforcement point
  so Chrome, DNS-supporting helpers, and downloads share one route.
- Browser-level `--proxy-server` is a tested fallback only if E2B cannot route
  a supported proxy protocol; it must pass the same DNS and download tests.
- Egress configuration is immutable after browser creation.
- Proxy credentials are decrypted only for sandbox creation, never returned,
  and redacted from provider errors and all logs.
- A configured route must verify its expected country or fixed exit before the
  browser becomes ready. Failure stops the sandbox; it never falls back to
  direct egress.

Locale, timezone, Accept-Language, and browser identity must be coherent with
the selected route. Bowser does not promise CAPTCHA bypass or anonymity.

E2B documents outbound controls and an `egressProxy` object in its
[create-sandbox API](https://e2b.dev/docs/api-reference/sandboxes/create-sandbox),
but does not document WireGuard/OpenVPN or TUN privileges. A live capability
spike and E2B confirmation are required before marketing a full VPN tunnel.

## Live View And Recording

noVNC binds inside the sandbox. Public clients receive a short-lived Bowser
live-view token and connect to a Bowser reverse proxy, which authorizes project
membership and forwards HTTP/WebSocket traffic with server-side E2B
credentials. The proxy is view-only by default; interactive control is a
separate permission and audit event.

Live URLs expire within five minutes and are bound to user, project, browser,
and permission. The dashboard reconnects after E2B resume. Recording is opt-in,
has an explicit retention period, and is represented as an artifact after the
browser stops.

## Browser Profiles

A profile is project-scoped encrypted Chrome state. At most one active browser
may hold its lease. Creation restores the latest complete generation before
Chrome starts. Graceful stop closes Chrome, archives only the owned profile
directory, encrypts and uploads it, verifies its digest, and atomically
advances the generation.

Partial uploads never replace the last good generation. A crash may lose the
current session's unflushed profile changes but must not corrupt the prior
generation. E2B volumes remain a possible optimization, not a launch
dependency, because the feature is currently
[private beta](https://e2b.dev/docs/volumes).

## Agent Runtime

Bowser depends on the separate Futex AI workspace at
`https://github.com/futex-ai/ai`, pinned to a reviewed Git revision. Initial
integration uses `ai-interface`, `ai-tool-calling`, the required provider
adapter crates, and model-policy wrappers. Local path overrides may be used for
development but may not be the CI or release dependency.

The in-memory `ToolCallingRuntime` is reconstructed from persisted messages
and provider replay context for every leased task. Its checkpoint hooks persist
the validated model response before any tool side effect and persist each tool
result before the next model round. `ToolInvocation.operation_id` is the
idempotency key for browser side effects. The AI `Logger` implementation writes
model, tool, activity, usage, and terminal turn records through Bowser's store
interface.

The initial browser tool catalog is intentionally small:

- `observe` captures the selected page or expands/reads metadata;
- `navigate` changes URL or history;
- `interact` clicks, types, selects, submits, presses keys, or scrolls;
- `pages` lists, opens, selects, or closes pages;
- `artifact` takes a screenshot or downloads a browser resource;
- `finish` returns a final text or schema-conforming result.

Tool inputs are fully typed JSON Schema. Observations are bounded and may
reference an artifact rather than inline large content. Screenshots are sent
to a vision-capable model only when the agent explicitly requests one; image
parts are removed from public history after the model call.

## Policy, Budgets, And Cancellation

Task action policies are `read_only`, `confirm_sensitive`, and `unrestricted`.
The default is `confirm_sensitive`. Form submission, file upload, purchase,
message sending, account changes, and destructive actions pause the task with
a typed confirmation request. The chat playground never selects
`unrestricted`.

Each task has hard step, wall-clock, model-cost, browser-lifetime, and artifact
limits. The worker checks cancellation and budget at every checkpoint and
between tool calls. It does not report success unless the agent produces a
final result and, when requested, that result validates against the output
schema. Hidden chain-of-thought is neither requested nor stored; persisted
step summaries describe observable actions and outcomes only.

## Reconciliation

A periodic reconciler compares non-terminal browser rows with E2B sandboxes
tagged for the environment. It marks missing providers failed, repairs
paused/active state, retries stops, kills confirmed Bowser-owned orphans, and
records every repair. It never kills an E2B sandbox whose metadata does not
match the configured environment and Bowser ownership marker.
