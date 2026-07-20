# API-First Managed Browser Product

Status: Active

## Outcome

Turn Bowser from a local Rust browser library and CLI into a real hosted
developer product: a versioned REST API and remote CLI for direct browser
commands and bounded natural-language browser tasks, backed by E2B, PostgreSQL,
an Expo dashboard, and reproducible GCP infrastructure.

The launch product should let a developer sign up, create a project key, start
a routed browser, issue deterministic commands, ask an agent to complete a
multi-step goal, follow progress, inspect all safe logs and usage, retrieve
artifacts, and stop the browser without using the dashboard for orchestration.

## Target Contracts

Implementation must remain aligned with:

- [Managed Product Overview](../docs/protocol/bowser/managed-product-overview.md)
- [Hosted API](../docs/protocol/bowser/hosted-api.md)
- [Managed Agent And Browser Runtime](../docs/protocol/bowser/managed-agent-and-browser.md)
- [Managed Data, Observability, And Security](../docs/protocol/bowser/managed-data-and-observability.md)
- [Managed Dashboard And CLI](../docs/protocol/bowser/managed-dashboard-and-cli.md)
- [Managed Infrastructure And Operations](../docs/protocol/bowser/managed-infrastructure.md)
- [Managed Testing And Release](../docs/protocol/bowser/managed-testing-and-release.md)

## Current Baseline

The repository currently has:

- a public `bowser` Rust crate that launches or resumes local Chrome, captures
  semantic page state, manages pages, and performs typed interactions;
- a local `bowser-cli` command surface and REPL;
- strong browser-backed integration coverage and repository checks;
- no hosted API, tenant/auth model, PostgreSQL schema, durable worker,
  dashboard, E2B adapter/template, GCP infrastructure, or remote client.

The existing engine is reused inside each E2B sandbox. It is not replaced with
Browser Use, Playwright, Kernel, or a dashboard-side browser implementation.

## Fixed Architecture Decisions

- The public API is the primary product; CLI and Expo are API clients.
- One managed browser maps to one E2B sandbox and one serialized command lane.
- The model/agent loop runs in GCP workers, not inside the browser sandbox.
- The separate Futex AI repository is a pinned Git dependency for managed
  model, tool-calling, logging, and usage contracts. The inspected development
  checkout is `/Users/calummoore/projects/futex/ai`; that path may override the
  dependency locally but cannot be the CI or release source.
- PostgreSQL is the product/audit source of truth. Pub/Sub is an at-least-once
  delivery mechanism, not authoritative state.
- GCS stores binary artifacts and encrypted browser profile generations;
  PostgreSQL stores their complete metadata and lifecycle.
- E2B sandbox-level egress proxying is preferred. Browser-level proxy flags are
  only a tested fallback. Full WireGuard/OpenVPN support is not claimed without
  proof.
- GKE Autopilot hosts the Rust control plane, workers, relay, reconciler,
  retention worker, and live proxy. Browsers remain in E2B.
- Identity Platform handles dashboard identity. Project API keys authenticate
  customer applications and the CLI.
- `bowser-503012` is development/staging only. A separate GCP project is
  required before production customer data.
- Self-serve billing and a full marketing site are deferred; usage ledger,
  quotas, budgets, and spend controls are not deferred.

## Target Workspace Shape

Exact crate names may change only with a documented boundary improvement. The
intended ownership is:

- **Existing Chrome/CDP engine:** `bowser`.
- **Pure hosted IDs, enums, state machines, and domain DTOs:**
  `bowser-domain`.
- **Public request/response and OpenAPI types:** `bowser-api-types`.
- **Store traits, persistence DTOs, and store error contract:**
  `bowser-store-interface`.
- **Diesel schema and migrations:** `bowser-db`.
- **PostgreSQL queries and row mapping only:** `bowser-store-pg`.
- **Queue, object, KMS, identity, and clock traits:**
  `bowser-platform-interface`.
- **GCP implementations of platform traits:** `bowser-platform-gcp`.
- **E2B lifecycle/runtime client trait and DTOs:**
  `bowser-sandbox-interface`.
- **Typed E2B REST/runtime adapter:** `bowser-sandbox-e2b`.
- **HTTP/noVNC process inside E2B:** `bowser-sandbox-runtime`.
- **Hosted application orchestration behind traits:** `bowser-service`.
- **AI tools, checkpoints, policies, and task runner:** `bowser-agent`.
- **HTTP client used by CLI and integration tests:** `bowser-api-client`.
- **Axum public API composition root:** `bowser-api`.
- **Worker, relay, reconciler, and retention composition roots:**
  `bowser-worker`.
