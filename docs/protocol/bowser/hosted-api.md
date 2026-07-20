# Hosted API

This page defines the public HTTP and event-stream contract for the managed
product. The checked-in OpenAPI document becomes the machine-readable source
of truth when the API milestone is implemented.

## Transport And Authentication

- The base path is `/v1`; incompatible changes require a new version.
- JSON request and response bodies use `snake_case` keys and RFC 3339 UTC
  timestamps.
- Customer applications send `Authorization: Bearer <project-api-key>`.
- The Expo app sends an Identity Platform ID token. The API maps its subject to
  a Bowser user and project membership. Project-scoped dashboard requests also
  send `X-Bowser-Project: <project-id>`; the server never trusts a project ID
  without checking membership.
- API keys are high-entropy, shown only once, stored only as a verifier plus a
  display prefix, and can be named, scoped, rotated, expired, and revoked.
- Every response includes `X-Request-Id`. A caller may supply a valid opaque
  `X-Request-Id`; otherwise Bowser creates one.

API-key scopes are `browsers:read`, `browsers:write`, `profiles:read`,
`profiles:write`, `tasks:read`, `tasks:write`, `events:read`, `usage:read`, and
`keys:manage`. Dashboard tokens receive permissions from membership, not from
client-provided claims.

## Asynchronous Mutations

Browser creation, commands, tasks, stop, profile persistence, and artifact
generation are asynchronous. On acceptance the API returns `202 Accepted`
with the created resource and a status URL. Creation endpoints may return
`201 Created` only when the resource is fully ready before the response.

All mutation requests require `Idempotency-Key`, except key creation and
revocation from an interactive dashboard session. Keys are scoped to project,
HTTP method, and route template, retained for 24 hours, and bound to a request
body digest. Reuse with a different body returns `409 idempotency_conflict`.

The API persists the mutation and its outbox entry in one transaction before
responding. Workers are at-least-once consumers; operation state and tool
invocation IDs make execution idempotent.

## Resource Endpoints

- **`GET /v1/workspaces`:** List dashboard-user workspaces and memberships.
- **`GET /v1/projects`:** List dashboard-user projects without requiring a
  selected project.
- **`GET /v1/projects/{id}`:** Get selected project metadata and effective
  role.
- **`POST /v1/browsers`:** Create a browser, optionally from a profile and
  egress policy.
- **`GET /v1/browsers`:** Cursor-list project browsers with status filters.
- **`GET /v1/browsers/{id}`:** Get current state and safe provider-derived
  metrics.
- **`POST /v1/browsers/{id}/stop`:** Gracefully stop, persist the profile, and
  release E2B.
- **`POST /v1/browsers/{id}/live-tokens`:** Mint a five-minute user-bound
  live-view token.
- **`POST /v1/browsers/{id}/commands`:** Queue one typed direct browser
  command.
- **`GET /v1/commands/{id}`:** Get command state, result, and safe error.
- **`POST /v1/commands/{id}/cancel`:** Cancel a queued command or request
  cooperative cancellation.
- **`POST /v1/tasks`:** Start a bounded natural-language task.
- **`GET /v1/tasks`:** Cursor-list tasks by status, browser, and time.
- **`GET /v1/tasks/{id}`:** Get task configuration, progress, result, and
  usage.
- **`POST /v1/tasks/{id}/cancel`:** Request cooperative cancellation.
- **`POST /v1/tasks/{id}/confirmations/{id}`:** Approve or reject a pending
  action.
- **`GET /v1/events`:** Query persisted project events.
- **`GET /v1/events/stream`:** Resume an SSE stream from an event cursor.
- **`GET /v1/api-requests`:** Query redacted API request metadata.
- **`GET /v1/audit-events`:** Return owner-only project security and
  administrative history.
- **`GET /v1/usage`:** Query browser, model, proxy, artifact, and request
  usage.
- **`GET /v1/artifacts/{id}`:** Get metadata and a short-lived download URL.
- **`POST /v1/profiles`:** Create project-scoped persistent browser profile
  metadata.
- **`GET /v1/profiles`:** List profiles and current lease and safe generation
  metadata.
- **`DELETE /v1/profiles/{id}`:** Stop future use and schedule encrypted
  generations for deletion.
- **`POST /v1/egress-profiles`:** Store encrypted project proxy
  configuration.
- **`GET /v1/egress-profiles`:** List only safe display and validation
  metadata.
- **`DELETE /v1/egress-profiles/{id}`:** Revoke future use without exposing
  credentials.
- **`POST /v1/api-keys`:** Create a project API key and return its secret
  once.
- **`GET /v1/api-keys`:** List key metadata, never key secrets.
- **`DELETE /v1/api-keys/{id}`:** Revoke a key immediately.

Project membership mutation endpoints follow the same conventions and are
added before team invitations are exposed in the dashboard.

## Operator Query Endpoints

