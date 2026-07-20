# Managed Testing And Release

The hosted product is complete only when deterministic command, agent,
persistence, API, CLI, UI, E2B, egress, security, and infrastructure paths are
tested together. External-site behavior is never the sole CI oracle.

## Test Layers

### Pure And Unit Tests

- domain state transitions, limits, IDs, cursors, error mapping, and redaction;
- API DTO and OpenAPI schema round trips;
- action-policy classification and confirmation state;
- usage pricing snapshots and unknown-measurement behavior;
- agent prompt/tool schemas, task budgets, cancellation, reconstruction, and
  structured output with mocked AI traits;
- E2B, object storage, KMS, Pub/Sub, clock, and auth boundaries with Unimock;
- Expo components, route guards, data states, accessibility, and formatters;
- CLI parsing, config precedence, output, retry, SSE resume, and exit codes.

Unit tests perform no real disk, database, subprocess, browser, network, cloud,
or clock work at trait boundaries.

### Store And Service Integration Tests

Ephemeral PostgreSQL tests apply migrations from zero and verify every query,
tenant predicate, unique/idempotency constraint, lease fence, outbox
transaction, cursor, retention batch, and aggregate. Upgrade tests start from
the prior released schema and apply only forward migrations.

Service integration tests use real stores with fake providers to cover API
admission through worker completion, duplicate Pub/Sub delivery, cancellation,
timeouts, dead letters, provider loss, profile generation failure, artifact
upload failure, and reconciliation.

### Browser Runtime Tests

The existing local Chrome suite remains mandatory. New sandbox-runtime tests
exercise its HTTP contract against real Chrome and controlled fixture sites:

- initialize, health/version, command serialization, and capability auth;
- navigation, capture generation, stale element rejection, all direct actions,
  multiple pages, screenshots, and downloads;
- browser/runtime restart and bounded shutdown;
- noVNC connection and view-only behavior;
- sensitive input absence from runtime logs.

### Live E2B Contract Tests

Credentialed staging tests build or select an immutable template and verify:

- secure/restricted public traffic rejects missing credentials;
- runtime and noVNC readiness from the snapshotted template;
- direct outbound traffic plus allow/deny enforcement;
- authenticated egress proxy routing, DNS behavior, expected exit identity,
  browser downloads, and no fallback after proxy failure;
- pause, auto-resume, WebSocket reconnect, absolute lifetime enforcement, and
  stop/kill cleanup;
- provider metrics collection and orphan reconciliation;
- template rollback to a prior build ID.

WireGuard/OpenVPN is not accepted as supported merely because package
installation succeeds. A full-tunnel claim requires a documented or confirmed
E2B capability and tests for TUN creation, route/DNS coverage, leak failure,
pause/resume, and cleanup.

## Controlled Web Fixtures

CI fixture sites cover dynamic content, frames, authentication cookies, forms,
downloads, popups, infinite scroll, modal overlays, deliberate slow/failure
paths, sensitive inputs, and a synthetic travel-price flow. Assertions target
fixture-owned semantic outcomes rather than brittle coordinates.

An opt-in smoke may run a read-only price search against a real public travel
site, subject to its terms and robots/access policy. A third-party layout,
CAPTCHA, or availability change cannot fail the required CI suite.

## Agent Acceptance Scenarios

With deterministic mock models and then one configured staging model, test:

- a one-step natural-language navigation;
- a multi-step fixture price comparison returning schema-valid JSON;
- page recapture after an element becomes stale;
- multi-tab research and result synthesis;
- task follow-up on the same browser with reconstructed persisted context;
- sensitive submit paused for approval, rejection, and approval paths;
- max-step, timeout, cost-budget, cancellation, provider refusal, context-limit,
  tool failure, and browser-loss outcomes;
- worker crash after model checkpoint and after tool side effect without
  duplicated browser action.

Success requires an explicit final result and correct terminal state, not just
an assistant statement claiming completion.

## API And CLI Contract Tests

The OpenAPI document is generated and diff-checked. Every launch endpoint has
tests for authentication, role/scope, project isolation, validation,
idempotency, rate/quota behavior, pagination, redaction, and error envelope.
An API compatibility check rejects accidental breaking changes under `/v1`.

CLI end-to-end tests run against the deployed staging API and cover key setup,
browser creation, a direct command sequence, a followed agent task, event
resume after disconnect, structured JSON output, cancellation, logs, usage,
and cleanup. Tests assert that secrets never appear in arguments or output.

## Dashboard And Mockup Verification

Design implementation starts with the repository mockup workflow. Every
changed page passes build, structure, test, typecheck, and direct-file visual
smoke checks for mobile and desktop.

The Expo app passes lint, typecheck, unit/component tests, production web
build, route tests, and browser end-to-end tests. Visual regression covers the
home hero, auth, empty/loading/error states, populated dashboard, playground,
active browser, logs, and usage at mobile and desktop breakpoints. Automated
accessibility tests and keyboard-only smoke tests cover all launch routes.

## Infrastructure Verification

- Terraform formatting, validation, provider lock consistency, lint, security
  policy, plan review, and import/no-recreate checks;
- Helm lint, values-schema validation, template snapshots, Kubernetes policy,
  and server-side dry run;
- migration apply-from-zero and prior-version upgrade tests;
- image, dependency, and secret scanning;
- staging restore, failover/reconnect, dead-letter, emergency admission stop,
  and rollback exercises;
- load tests for API admission, SSE fan-out, worker backlog, per-browser
  serialization, and database connection budgets.

## Release Gate

A release candidate must satisfy all of the following:

1. protocol docs, OpenAPI, crate/app READMEs, mockups, CLI help, and behavior
   agree;
2. `cargo xtask check` and all TypeScript, mockup, infrastructure, and live
   staging checks pass with no skipped launch requirement;
3. the exact service image digests and E2B template build ID are recorded;
4. schema migration, rollback-compatible app deployment, backup restore, and
   E2B rollback are proven;
5. security/redaction and tenant-isolation tests pass;
6. monitoring, SLO dashboards, alerts, quotas, budgets, retention, and runbooks
   are active;
7. the completed diff is committed with Conventional Commits and pushed;
8. `cargo xtask review` runs after the push, and every finding is reported for
   user decision rather than silently fixed.

Launch smoke creates a key, starts a routed browser, performs direct commands,
runs the fixture price agent, watches live progress, queries logs/usage, stops
the browser, verifies provider cleanup, and revokes the key.