- **Remote-first CLI plus explicit local namespace:** `bowser-cli`.
- **Expo application:** `ts/app`.
- **Mockup system:** `docs/mockups`.
- **E2B, Terraform, and Helm sources:** `infra/e2b`, `infra/terraform`, and
  `infra/helm`.

Each new Rust crate gets a publish-quality README in the required section order.
Impure behavior is exposed behind traits and tested with Unimock. Composition
roots construct concrete adapters. New crates use typed errors and structured
DTOs; they do not pass untyped JSON beyond forced provider/HTTP boundaries.

## Implementation Milestones

### Milestone 0: Prove External Capabilities And Lock Decisions

Summary: remove upstream and account uncertainty before building product code.
The repository remains a functioning local product throughout this milestone.

- [ ] Reauthenticate `gcloud`, explicitly select `bowser-503012`, inventory all
  existing project resources/IAM/billing, and record import-versus-create
  decisions without mutating existing infrastructure.
- [ ] Confirm `europe-west2` or replace it in the protocol with the chosen GCP
  region based on residency, latency, service availability, and team access.
- [ ] Confirm the base domain and the separate future production project plan;
  keep domain names as Terraform inputs until ownership is verified.
- [ ] Confirm E2B team, plan, region, concurrency, continuous-runtime,
  pause-storage, template-build, restricted-traffic, and support limits.
- [ ] Build a disposable E2B template spike containing Chrome, Xvfb, noVNC, and
  an authenticated test runtime; verify immutable build IDs and readiness.
- [ ] Prove restricted public runtime and noVNC access rejects unauthenticated
  traffic and supports HTTP plus WebSocket forwarding from a server client.
- [ ] Prove direct egress, E2B allow/deny rules, `network.egressProxy`, proxy
  authentication, DNS routing, browser navigation, and browser downloads using
  canary endpoints.
- [ ] Prove provider failure never leaks credentials and can be detected before
  browser readiness; document whether E2B or Chrome owns the selected fallback.
- [ ] Ask E2B for explicit TUN/WireGuard/OpenVPN capability details. Add a full
  tunnel only if the required privileges and pause/resume behavior are proven;
  otherwise ship egress proxy routing and use that product language.
- [ ] Select the managed egress vendor and country coverage, or explicitly
  launch with direct plus project-owned egress profiles only. Record pricing,
  sticky-session, protocol, DNS, credential rotation, and outage behavior.
- [ ] Prove E2B pause/auto-resume, runtime/noVNC reconnection, metrics, absolute
  Bowser lifetime enforcement, and safe kill/reconciliation.
- [ ] Pin a reviewed commit from `https://github.com/futex-ai/ai` and build a
  small integration using `ai-interface`, `ai-tool-calling`, a mock model,
  one typed tool, checkpoints, logger callbacks, and usage DTOs.
- [ ] Decide the initial server-side model route and credential owner; expose a
  product tier rather than arbitrary provider/model strings unless the
  protocol is deliberately expanded.
- [ ] Confirm launch defaults for concurrency, task steps, task cost, browser
  lifetime, artifacts, retention, and operator emergency admission stop.
- [ ] Update protocol pages and an ADR/research record with measured results,
  upstream versions, costs, and rejected alternatives.
- [ ] Build and run every spike's automated test plus a manual browser/live-view
  smoke; remove disposable resources and verify no E2B sandbox remains.

Milestone acceptance: every external dependency has a tested contract and an
owner; the plan contains no unsupported VPN, GCP, E2B, AI, or egress claim.

### Milestone 1: Establish Hosted Workspace Boundaries

Summary: add compile-tested domain/interface crates and preserve all current
local browser behavior.

- [ ] Add the hosted crates in dependency order, using `cargo add` without a
  guessed version for every crates.io dependency and reviewed Git revisions
  for the Futex AI crates.
- [ ] Add `#![warn(unreachable_pub)]`, module docs, public API docs, typed error
  contracts, and the required crate README sections to every new crate.
- [ ] Implement pure prefixed UUIDv7 IDs, timestamps, money-in-micro-USD,
  bounded pagination, request correlation, actor, role/scope, quota, and
  redaction value types in `bowser-domain`.
- [ ] Implement browser, command, task, artifact, profile, confirmation,
  idempotency, event, lease, and usage state machines with exhaustive transition
  tests and no stringly typed statuses.
- [ ] Define public `/v1` request/response/error/event DTOs and tagged command,
  egress, and task unions in `bowser-api-types`.
