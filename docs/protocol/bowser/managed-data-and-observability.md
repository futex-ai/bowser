# Managed Data, Observability, And Security

PostgreSQL is the authoritative record for product state, customer-visible
history, audit, and usage. Cloud Logging and Monitoring hold operational
telemetry; GCS holds binary artifacts. “Log everything” means every accepted
product action and provider lifecycle boundary is reconstructable without
storing secrets or hidden model reasoning.

## Database Ownership

Diesel migrations and generated schema live in a dedicated database crate.
Store-interface crates own typed persistence DTOs and errors. PostgreSQL store
crates contain only queries, row mappings, and interface translation. Business
state transitions, encryption, authorization, and provider calls live outside
store crates.

Every tenant table includes `project_id`, uses the narrowest useful indexes,
and is covered by store tests proving project predicates. Cross-project lookup
by public ID must behave as not found.

## Core Records

- **`users`, `identities`:** Bowser user and external auth subject mapping.
- **`workspaces`, `memberships`, `projects`:** Tenant hierarchy and roles.
- **`api_keys`:** Prefix, verifier, scopes, expiry, revocation, and last-used
  time.
- **`browser_profiles`, `profile_generations`:** Profile metadata, lease, and
  encrypted object generation.
- **`egress_profiles`:** Encrypted provider configuration and safe display
  metadata.
- **`browser_sessions`:** Product state, deadlines, viewport, route, and safe
  provider metrics.
- **`operations`:** Shared command and task admission, status, deadline,
  actor, and idempotency.
- **`commands`:** Typed direct command input, capture generation, and result or
  error.
- **`tasks`, `task_messages`, `task_steps`:** Goal, policy, checkpoints,
  messages, result, and limits.
- **`model_calls`, `tool_calls`:** Provider and model identity, safe payload
  metadata, timing, usage, and result.
- **`events`:** Ordered customer-visible and audit-safe lifecycle stream.
- **`artifacts`:** GCS object identity, digest, size, media type, retention,
  and state.
- **`usage_ledger`:** Immutable measured units and micro-USD cost snapshots.
- **`api_request_logs`:** Route, actor or key, status, latency, bytes, and
  request correlation.
- **`audit_events`:** Security and administrative actor, action, and target
  history.
- **`idempotency_keys`:** Request digest and original response or resource
  binding.
- **`outbox`, `work_leases`:** Reliable publication and at-least-once worker
  ownership.

Known statuses, event kinds, usage units, actor types, and operation types are
Rust enums mapped to constrained database values. They are not differentiated
with free-form strings.

## Transaction And Ordering Contract

An accepted mutation transaction writes its domain record, operation,
idempotency record, first event, and outbox row together. The outbox relay
publishes after commit. A worker acquires a renewable lease with a fencing
generation; only the current generation may write a transition or result.

Events use a project-scoped monotonically increasing sequence in addition to
their public ID. All state transitions append an event in the same transaction.
Terminal operations are immutable except for separately appended retention,
redaction, or administrative annotations.

Pub/Sub is treated as at-least-once even if an upstream exactly-once option is
enabled. Duplicate deliveries are no-ops after checking operation state and
fencing generation. A dead-letter consumer records the exhausted delivery and
transitions an otherwise stranded operation to a safe failure.

## Event Envelope

Each event records:

- public ID, project sequence, kind, severity, and schema version;
- project, request, trace, browser, operation, command, and task IDs where
  applicable;
- actor type and safe actor identifier;
- occurrence and persistence timestamps;
- a typed, size-bounded JSON payload;
- visibility: `customer`, `operator`, or `audit`.

Customer APIs return only customer-visible events. Operator detail requires an
operator role and reason. Audit events cannot be edited or deleted through the
normal API.

## Data Classification And Redaction

Data is classified as public, customer content, sensitive, secret, or
operator-only.

- Authorization headers, API-key secrets, E2B tokens, runtime capabilities,
  KMS material, model keys, proxy passwords, cookies, and passwords are secret
  and never logged.
- `type_text` input is sensitive by default. Logs store only length and a
  redaction marker. A future explicit capture policy may store encrypted text,
  but default behavior cannot change silently.
- Task goals, chat messages, page captures, URLs, titles, model-visible tool
  input/output, and downloads are customer content. They are project-isolated,
  encrypted, retained for a bounded period, and queryable only with matching
  permission.
