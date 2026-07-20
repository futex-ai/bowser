# Managed Infrastructure And Operations

Terraform declares GCP resources and IAM. Helm declares Kubernetes workloads.
No console-only resource is part of a reproducible environment.

## Target And Preconditions

The initial target project is `bowser-503012`. The recommended primary region
is `europe-west2`; it remains a Terraform variable so a production project can
select a different region for residency or latency.

Before the first plan or apply:

- reauthenticate `gcloud` and select `bowser-503012` explicitly;
- inventory existing services, IAM, networking, DNS, state buckets, GKE,
  Cloud SQL, Artifact Registry, and budgets;
- confirm billing, organization policies, domain ownership, E2B plan and
  concurrency, model-provider access, and a managed egress provider;
- import matching existing resources instead of recreating or overwriting
  them;
- create a remote Terraform state bucket with versioning and restricted IAM.

The current non-interactive credential check fails token refresh and the
active local config points to `polybase-internal`, so no assumption is made
that the target project is empty or ready.

## GCP Resource Topology

Terraform provisions:

- a custom VPC, regional subnets, secondary GKE ranges, private Google access,
  Cloud NAT, firewall policy, and Private Service Access;
- a regional GKE Autopilot cluster with Workload Identity Federation, managed
  Gateway support, release-channel pinning, and private workload nodes;
- Cloud SQL for PostgreSQL on private IP, a zonal non-production instance and
  regional HA production configuration, automated backups, and PITR;
- Artifact Registry repositories for service images;
- private GCS buckets for web assets, customer artifacts, profile archives,
  and deployment records, each with explicit lifecycle/CORS policy;
- Pub/Sub work and dead-letter topics plus pull subscriptions;
- Secret Manager containers and KMS key rings/keys;
- Identity Platform/Firebase web app configuration;
- external HTTPS load balancing, managed certificates, Cloud DNS records when
  Bowser owns the zone, and Cloud Armor rate/WAF policies;
- service accounts, least-privilege IAM, log sinks, monitoring dashboards,
  alert policies, uptime checks, and GCP budgets.

Production must use a separate GCP project before accepting customer data. The
same modules instantiate it; namespace separation inside `bowser-503012` is
for development and staging, not a production security boundary.

## Terraform Layout

```text
infra/terraform/
  modules/{project-services,network,gke,postgres,storage,pubsub,identity,edge,observability}
  environments/{dev,staging,prod}
```

Modules expose typed inputs and outputs and contain no environment credentials.
Environment roots pin provider/module versions and backend configuration.
CI runs formatting, validation, lint/security scans, and a saved plan. Apply is
manual for production and uses Workload Identity Federation, never a service
account key.

Terraform creates secret resources but not secret payloads. Values are written
through a separately audited bootstrap/release step so they do not enter Git,
plan output, or state. Destructive database, bucket, key, and project changes
require deletion protection and explicit operator approval.

## Helm Layout And Workloads

`infra/helm/bowser` deploys independently scalable components:

- `api`: stateless REST/SSE service;
- `worker`: direct-command and agent task consumers;
- `outbox-relay`: publishes committed database outbox records;
- `reconciler`: repairs E2B and database lifecycle drift;
- `retention-worker`: expires artifacts and customer content;
- `live-proxy`: authorizes and proxies noVNC WebSockets;
- `migration`: one-shot pre-upgrade Diesel migration job.

Each workload has a distinct Kubernetes and Google service account, resource
requests/limits, probes, disruption budget where useful, network policy,
topology spreading, termination grace, and autoscaling signal. API and worker
pods use the Cloud SQL Auth Proxy or approved connector with Workload Identity;
the database has no public IP.

Helm values contain resource names and Secret Manager references, not secret
values. Charts render successfully for dev, staging, and production and are
checked with schema validation and policy tests.

## Network And IAM Boundaries

Only the external load balancer is public. The API accepts customer traffic;
the live proxy accepts short-lived Bowser live tokens. Workers, relay,
reconciler, migration, and PostgreSQL are private.

Workloads receive outbound access only to their required Google APIs and
providers. E2B browser traffic leaves from E2B or the selected egress proxy,
not the GCP VPC. Customer-controlled URLs are never fetched by API pods.

IAM separates infrastructure apply, deploy, migration, API runtime, worker,
live proxy, and operator access. Break-glass access is time-bound and audited.
GKE uses
[Workload Identity Federation](https://cloud.google.com/kubernetes-engine/docs/concepts/workload-identity)
instead of exported Google service-account keys.

## Database Operations

Migrations are forward-only once merged. A release job runs the exact image to
be deployed, acquires a migration lock, reports the current and target schema,
and finishes before new pods roll out. Rollback deploys compatible application
code; it does not rewrite an applied migration.

Connection pools are bounded per deployment so total connections remain below
the Cloud SQL limit during rollout. Production uses regional HA, automated
backups, PITR, deletion protection, and quarterly restore exercises. Google
documents regional Cloud SQL as synchronously replicated across zones for
[high availability](https://cloud.google.com/sql/docs/postgres/high-availability).

## Messaging And Work Leases

Pub/Sub pull subscriptions use a dead-letter policy and bounded retry backoff.
Browser ID is the ordering key where ordering materially reduces contention,
but database fencing remains authoritative. Workers extend acknowledgements
while their database lease is valid and acknowledge only after a terminal
transition or durable retry schedule.

The design assumes at-least-once delivery, consistent with Pub/Sub's documented
[default behavior](https://cloud.google.com/pubsub/docs/subscription-overview).
Exactly-once delivery may be enabled later as an optimization, not as the only
duplicate defense.

## Build And Deployment Pipeline

CI performs these immutable build steps:

1. run repository checks and all credential-free tests;
2. build and scan multi-stage service images and the Linux sandbox binary;
3. publish images by digest to Artifact Registry;
4. build an E2B template from that binary, run live template smoke tests, and
   record its build ID;
5. deploy image digests and the E2B build ID to staging with Helm;
6. run API, CLI, agent, egress, pause/resume, live-view, and reconciliation
   staging smoke tests;
7. promote the same digests/build ID to production after approval.

No deployment uses mutable `latest` tags. Rollback restores the prior image
digests and E2B build ID. Existing browsers remain on their creation build;
new browsers use the promoted build.

## Scaling, Reliability, And Cost

Launch objectives are 99.9% monthly API availability, p95 accepted-mutation
latency below 300 ms excluding provider readiness, and no operation stranded
without a terminal state or active lease for more than five minutes. Browser
startup has a separately measured provider-dependent SLI.

Autoscaling uses request concurrency for API/live proxy, Pub/Sub backlog age
for workers, and lag for relay/reconciliation. Project admission limits protect
E2B concurrency and model/proxy spend before provider calls. A global emergency
control can reject new browsers/tasks while preserving reads, stops, and
cleanup.

GCP budgets and per-service labels cover GCP spend. Bowser records E2B, model,
and egress-provider usage in its own ledger and alerts on invoice drift,
orphaned sandboxes, and daily/project caps. Paused E2B sessions remain governed
by Bowser's absolute lifetime and retention jobs even if upstream pause storage
is indefinite.

## Environment Configuration

Runtime configuration is typed and validated at startup. It includes public
base URLs, project/region, database connection target, topic/subscription IDs,
bucket names, E2B template build ID, default limits, retention, model route,
and feature gates. Missing required production configuration is fatal.

Normal product views do not display environment names. Environment selection
comes from deployment configuration and URL, not user-facing badges or copy.