- [ ] Generate an initial `docs/api/openapi.yaml`, add schema snapshot and
  compatibility checks, and document async/idempotency behavior with examples.
- [ ] Define store, sandbox, queue, object, encryption, identity-verification,
  clock, and application-service traits in their owning interface crates.
- [ ] Keep interfaces at real behavior boundaries; do not create pass-through
  modules or public re-exports to avoid updating imports.
- [ ] Add configuration structs with startup validation for local test, GCP,
  E2B, limits, retention, model routes, and feature gates; secret values remain
  separate secret references.
- [ ] Add credential-free composition smoke binaries using in-memory/mocked
  adapters so the dependency graph is exercised before infrastructure exists.
- [ ] Update root, crate, developer architecture, and protocol documentation
  with real ownership and code jumping-in points.
- [ ] Run unit tests, doctests, source/trait/file-length audits, format, clippy,
  the existing browser suite, CLI smoke, and `cargo xtask check`.

Milestone acceptance: the new hosted architecture compiles and is fully tested,
OpenAPI is checked, and the existing local CLI/library remain green.

### Milestone 2: Build PostgreSQL, Audit, And Durable Work Foundations

Summary: make every hosted mutation and observable lifecycle durable before
connecting a real browser provider.

- [ ] Add the dedicated Diesel migration/schema crate, store-interface DTOs,
  and PostgreSQL query crate while keeping business logic out of `*-store-pg`.
- [ ] Create immutable timestamped migrations for users/identities, workspaces,
  memberships, projects, API keys, browser/profile/egress records, operations,
  commands, tasks/messages/steps/confirmations, model/tool calls, events,
  artifacts, usage, request logs, audit, idempotency, outbox, and work leases.
- [ ] Add foreign keys, state/check constraints, project-scoped uniqueness,
  retention indexes, cursor indexes, lease indexes, and bounded payload/length
  validation appropriate to every table.
- [ ] Implement store traits and PostgreSQL queries for transactional admission,
  compare-and-set transitions, project event sequencing, idempotency replay,
  lease fencing/renewal, outbox claiming, dead-letter transitions, retention
  batches, API-key lookup, and usage aggregation.
- [ ] Prove every tenant query includes project scope and cross-project public
  IDs return not found; add adversarial multi-tenant integration tests.
- [ ] Implement API-key generation, prefix/verifier storage, constant-time
  verification, one-time display, scopes, expiry, rotation, revocation, and
  last-used sampling without writing raw keys to logs.
- [ ] Add recursive structured redaction and canary-secret tests covering
  typed text, URLs, headers, provider errors, and task/model/tool payloads.
- [ ] Implement versioned envelope-encryption traits and an in-memory test
  adapter; persist authenticated encryption context and rotation metadata.
- [ ] Implement immutable usage entries with measurement state and price
  snapshots; unknown usage/cost must remain distinguishable from zero.
- [ ] Add an ephemeral PostgreSQL developer/test environment and commands that
  migrate from zero, migrate from the prior fixture schema, verify rollback
  assumptions, and clean up safely.
- [ ] Add service tests for duplicate delivery, worker crash boundaries, stale
  leases, event ordering, retention resumption, and outbox/database failure.
- [ ] Update database, store, security, and developer READMEs with schema and
  migration ownership.
- [ ] Run focused database/service tests and the full `cargo xtask check`.

Milestone acceptance: a provider-free end-to-end test admits work, leases it,
checkpoints simulated results, streams/query events, meters usage, and reaches
one correct terminal state under retries.

### Milestone 3: Run Bowser Reliably Inside E2B

Summary: deliver a secure, typed sandbox runtime and E2B adapter that can drive
the existing engine without any tenant/model/database secret in the sandbox.

- [ ] Add the `bowser-sandbox-runtime` binary with trait-backed lifecycle and a
  narrow authenticated HTTP/WebSocket protocol for initialize, health/version,
  commands, progress, metrics, profile import/export, and shutdown.
- [ ] Refactor engine integration only where needed so the sandbox runtime can
  launch one owned Chrome session without weakening local process/profile
  ownership checks.
- [ ] Add durable capture IDs at the worker boundary and sandbox DOM-generation
  checks so stale element commands fail before any browser side effect.
- [ ] Map every launch command union variant to typed `BrowserEngine` and
  `PageEngine` behavior, including multi-page operations, artifacts, downloads,
  bounded output, timeouts, cancellation generation, and safe errors.
- [ ] Ensure sensitive text, cookies, proxy configuration, runtime capabilities,
  and provider errors are absent from tracing and runtime responses.
