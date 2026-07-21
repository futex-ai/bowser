# Managed Dashboard And CLI

The Expo dashboard and CLI are first-party clients of the hosted API. They do
not contain alternate orchestration, authorization, browser, or usage logic.

## Expo Application

The application lives at `ts/app` and uses Expo Router for web and future
native builds. The initial product is web-first, but every product screen and
mockup has mobile and desktop variants. Marketing routes are excluded from
native navigation.

Before implementing a screen, contributors must inspect and reuse
`ts/app/src/components`. Repeated patterns become base components. One screen
equals one screen component; flows compose those components rather than
inlining copies.

## Public Home And Authentication

The home page is a dark, editorial hero only:

- Bowser wordmark/logo at the left of a restrained navigation bar;
- links for Docs and GitHub;
- Sign in and Get started actions;
- one outcome-led headline, short supporting sentence, primary Get API key
  action, and secondary View docs action;
- no feature grid, testimonials, pricing table, fake terminal output, customer
  logos, or invented product metrics.

`/login` and `/signup` use Identity Platform through the Firebase client SDK.
Launch methods are email/password and Google. Sign-up requires email
verification before API-key creation. Password reset, sign-out, expired-token,
and account-linking errors have complete user flows. Email enumeration
protection is enabled.

## Dashboard Routes

- **`/dashboard`:** Real usage summary, active browsers, recent tasks, and
  recent errors.
- **`/dashboard/playground`:** Chat playground plus live browser and step
  activity.
- **`/dashboard/api-keys`:** Create, copy-once, scope, rotate, and revoke keys.
- **`/dashboard/browsers`:** Filter active, paused, stopped, failed, and
  expired browsers.
- **`/dashboard/browsers/[id]`:** State, pages, live view, route, metrics,
  commands, events, and stop controls.
- **`/dashboard/tasks`:** Filter tasks by status, browser, and time.
- **`/dashboard/tasks/[id]`:** Goal, messages, observable steps, result, usage,
  events, and cancellation.
- **`/dashboard/logs`:** Filter request and product events with correlated
  detail.
- **`/dashboard/usage`:** Browser, proxy, model, artifact, task, and API usage
  over time.
- **`/dashboard/profiles`:** Create profiles, inspect their lease and status,
  and delete them.
- **`/dashboard/settings`:** Project, member, retention, and default task
  settings.

The project switcher is always visible on desktop and available from the
mobile menu. Every request is scoped to the selected project; changing project
clears cached project data and event subscriptions.

## Dashboard Data States

Every collection and detail screen implements loading, empty, error,
permission-denied, and stale/reconnecting states. Empty states explain the
next real action. The application must not show placeholder counts, dummy
rows, simulated events, or sandbox/test badges in reachable product routes.

Dates use the viewer's locale with an accessible exact UTC value. Costs use
ledger measurement state and never render unknown cost as `$0.00`. Status
colors always have text/icon equivalents.

## Playground

The playground starts with an empty conversation and a task composer. A user
can choose an existing idle browser or let Bowser create one, select direct or
available country egress, set structured-output JSON Schema in an advanced
panel, and submit a goal.

The task view renders user messages, concise assistant results, observable
action summaries, tool status, screenshots when requested, cost/step limits,
and confirmation cards. It does not render hidden reasoning, raw model payloads,
provider names in headline copy, or internal protocol terms.

The live view uses a short-lived authorized Bowser URL. Losing the live stream
does not fail the task. Cancel and stop are distinct: cancel stops task
orchestration; stop closes the browser after warning about pending profile
persistence.

## Browser, Logs, And Usage Views

Browser details show product ID, status, creation/expiry time, current URL and
page, viewport, egress country/exit identity when safe, task/command ownership,
resource metrics, and ordered events. E2B identifiers and tokens never appear.

Logs have a time range, severity, event kind, request ID, trace ID, browser,
command, and task filters. Detail drawers show redacted structured data and
correlation links. Platform-operator diagnostics are not exposed in project
logs.

Usage charts are derived from the usage endpoint. The summary includes browser
running time, task and command success rates, model usage/cost, proxy bytes,
and artifact storage. Charts must state their range and aggregation interval.

## Operator Admin

An internal Expo web route group provides `/admin`, `/admin/projects`,
`/admin/browsers`, `/admin/operations`, `/admin/logs`, `/admin/audit`, and
`/admin/usage`. It shows service health, queue and provider state, tenant-safe
resource metadata, correlated failures, security access history, and measured
cost. Every safe record stored for operation is filterable and linked through
request and resource IDs.

