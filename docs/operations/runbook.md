# Production runbook

What an operator needs at 3am. Written for 0.6.0.

## Health, and what each signal means

| Endpoint | Meaning | Probe |
|---|---|---|
| `GET /health/live` | The process is up and its event loop responds. Touches nothing external **on purpose**: a database outage must not restart every pod. | `livenessProbe` |
| `GET /health/ready` | The process will accept work: started, not draining, and a metadata-store round-trip succeeded. | `readinessProbe` |
| `GET /health` | Alias of `/health/ready`, kept for compatibility. | — |
| `GET /metrics` | Prometheus exposition. Unauthenticated by design for scraping — do not expose it publicly. | `ServiceMonitor` |

Before 0.6.0 `/health` was the string literal `"OK"`. It returned `200` whether
or not the database was reachable, and the readiness probe pointed at it, so a
pod with a dead database stayed in the Service and kept taking traffic. If you
are upgrading, repoint your probes.

## Metrics worth alerting on

```
pangolin_http_requests_total{method,route,status}
pangolin_http_request_duration_seconds{method,route,status}   # histogram
pangolin_table_commits_total
pangolin_table_commits_succeeded_total
pangolin_table_commits_conflicted_total
pangolin_table_commit_cas_retries_total
pangolin_auth_success_total
pangolin_auth_failure_total
pangolin_token_revocation_check_errors_total
pangolin_audit_write_failures_total
pangolin_warehouse_cache_hits_total / _misses_total
pangolin_ready                                                 # gauge, 0 or 1
```

Suggested starting rules — tune against your own traffic, these are not
measured SLOs:

| Alert | Expression sketch | Why |
|---|---|---|
| Instance not ready | `pangolin_ready == 0` for 5m | The pod cannot reach its store |
| Elevated 5xx | `rate(pangolin_http_requests_total{status=~"5.."}[5m]) > 0.05 * rate(pangolin_http_requests_total[5m])` | Something is broken |
| Commit conflict storm | `rate(pangolin_table_commits_conflicted_total[5m])` above baseline | Writers are contending, or a client is retrying incorrectly |
| Audit writes failing | `increase(pangolin_audit_write_failures_total[15m]) > 0` | Compliance-relevant: operations are succeeding with no record |
| Revocation checks failing | `increase(pangolin_token_revocation_check_errors_total[5m]) > 0` | Requests are being rejected because the store is unreachable |
| Auth failure spike | `rate(pangolin_auth_failure_total[5m])` above baseline | Credential stuffing, or a broken client |

## Logs

`RUST_LOG` works from 0.6.0. It did not before: `tracing-subscriber` was built
without the `env-filter` feature, so `fmt::init()` installed a fixed-level
subscriber and the variable was silently ignored — while the Dockerfile set it
and the chart documented it as a tunable.

```bash
RUST_LOG=info                                   # default
RUST_LOG=debug                                  # verbose
RUST_LOG=info,pangolin_api::auth_middleware=debug   # per-module
LOG_FORMAT=json                                 # structured, for aggregation
```

Every request carries a `request_id`, echoed in the `x-request-id` response
header and attached to every log line for that request. An inbound
`x-request-id` from a trusted gateway is honoured if it is short and
alphanumeric.

## Common incidents

### The server will not start

Startup validates configuration and fails loudly rather than falling back to
something insecure. Read the first line of stderr.

| Message | Fix |
|---|---|
| `PANGOLIN_JWT_SECRET is not set` | Set it (`openssl rand -base64 48`). For local work only, `PANGOLIN_DEV_MODE=true` generates a random ephemeral secret |
| `PANGOLIN_JWT_SECRET is a known placeholder value` | It is one of the values published in this repository or its chart. Generate a real one |
| `PANGOLIN_JWT_SECRET is too short` | At least 32 bytes |
| `PANGOLIN_NO_AUTH=true … non-loopback bind` | `PANGOLIN_NO_AUTH` disables all authentication; it is refused on a public bind |
| `could not initialise the metadata store` | `DATABASE_URL` is wrong or the database is unreachable |
| `unknown PANGOLIN_STORAGE_TYPE` | One of `memory`, `sqlite`, `postgres`, `mongodb` |

### Pods restart in a loop

Check the liveness probe path. It must be `/health/live`, not `/health` —
readiness depends on the database, and using it for liveness turns a database
outage into a restart storm.