- [ ] Implement the trait-backed Rust E2B REST adapter for create/get/list,
  secure/restricted traffic, runtime calls, metrics, pause/connect, timeout,
  network egress, and kill; preserve upstream errors behind Bowser errors.
- [ ] Implement E2B/runtime credential encryption and rotation in the control
  plane; never expose raw provider IDs or endpoints to callers.
- [ ] Implement direct and selected egress modes from the proven Milestone 0
  path, exit/DNS/browser-download verification, identity coherence, and
  fail-closed behavior.
- [ ] Implement optional profile restore/export through encrypted GCS object
  generations, exclusive profile leases, atomic promotion, and crash-safe use
  of the last good generation.
- [ ] Build the E2B template under `infra/e2b` with pinned Chrome/runtime,
  unprivileged processes, Xvfb, noVNC, readiness, template tags, and build-ID
  output; do not depend on private-beta E2B volumes.
- [ ] Implement the authorized live-view proxy contract and view-only default,
  including short-lived tokens and reconnect after pause/resume.
- [ ] Add controlled fixture pages and real local Chrome runtime integration
  tests for every command and failure path.
- [ ] Add opt-in E2B contract tests for secure ports, egress, DNS, pause/resume,
  noVNC, metrics, artifacts, profile generation, cleanup, and rollback.
- [ ] Add a smoke command that creates one browser, navigates/captures/clicks,
  downloads an artifact, pauses/resumes, and stops while proving provider
  cleanup.
- [ ] Update engine, runtime, E2B, egress, and template READMEs/protocol docs.
- [ ] Run template builds, runtime tests, live E2B smoke, and the full
  `cargo xtask check`.

Milestone acceptance: a developer-only harness can safely execute the complete
direct command flow in an E2B browser with verified routing, live view,
pause/resume, artifacts, and no orphaned sandbox.

### Milestone 4: Provision Reproducible GCP And Kubernetes Foundations

Summary: create a development/staging platform without mixing infrastructure
work into backend or UI milestones.

- [ ] Create/import the remote Terraform state and record the pre-change GCP
  inventory for `bowser-503012` after explicit project selection.
- [ ] Implement version-pinned Terraform modules and dev/staging roots for
  project services, VPC/private access/NAT, GKE Autopilot, Cloud SQL, Artifact
  Registry, GCS, Pub/Sub/dead-lettering, Identity Platform, Secret Manager/KMS,
  edge/DNS/certificates/Armor, IAM, budgets, logging, and monitoring.
- [ ] Keep secret payloads out of Terraform state; add an audited bootstrap
  procedure for E2B, AI, egress, key-pepper, and deployment secrets.
- [ ] Configure Workload Identity Federation and distinct least-privilege
  service identities for API, worker, relay, reconciler, retention, live proxy,
  migration, CI plan, CI deploy, and operators.
- [ ] Configure private Cloud SQL connectivity, connection budgets, automated
  backups, PITR, deletion protection, and a tested development restore.
- [ ] Add Pub/Sub subscriptions with retry/dead-letter policies and monitoring
  for backlog, lease expiry, and dead letters.
- [ ] Add private artifact/profile buckets with retention/lifecycle, CORS,
  signed-URL identities, KMS, access logging, and deletion tests.
- [ ] Implement the Helm chart with value schemas, per-workload identities,
  probes, resources, autoscaling inputs, disruption/topology policy, network
  policy, Secret Manager references, Cloud SQL connectivity, and migration job.
- [ ] Add placeholder-safe deployment values for components not yet released;
  never deploy fake product APIs or user-visible placeholder data.
- [ ] Configure Artifact Registry image scanning, immutable digest deployment,
  GitHub Actions Workload Identity Federation, Terraform plan, and manual
  staging apply/deploy workflows.
- [ ] Add monitoring dashboards and initial alerts for API health, Cloud SQL,
  Pub/Sub, GKE, budgets, and E2B orphan/spend metrics.
- [ ] Validate Terraform format/init/validate/lint/security/plan and Helm
  lint/schema/template/policy/server dry-run; review the saved plan for every
  destructive or replacement action before apply.
- [ ] Deploy a private health-check workload, verify edge TLS/routing, database,
  Pub/Sub, GCS/KMS, Secret Manager, and workload identity, then remove the
  temporary workload.
- [ ] Document bootstrap, plan/apply, rollback, access, restore, and teardown
  runbooks without recording credentials.

Milestone acceptance: staging infrastructure is reproducible from reviewed
Terraform and Helm, private dependencies are reachable only by intended
identities, and no product browser runs on GKE.

