# Managed Product Overview

This page defines the target contract for turning the existing local Bowser
library and CLI into a hosted, API-first browser automation product. It is a
future-state contract: the local browser protocol remains authoritative until
the managed milestones in the active plan are implemented.

## Product Goal

Bowser gives developers one hosted surface for two kinds of work:

- deterministic browser commands such as navigate, capture, click, type, and
  download;
- natural-language tasks in which an agent chooses and executes several
  browser commands to reach a goal.

The REST API is the primary product. The CLI is a first-party API client. The
Expo dashboard is for onboarding, API-key management, usage and activity
inspection, active-browser control, and a chat playground; it is not a
separate browser implementation.

## Launch Scope

The first launch includes:

- email/password and Google sign-in;
- personal workspaces, projects, project API keys, and project roles;
- managed E2B browser sessions with direct or configured egress routing;
- direct browser commands and bounded natural-language tasks;
- structured results, server-sent progress events, screenshots, downloads,
  and a protected live browser view;
- queryable request, command, task, model, tool, browser, audit, and usage
  records;
- an OpenAPI document, remote CLI, and generated dashboard client;
- Terraform-managed GCP resources and Helm-managed Kubernetes workloads.

Self-serve payment collection, a full marketing site, scheduling, connector
marketplaces, browser extensions, and arbitrary customer code execution are
not launch requirements. Usage metering, quotas, and cost limits are launch
requirements even before billing is enabled.

## Product Boundaries

The existing `bowser` crate remains the browser engine. It owns Chrome, CDP,
page capture, element interaction, and local process safety. It must not learn
about tenants, API keys, E2B credentials, PostgreSQL, or GCP.

The managed product adds these boundaries:

```text
Expo / CLI / customer app
           |
       REST + SSE
           |
   API control plane ---- PostgreSQL ---- query/admin surfaces
           |                    |
      durable work          event outbox
           |                    |
        workers <---------- Pub/Sub
        /     \
 AI runtime   E2B provider
                  |
        private sandbox runtime
                  |
          bowser crate + Chrome
```

The agent and all provider credentials run in the GCP worker. The E2B sandbox
receives browser commands and returns observations, but receives no database,
GCP, customer API-key, or model-provider credentials.

## Tenancy

The hierarchy is `user -> workspace -> project -> resource`.

- Every user receives a personal workspace during first sign-in.
- A workspace may contain several projects and memberships.
- API keys belong to one project and never cross project boundaries.
- Browser sessions, profiles, tasks, commands, events, artifacts, and usage
  entries all carry a non-null project ID.
- Project roles are `owner`, `developer`, and `viewer`. Viewers cannot create
  keys, mutate browsers, or start tasks.
- Platform-operator access is separate from project membership, requires a
  reason, and creates an audit event.

Externally visible IDs are opaque, prefixed IDs backed by UUIDv7 values. The
initial prefixes are `wrk_`, `prj_`, `key_`, `req_`, `op_`, `brw_`, `prf_`,
`cap_`, `cmd_`, `tsk_`, `cnf_`, `evt_`, and `art_`. Database sequence values
and E2B IDs are never public IDs.

## Resource State Machines

- **Browser:** `creating -> active <-> paused -> stopping -> stopped`;
  `failed` and `expired` are terminal.
- **Command:** `queued -> running`, followed by `succeeded`, `failed`,
  `cancelled`, or `timed_out`.
- **Task:** `queued -> running`, optionally
  `awaiting_confirmation -> running`, followed by `succeeded`, `failed`,
  `cancelled`, `timed_out`, or `budget_exceeded`.
- **Artifact:** `pending`, followed by `available`, `failed`, `expired`, or
  `deleted`.

State transitions are compare-and-set operations persisted before an event is
published. Provider state is evidence, not the product source of truth. A
reconciler repairs drift between Bowser and E2B.

## Cross-Cutting Invariants

- Mutations accept an idempotency key and never repeat a completed side effect
  for the same project, route, and key.
- At most one mutating command or agent step executes against a browser at a
  time. Read-only status and event queries may run concurrently.
- Every accepted mutation has a request ID, operation row, ordered event
  history, actor, timestamps, and terminal result or terminal error.
- Binary artifacts live in object storage; PostgreSQL stores their metadata,
  digest, size, ownership, retention, and access audit.
- Secrets, raw authorization headers, proxy passwords, typed password values,
  and hidden model reasoning are never written to logs.
- No public response exposes E2B sandbox IDs, access tokens, traffic tokens,
  proxy credentials, internal hosts, or provider error bodies.
- API, CLI, and dashboard use the same service contracts and authorization
  rules. The dashboard does not call E2B or PostgreSQL directly.
- Product screens render real API data or explicit loading, empty, and error
  states. Product code contains no demonstration statistics or fake activity.

## Implementation Ownership

The target Rust workspace separates domain DTOs, store interfaces, PostgreSQL
queries, E2B interfaces and adapters, agent orchestration, API transport,
workers, the sandbox runtime, and the API client into distinct crates. Impure
services and providers are trait-backed and composed only in binaries.

The Expo application lives under `ts/app`. E2B template sources live under
`infra/e2b`, Terraform under `infra/terraform`, and the Helm chart under
`infra/helm`.

## Validated Product References

The scope intentionally follows the useful split in Browser Use between
[agent sessions and raw browser sessions](https://docs.browser-use.com/cloud/quickstart),
including live views, profiles, structured output, and location-aware proxies,
without copying its API.

The managed execution design relies on E2B's documented
[custom templates](https://e2b.dev/docs/template/quickstart),
[secured access](https://e2b.dev/docs/sandbox/secured-access), and
[pause/resume lifecycle](https://e2b.dev/docs/sandbox/persistence). Upstream
capabilities are revalidated by live contract tests before release.