- Query strings and URL user-info are redacted before URL persistence. Known
  secret fields are recursively redacted from structured payloads.
- Model provider replay context needed to continue a task is encrypted and
  operator-hidden. Provider reasoning content is not made customer-visible and
  is discarded when it is not required for replay.
- Provider error bodies are sanitized at their adapter boundary. Public errors
  contain a Bowser code; operator diagnostics retain only the minimum safe
  context.

Redaction tests use canary secrets and assert they are absent from database
rows, structured logs, traces, events, API responses, and artifacts.

## Encryption And Secret Storage

Cloud SQL and GCS use Google-managed encryption plus customer-managed KMS keys
where supported. Sensitive application fields use envelope encryption with a
versioned KMS key and authenticated context containing project ID, record type,
and record ID. Key rotation rewraps data keys without rewriting plaintext.

Secret Manager stores model-provider keys, the E2B key, proxy-provider master
credentials, API-key verifier pepper, and deployment secrets. Terraform creates
secret containers and IAM only; secret values must not enter Terraform state,
Helm values, images, or Git. GKE workloads use Workload Identity and receive
only the secrets required by their Kubernetes service account.

Project API keys contain a non-secret lookup prefix and 32 random bytes encoded
with unpadded base64url under the `bws_` namespace. Bowser stores the lookup
prefix and `HMAC-SHA-256(pepper-version, complete-key)` verifier, never the
complete key. Verification selects the recorded pepper version, uses a
constant-time comparison, and supports rotation without logging either value.

## Artifacts

Screenshots, recordings, downloads, profile archives, and oversized captures
live in private GCS buckets. The artifacts table stores object generation,
SHA-256 digest, byte length, media type, original safe filename, creator,
retention deadline, and scan status.

Downloads are served with short-lived signed URLs after project authorization.
Executable or unknown downloads are never rendered inline. Profile archives
are never directly downloadable. Object deletion is idempotent and leaves an
audit tombstone after bytes are removed.

## Usage And Statistics

The usage ledger records immutable quantities with source, unit, price version,
measurement state, and micro-USD cost. Initial units are browser running
milliseconds, paused storage milliseconds when billed, proxy bytes, model
tokens by category, model cost, artifact bytes, task steps, and API requests.

Dashboard statistics are SQL aggregations over this ledger and operation
records. Provider invoices or metrics are reconciled into adjustment entries;
past ledger rows are not rewritten. Unknown price or usage is marked
`unpriced`, not treated as zero.

## Query And Retention

The public event API filters by time, kind, severity, browser, command, task,
request, and trace. Usage groups by hour, day, or month and optionally by unit.
Expensive ranges are capped and served from bounded aggregate queries.

Launch defaults are:

- API request metadata: 30 days;
- event, command, task, model, and tool records: 90 days;
- screenshots, downloads, and recordings: 7 days unless shortened;
- audit events and usage ledger: 400 days;
- browser profiles: until explicit deletion or workspace deletion.

Retention workers delete object bytes before redacting or tombstoning database
content and record their progress idempotently. Legal or operator holds are
explicit records with audit history, not silent skipped deletions.

## Operational Telemetry

All services emit structured `tracing` output with request, trace, project-safe
hash, operation, and browser IDs. Cloud Logging excludes customer payloads by
default. OpenTelemetry exports request latency, queue age, worker lease age,
provider latency/error rate, active/paused sandbox counts, reconciliation
drift, database pool health, model usage, and artifact failures.

Alerts cover API availability, queue age, dead letters, stuck browsers,
orphaned E2B spend, Cloud SQL health, error-rate burn, quota exhaustion, secret
access anomalies, and cost budgets. Customer-visible event history does not
depend on Cloud Logging retention.

## Backup And Deletion

Cloud SQL production enables HA, automated backups, point-in-time recovery,
and tested restores. GCS enables lifecycle rules and object versioning only
where recovery value exceeds privacy cost. The target production recovery
objectives are RPO 15 minutes and RTO 4 hours.

Workspace deletion immediately revokes keys and stops browsers, then schedules
project data and artifacts for verified deletion. Identity deletion is kept
separate until all owned workspaces are transferred or deleted. Each phase is
resumable and visible in an operator audit trail.