### Milestone 5: Deliver Identity, Projects, Keys, And The Public API Shell

Summary: expose a real authenticated `/v1` control plane with durable admission,
query, and event contracts before real command execution is enabled.

- [ ] Implement Identity Platform ID-token verification behind a trait, subject
  mapping, first-sign-in personal workspace/project creation, email-verification
  gating, membership roles, project selection, and disabled-user handling.
- [ ] Implement project API-key authentication/scopes, immediate revocation,
  per-key/project rate limiting, request ID/trace propagation, safe request
  metadata logging, and audit events.
- [ ] Implement Axum middleware for body limits, timeouts, CORS, security
  headers, idempotency, authorization, typed errors, panic containment, and
  redaction.
- [ ] Implement workspace/project/key/profile/egress metadata endpoints required
  by launch, with cursor pagination and consistent cross-project not-found
  behavior.
- [ ] Implement browser, command, task, confirmation, artifact, event, usage,
  and API-request read endpoints against real PostgreSQL state.
- [ ] Implement operator-only overview, project, browser, operation, event,
  request, audit, and usage queries with step-up authentication, required
  access reasons, server-side filtering, and an audit record for every query.
- [ ] Implement mutation admission endpoints that atomically write domain,
  idempotency, event, and outbox records and return `202` without performing
  provider work on API request threads.
- [ ] Implement resumable SSE with database replay, bounded live tail,
  heartbeat, slow-client disconnect, auth recheck, and cursor isolation.
- [ ] Implement Cloud Armor/application quota mapping and the emergency mode
  that rejects new spend while retaining reads, stop, cancel, and cleanup.
- [ ] Generate and diff-check OpenAPI from the implementation; generate the
  Rust API client and TypeScript dashboard client in reproducible commands.
- [ ] Add contract tests for every endpoint, role/scope, project isolation,
  idempotency replay/conflict, quota, pagination/cursor, event resume, safe
  errors, and canary-secret absence.
- [ ] Build the API image, run migrations, deploy it to staging, and run
  authentication, key, mutation-admission, event, and query smoke paths.
- [ ] Update API, auth, client, and operator READMEs plus public curl examples.
- [ ] Run API/store tests, image build, staging smoke, and the full
  `cargo xtask check`.

Milestone acceptance: a verified user can create and revoke a real project key,
admit/query a simulated operation, resume its events, and view real zero/empty
usage without any direct database or provider access.

### Milestone 6: Ship Direct Managed Browser Commands

Summary: connect durable work admission to real E2B execution and make the
hosted deterministic browser API usable end to end.

- [ ] Implement the outbox relay with transactional claim, retry, lag metrics,
  publish confirmation, and crash recovery.
- [ ] Implement the worker pull loop, renewable fenced leases, bounded
  concurrency, per-browser serialization, acknowledgement extension, typed
  retry/dead-letter behavior, and graceful shutdown.
- [ ] Implement browser create, pause/resume reconciliation, stop, expiry, and
  provider cleanup operations using the E2B adapter and absolute deadlines.
- [ ] Implement every direct browser command, capture-generation validation,
  operation events, artifacts, downloads, cancellation, deadlines, and terminal
  result/error mapping.
- [ ] Implement egress-profile CRUD/validation and managed country routing from
  the selected provider without returning or logging credentials.
- [ ] Implement profile create/delete, exclusive lease, restore, safe final
  generation, failure recovery, and profile retention.
- [ ] Implement GCS artifact finalization, digest verification, safe filenames,
  scan/status, signed download URLs, and sandbox temporary-file cleanup.
- [ ] Implement browser/provider/model-free usage metering for runtime, paused
  state, proxy bytes, artifact bytes, commands, and request counts.
- [ ] Implement live-view authorization/proxying, reconnect, view-only default,
  optional interaction permission, and access audit.
- [ ] Implement the E2B reconciler and retention worker with dry-run modes,
  environment ownership checks, orphan cleanup, drift repair, and alerts.
- [ ] Add integration tests for duplicate work messages, worker crash after
  side effect, provider loss, proxy outage, stale capture, profile failure,
  artifact failure, cancel/stop races, expiry, and orphan discovery.
- [ ] Deploy worker/relay/reconciler/retention/live proxy to staging and run the
  full direct API launch smoke including logs, usage, pause/resume, and cleanup.
- [ ] Update browser/runtime/API/operations/runbook documentation with measured
  limits and real examples.
- [ ] Run all direct-command, E2B, API, staging, and `cargo xtask check` gates.