The route group is absent from project and native navigation. It requires a
server-assigned operator role, recent step-up authentication, and a reason for
each query session. Non-operators receive the same not-found treatment as an
unknown route. API-key material, proxy credentials, provider tokens, profile
contents, hidden reasoning, and unredacted customer content never render.
Operator queries themselves appear in the audit view.

## Mockups And Mokabook

Mockups are Bowser's visual product specification. **Mokabook** is the required
local browser and comparison tool for those mockups; it consumes the same
registry, renderer, manifest, and committed HTML rather than becoming a second
source of truth.

The setup uses a root `package.json` that forwards mockup commands into the
TypeScript npm workspace. `ts/mockups` owns the private Mokabook package and
dependencies. Canonical source lives under `docs/mockups/src`: reusable visual
components under `components`, structured definitions under
`entries/**/*.mockup.tsx`, and the Mokabook shell under `mokabook`. Generated
mobile and desktop fragments plus `docs/mockups/mokabook-manifest.json` remain
committed and directly openable from disk.

The registry provides `defineScreen`, `defineCollection`, and `defineUseCase`:

- a screen has one stable ID, one mobile render, and one desktop render;
- a collection is navigation structure and does not duplicate a screen;
- a use case references existing screen IDs and links every step back to its
  standalone screen.

Mokabook Browse mode provides searchable nested navigation, durable screen and
use-case links, viewport controls, and a details panel with description,
rationale, source, related protocol pages, and dependencies. Review mode
compares committed base and working-tree fragments per viewport against
`origin/main`, supports side-by-side, overlay, and difference views, and writes
a self-contained `.context/mokabook-review` artifact for local or CI review.

The required root commands are:

```bash
npm run mockups:build
npm run mockups:check
npm run mockups:serve
npm run mockups:review -- --base origin/main
npm run mockups:test
npm run mockups:test:browser
npm run mockups:typecheck
```

`mockups:check` rejects stale generated output, duplicate IDs or routes,
missing mobile/desktop renders, broken links, orphan entries, and use cases
that reference missing screens. Changed screens must pass every terminating
gate, produce a Review artifact, be browsed through `mockups:serve`, and be
opened directly from disk before Expo implementation begins.

## Remote CLI

The `bowser` executable becomes remote-first while retaining the current local
engine under an explicit `bowser local ...` namespace for debugging and
embedding compatibility.

```text
bowser auth set-key
bowser projects list
bowser browsers create|list|get|stop|live
bowser run <GOAL>
bowser tasks list|get|cancel|follow
bowser navigate <URL> --browser <ID>
bowser capture --browser <ID>
bowser click <ELEMENT_ID> --capture <ID> --browser <ID>
bowser type <ELEMENT_ID> <TEXT> --capture <ID> --browser <ID>
bowser key|scroll|back|forward|reload|screenshot|download ...
bowser logs list|follow
bowser usage
bowser local <existing-local-command>
```

`bowser run` creates a task, follows events by default on an interactive TTY,
and prints only the final result on success. `--no-wait` returns the task JSON.
`--json` emits stable machine-readable objects and sends progress to stderr.
`--output-schema <PATH>` validates JSON Schema locally before submission.

## CLI Authentication And Configuration

Precedence is flags, environment, project config, then user config. Supported
hosted variables include `BOWSER_API_KEY`, `BOWSER_BASE_URL`, and
`BOWSER_PROJECT`. The API key may be stored only in a user config file with
owner-only permissions; project config stores a key reference or project ID,
never a secret. No command accepts an API key as a flag because process lists
and shell history are not secret stores.

The CLI never calls E2B directly. It retries safe reads and idempotent accepted
mutations with bounded backoff, honors `Retry-After`, resumes SSE from the last
event ID, and does not retry a rejected request with a new idempotency key.

Exit codes are stable categories: `0` success, `2` invalid local input, `3`
authentication/authorization, `4` not found/conflict, `5` task or command
failure, `6` timeout/cancellation, `7` quota/rate limit, and `10` transport or
server failure. Errors include the Bowser code and request ID without debug
provider output.

## Client Generation And Documentation

The dashboard client and CLI API types derive from the checked OpenAPI
contract. Public API documentation includes curl and CLI examples for every
launch endpoint, describes idempotency and async behavior, and uses real empty
or example-labelled payloads. Examples never contain working credentials,
provider tokens, or claims that CAPTCHA/anti-bot bypass is guaranteed.