### Clients get 409 CommitFailedException

Expected under concurrent writers: a commit whose `assert-ref-snapshot-id` no
longer holds is now rejected instead of being silently applied to a branch that
moved on. Engines retry. Investigate only if
`pangolin_table_commits_conflicted_total` rises without a matching rise in
`pangolin_table_commits_succeeded_total`, which suggests a client that does not
retry.

### Clients get 501 UnsupportedOperationException on commit

The commit contained an update type Pangolin does not implement. Before 0.6.0
these were discarded and `200 OK` returned, so the operation appeared to succeed
and did not happen. The error is the honest answer. The message names the
operation.

### Storage credentials are stale after rotation

The warehouse cache is node-local with a 5-second TTL by default (it was 60s).
With more than one replica, a peer can vend the previous credential for up to
the TTL after you rotate. Set `PANGOLIN_WAREHOUSE_CACHE_TTL_SECS=0` to disable
caching during a rotation. Cross-node invalidation is not implemented (A-28).

### An audit write failed

`pangolin_audit_write_failures_total` increments and the failure is logged at
error level with the reason. The operation itself still succeeded — audit
writes are best-effort, buffered retry is Phase 3.5. For a compliance-relevant
window, reconcile from application logs, which carry the same request IDs.

## Rolling upgrade

The server installs a SIGTERM handler, stops reporting ready immediately, and
drains in-flight requests before exiting. Give it room:

* `terminationGracePeriodSeconds: 40` (chart default)
* `PANGOLIN_SHUTDOWN_GRACE_SECS=25` (default)

This matters more than it looks. A table commit writes its metadata file to
object storage *before* the compare-and-swap that publishes it; a SIGKILL in
that window leaks an orphaned metadata file with no cleanup path. There is no
reaper yet (Phase 1.11), so orphans accumulate until you remove them by hand.

Upgrade order: run migrations by rolling one replica first (schema changes are
additive and applied under an advisory lock, so mixed versions are safe for a
short window), confirm it becomes ready, then roll the rest.

## Backup and restore

**Not yet tested end to end, and there is no RPO/RTO commitment.** Stated
plainly because a runbook that overclaims here is worse than one that says
nothing (C-16).

What to back up:

1. **The metadata database.** For PostgreSQL, `pg_dump` plus WAL archiving for
   point-in-time recovery. This is the authoritative index of your lake; losing
   it means losing the ability to read the lake even though every byte of data
   is still in object storage.
2. **Object storage** is usually versioned independently and not Pangolin's
   concern, but the metadata files under `<warehouse>/metadata/` must be
   restored to the same point as the database or tables will point at metadata
   that no longer exists.

Restore, PostgreSQL:

```bash
kubectl scale deployment/pangolin --replicas=0
pg_restore --clean --if-exists -d "$DATABASE_URL" backup.dump
kubectl scale deployment/pangolin --replicas=3
```

Then check `GET /health/ready` on each pod and list a known table through the
Iceberg API.

Because administrative multi-statement operations are not transactional yet
(see `backend-parity.md`), a restore can land mid-operation. There is no
consistency checker; inspect branch heads and metadata pointers for tables that
were being written at the backup point.

## Audit-log retention

`audit_logs` is range-partitioned by `timestamp` on PostgreSQL, with a catch-all
default partition. To implement retention, attach dated partitions and drop old
ones:

```sql
CREATE TABLE audit_logs_2026_09 PARTITION OF audit_logs
    FOR VALUES FROM ('2026-09-01') TO ('2026-10-01');

DROP TABLE audit_logs_2025_09;   -- past your retention window
```

There is deliberately no foreign key from `audit_logs` to `tenants`. The
previous `ON DELETE CASCADE` erased a tenant's audit trail exactly when it
became most interesting.

## Capacity

No published throughput or latency figures, and no load-test harness
(C-15/3.14). The chart's defaults — 250m CPU / 512Mi requested, 2 CPU / 2Gi
limit — are a starting point, not a measurement. Watch
`pangolin_http_request_duration_seconds` and size from your own traffic.

Two things constrain horizontal scaling today:

* The warehouse cache is node-local (above).
* `cleanup_job.rs` runs token cleanup in **every** replica with no coordination,
  so N replicas run N concurrent cleanups. Harmless but wasteful; leader
  election is Phase 3.6.