Milestone acceptance: an API-only customer can create a routed browser, run the
full typed command surface, stream/query every safe lifecycle event, retrieve
artifacts, inspect real usage, and stop with verified E2B cleanup.

### Milestone 7: Add Durable Natural-Language Browser Tasks

Summary: integrate the Futex AI runtime as a bounded, reconstructable agent
that uses exactly the same managed browser command path.

- [ ] Add the reviewed Futex AI Git dependencies at pinned revisions and record
  a repeatable update/review process; CI must not depend on the local path.
- [ ] Compose the configured `ai-interface::Model` route with provider adapters,
  retry/concurrency/pricing policy, structured output, and no provider secrets
  in E2B.
- [ ] Implement the small typed `observe`, `navigate`, `interact`, `pages`,
  `artifact`, and `finish` tool catalog over the browser application service.
- [ ] Make each `ToolInvocation.operation_id` drive idempotent command creation
  so replay after a worker crash cannot duplicate a browser side effect.
- [ ] Implement the Bowser AI `Logger` adapter for model calls, tool calls,
  public activity, usage, errors, and terminal outcomes through store traits.
- [ ] Persist validated model responses before tools, each tool result before
  the next round, provider replay context encrypted, and bounded observable
  step summaries without hidden reasoning.
- [ ] Reconstruct `ToolCallingRuntime` from persisted messages/checkpoints on
  every lease and compact safely after context-limit errors without losing tool
  or provider replay integrity.
- [ ] Implement task-owned versus reused browsers, follow-up tasks, cleanup on
  every terminal outcome, and browser-busy ordering with direct commands.
- [ ] Implement `read_only`, `confirm_sensitive`, and `unrestricted` action
  policies, typed confirmation requests, approval/rejection, expiry, and an
  enforced non-unrestricted playground default.
- [ ] Enforce max steps, wall time, model micro-USD cost, browser lifetime,
  output size, artifact bytes, cancellation, and structured-output validation
  at durable checkpoints.
- [ ] Add model token/cost usage entries with price version and measurement
  state and reconcile provider-reported usage without rewriting history.
- [ ] Add deterministic mocked-model tests for every tool and outcome, including
  stale recapture, multi-tab fixture prices, confirmation, cancellation,
  budget/time/step limits, provider refusal/truncation/context limit, tool
  failure, browser loss, and worker crashes at both checkpoint boundaries.
- [ ] Add opt-in staging-model tests for one-step and multi-step controlled
  fixture goals plus schema-valid results; keep real travel-site smoke optional
  and read-only.
- [ ] Deploy the agent worker configuration to staging and run API-only create,
  follow, query, reuse, cancel, confirm, usage, and cleanup flows.
- [ ] Update agent, AI dependency, policy, API, and public usage docs with real
  limits and safe examples.
- [ ] Run agent/model contract tests, staging smoke, and full
  `cargo xtask check`.

Milestone acceptance: an API caller can submit the fixture price goal, watch a
bounded agent use multiple browser steps, receive a validated result, inspect
model/tool/browser logs and cost, and prove no duplicate action after recovery.

### Milestone 8: Make The CLI Remote-First

Summary: deliver the primary human developer interface without coupling it to
E2B, PostgreSQL, or internal service crates.

- [ ] Move the current local command surface behind `bowser local ...` while
  preserving local library capabilities and documenting the intentional
  breaking CLI migration.
- [ ] Implement `auth set-key`, project selection, browser lifecycle/live,
  direct commands, `run`, task list/get/cancel/follow, logs list/follow, usage,
  profiles, and artifact download through `bowser-api-client` only.
- [ ] Require capture ID for element actions and make stale-capture recovery
  explicit; never guess an element against a newer page state.
- [ ] Implement config precedence, `BOWSER_API_KEY`/base URL/project variables,
  owner-only secret file permissions, and prohibit API-key flags.
- [ ] Implement idempotency-key generation/persistence across safe retries,
  bounded backoff, `Retry-After`, request IDs, SSE cursor resume, reconnect, and
  cooperative interrupt/cancel behavior.
- [ ] Implement human output, stable `--json`, quiet/verbose behavior, stderr
  progress, output-schema file validation, artifact output paths, and specified
  exit codes.
- [ ] Ensure goal text, typed secrets, API keys, proxy data, and tokens do not
  appear in diagnostics or process arguments beyond intentional goal/command
  arguments documented to the user.
- [ ] Add parser/unit tests, mocked HTTP/SSE tests, snapshot tests, credential
  permission tests, retry/idempotency tests, and request/response compatibility
  tests generated from OpenAPI.