The operator dashboard queries `/v1/admin/overview`, `/v1/admin/projects`,
`/v1/admin/browsers`, `/v1/admin/operations`, `/v1/admin/events`,
`/v1/admin/api-requests`, `/v1/admin/audit-events`, and `/v1/admin/usage`.
These endpoints support cursor, time, status, project, provider, request, and
resource filters appropriate to each record.

Only an Identity Platform session with a server-assigned operator role and a
recent step-up may call these routes. Project API keys are always rejected.
Every request requires an `X-Bowser-Access-Reason`, records the operator,
reason, filters, result count, and request ID in the audit log, and applies
field-level redaction before returning data.

The operator API exposes every safe persisted event, request, operation,
provider-health, usage, and lifecycle record needed to operate the service. It
never returns API-key material, proxy credentials, provider tokens, profile
contents, hidden model reasoning, or unredacted customer content. Launch admin
routes are query-only; privileged recovery actions remain explicit, audited
operator workflows rather than generic record mutation endpoints.

## Browser Creation

```json
{
  "profile_id": null,
  "idle_timeout_seconds": 300,
  "max_lifetime_seconds": 14400,
  "viewport": { "width": 1440, "height": 900 },
  "egress": { "mode": "direct" },
  "live_view": true,
  "recording": false
}
```

Supported egress modes are `direct`, `country`, and `profile`. `country`
requires an operator-configured managed egress provider. `profile` references
an encrypted project egress profile. Unsupported modes or countries fail
before any sandbox is created; Bowser never silently falls back to direct
traffic.

## Direct Commands

The `command` object is a tagged union. Launch commands include `navigate`,
`capture`, `click`, `type_text`, `clear`, `select_option`, `submit`,
`press_keys`, `scroll`, `back`, `forward`, `reload`, `page_list`, `page_new`,
`page_select`, `page_close`, `metadata`, `expand`, `screenshot`, `download`,
and `evaluate_js`.

```json
{
  "command": {
    "type": "click",
    "page_id": "pg_1",
    "capture_id": "cap_019...",
    "element_id": 12
  }
}
```

Element commands require the capture ID that produced the element ID. A stale
capture returns `409 stale_capture`; it is never applied to the current DOM by
guessing. Commands queued while the browser is busy remain ordered until their
deadline, unless the caller requests `reject_if_busy`, in which case the API
returns `409 browser_busy`.

`type_text` values are accepted as sensitive input. Responses and default logs
contain only character count and a redaction marker. Downloads and screenshots
return artifact references, not unbounded inline bytes.

## Agent Tasks

```json
{
  "goal": "Find the lowest displayed fare from London to Lisbon next Friday",
  "browser_id": null,
  "keep_browser": false,
  "action_policy": "confirm_sensitive",
  "max_steps": 40,
  "timeout_seconds": 600,
  "max_cost_microusd": 500000,
  "output_schema": null,
  "egress": { "mode": "country", "country_code": "gb" }
}
```

If `browser_id` is omitted, Bowser creates a browser owned by the task. A task
that owns its browser stops it at every terminal outcome unless `keep_browser`
is true. Follow-up tasks may reuse an idle browser but receive a new agent
runtime reconstructed from persisted messages and checkpoints. Selecting
`unrestricted` also requires the project setting that enables it; a normal
`tasks:write` key cannot bypass a project-level action policy.

## Events And Pagination

SSE messages contain `id`, `type`, `occurred_at`, `resource`, and a typed
`data` object. Clients resume with `Last-Event-ID`. The server first replays
persisted events after the cursor and then tails new events. Heartbeats contain
no product data. A slow client is disconnected with a resumable cursor rather
than buffering without a bound.

List endpoints use `limit` (default 50, maximum 200), `starting_after`, stable
descending creation order, and an object containing `data`, `has_more`, and
`next_cursor`. Time filters are inclusive `from` and exclusive `to`. Invalid
or cross-project cursors return a typed `400` error.

## Error Envelope

```json
{
  "error": {
    "type": "invalid_request",
    "code": "stale_capture",
    "message": "Capture cap_019... is no longer current.",
    "param": "command.capture_id",
    "request_id": "req_019...",
    "retryable": false
  }
}
```

Messages are safe product copy. Internal provider bodies and debug chains are
stored only in protected operator diagnostics. Retryable failures include a
bounded `Retry-After` where applicable. Authentication failures do not reveal
whether a project or key exists.

## Quotas And Contract Generation

Quotas cover concurrent active browsers, queued operations, task steps, task
cost, requests per minute, artifact bytes, and retained profiles. Admission is
transactional, project-scoped, and occurs before provider spend. `429` reports
the quota code and reset time without exposing another tenant's capacity.

The API implementation generates and checks `docs/api/openapi.yaml`. The CLI
and Expo app consume generated clients or shared generated types; handwritten
request shapes that can drift from OpenAPI are not allowed.