- [ ] Add deployed staging CLI tests for key setup, routed browser, direct
  capture/action, followed task, SSE reconnect, structured result, logs, usage,
  artifact, cancel, stop, and cleanup.
- [ ] Update CLI README, shell completion, `--help`, migration guide, install
  instructions, and API quick start.
- [ ] Build release binaries and archives for supported platforms, run manual
  CLI smoke paths, and run the full `cargo xtask check`.

Milestone acceptance: a developer can use every launch capability from a clean
terminal with only a Bowser API key, and machine-readable output is stable.

### Milestone 9: Specify The Expo Experience In Mockups

Summary: complete design/spec work before dashboard implementation, with
separate mobile and desktop components and no fake product data.

- [ ] Scaffold the React-backed mockup system and shared dark editorial design
  tokens/components under `docs/mockups/src` before creating page markup.
- [ ] Create standalone mobile and desktop screen components for home hero,
  login, signup, password reset, dashboard overview, API keys, browsers,
  browser detail/live view, tasks, task detail, playground chat, logs, usage,
  profiles, and settings.
- [ ] Create separate operator-admin mobile and desktop screens for service
  health, projects, browsers, operations, logs, audit, and usage; keep them out
  of project navigation and all non-operator bundles/routes where practical.
- [ ] Include loading, empty, error, permission, quota, reconnecting,
  confirmation, terminal success, and terminal failure variants using product
  language and real-shape example labels only in mockup data.
- [ ] Keep the public home to logo/nav/hero/actions and hide marketing routes
  from native flow/navigation.
- [ ] Define the project switcher, responsive navigation, accessible tables and
  filter drawers, status language, cost measurement states, live-view controls,
  copy-once key flow, and destructive confirmations.
- [ ] Create user flows for onboarding to first key, first direct browser
  command, first agent playground task, sensitive-action confirmation, logs
  correlation, and browser stop/profile persistence using only standalone
  screen components with backlinks.
- [ ] Keep implementation notes outside rendered screens and ensure user copy
  contains no E2B, schema-pipeline, environment, or internal code terms unless
  shown in a secondary developer detail where genuinely useful.
- [ ] Generate and commit matching HTML with `npm run mockups:build`.
- [ ] Run `npm run mockups:check`, `npm run mockups:test`, and
  `npm run mockups:typecheck` with 100% pass rate.
- [ ] Open every changed generated page directly from disk at mobile and
  desktop sizes, verify visual/layout/accessibility behavior, and record the
  manual smoke result.
- [ ] Review all screens against hosted API fields and add any discovered
  contract gaps to the protocol and this plan before implementation.

Milestone acceptance: every launch route and state has an approved reusable
mobile/desktop mockup backed by the real API contract.

### Milestone 10: Build The Expo Home, Dashboard, And Playground

Summary: implement the approved mockups as a web-first Expo product client,
without adding alternate backend logic.

- [ ] Scaffold `ts/app` with Expo Router, strict TypeScript, web/native route
  groups, production builds, lint/typecheck/test commands, and per-package
  README/developer docs.
- [ ] Inspect and establish reusable base components in
  `ts/app/src/components` for typography, controls, forms, tables/lists,
  navigation, cards, status, charts, filters, code/key fields, empty/error
  states, timelines, chat, confirmation, and live view.
- [ ] Implement the dark editorial public home exactly to mockup scope and keep
  it out of native navigation.
- [ ] Implement Identity Platform email/password and Google login/signup,
  verification, reset, sign-out, refresh/expiry, account linking, route guards,
  and email-enumeration-safe errors.
- [ ] Implement project selection and generated API client wiring with cache
  keys scoped to project; clear cache and SSE when project changes.
- [ ] Implement dashboard overview, API keys, browser list/detail/live, task
  list/detail, logs, usage, profiles, and settings with real API data only.
- [ ] Implement the step-up-protected operator admin views against only the
  operator query API, with an access-reason prompt and visible audit context.
- [ ] Implement playground task composition, existing/new browser choice,
  egress selection, advanced output schema, SSE messages/steps, live view,
  confirmation, cancellation, final result, usage, and browser stop distinction.
- [ ] Implement all loading, empty, error, permission, stale/reconnecting,
  quota, unknown-cost, and terminal states from the mockups.
- [ ] Ensure operator-only payloads, E2B IDs/tokens, hidden reasoning, raw model
  payloads, environment badges, and fake data never enter product rendering.
- [ ] Add responsive mobile/desktop layout, keyboard navigation, focus
  management, reduced motion, screen-reader labels, contrast, and exact-time
  accessibility.
- [ ] Add unit/component tests, route/auth tests, generated-client compatibility,
  mocked-SSE reconnect tests, visual regression, accessibility tests, and
  browser end-to-end tests against a controlled API environment.
- [ ] Run production Expo web build and serve it locally; manually smoke every
  changed route and the onboarding/direct/task/log/usage flows at mobile and
  desktop sizes.
- [ ] Add Terraform/CI static web deployment and edge routing without exposing
  source maps or runtime secrets.
- [ ] Update app, component, auth, deployment, and user-facing documentation.
- [ ] Run all TypeScript/app/mockup checks plus the full `cargo xtask check`.

Milestone acceptance: a new user can sign up, verify, obtain a real key, inspect
real stats and active browsers, run the chat demo, view logs/usage, and clean up
from both desktop and mobile layouts.

### Milestone 11: Harden, Document, And Launch

Summary: prove tenant safety, recovery, cost control, documentation, and exact
artifact promotion before calling Bowser a real product.

- [ ] Add and pass API admission, SSE, Pub/Sub backlog, per-browser ordering,
  agent, live proxy, artifact, and database connection-budget load tests.
- [ ] Run adversarial tenant isolation, authorization/scope, SSRF boundary,
  artifact filename/content, live-view token, API-key, redaction/canary, KMS,
  Secret Manager, provider error, and dependency/image security tests.
- [ ] Prove Cloud SQL failover/reconnect, backup/PITR restore to an isolated
  environment, dead-letter recovery, worker termination, provider outage,
  emergency admission stop, E2B orphan cleanup, and retention resumption.
- [ ] Prove image digest and E2B template build-ID promotion, previous-version
  compatibility, application rollback, and E2B template rollback in staging.
- [ ] Measure and publish internal SLO dashboards for API availability/latency,
  browser readiness, task/command success, queue age, and stranded operations;
  configure burn-rate and spend alerts.
- [ ] Reconcile measured E2B/model/egress/GCP usage against the immutable usage
  ledger and set project/global hard spend limits before external access.
- [ ] Finish public API docs, OpenAPI examples, CLI quick start, dashboard help,
  architecture/security/data-retention docs, status/support paths, and operator
  runbooks; update every affected README and protocol page.
- [ ] Confirm the home page remains hero-only and that billing, scheduling,
  connectors, and full marketing remain explicitly deferred rather than
  partially exposed.
- [ ] Run the complete controlled launch smoke: signup, verification, key,
  routed direct browser, commands, fixture-price agent, live progress, logs,
  usage, artifacts, profile, stop, provider cleanup, and key revocation.
- [ ] Run optional read-only real-site smoke only after terms/access review; do
  not treat CAPTCHA bypass or third-party layout as a launch guarantee.
- [ ] Run all Rust, TypeScript, mockup, Terraform, Helm, image, migration,
  OpenAPI compatibility, E2B contract, staging end-to-end, security, load,
  restore, and rollback checks with 100% required-test pass rate.
- [ ] Run `cargo xtask check` as the final repository check and resolve every
  compile, lint, test, smoke, or documentation failure before proceeding.
- [ ] Mark every completed TODO and milestone in this plan, move this plan from
  Active to Completed in `plans/README.md`, and review the complete diff.
- [ ] Run `git add -A`, verify all new files are tracked, commit the completed
  work with an under-50-character Conventional Commit title and descriptive
  body, and push the current branch.
- [ ] After the push, run `cargo xtask review` against `origin/main`; do not
  automatically fix findings, and report each numbered finding with severity,
  context, impact, lettered solution options, and a recommended option for the
  user's decision.

Milestone acceptance: all release gates in the managed testing protocol pass,
the deployed artifact identities are recorded and reversible, no required work
remains, and the user has the review findings needed for the final launch
decision.

## Explicitly Deferred Work

- full marketing pages beyond the single hero;
- Stripe checkout, subscription lifecycle, invoices, and automatic plan
  changes (usage and quotas are already billing-ready);
- scheduled/recurring tasks;
- app/email/connector marketplaces;
- browser extensions and customer-hosted runners;
- arbitrary user code execution inside Bowser sandboxes;
- guaranteed CAPTCHA, anti-bot, anonymity, or full VPN-tunnel claims;
- native App Store/Play Store distribution beyond keeping Expo screens and
  navigation compatible.

Deferred work must not appear as reachable placeholders. It receives a new
protocol update and separate plan before implementation.
