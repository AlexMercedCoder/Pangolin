# Changelog

All notable changes to Pangolin are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/) — with the caveat that
Pangolin is pre-1.0, so a minor release may contain breaking changes.

From 0.6.0 the server, both CLIs, the Python SDK, the UI and the Helm chart all
carry the same version number. Before that they had drifted to five different
values and there was no way to tell which combination had been tested together.

## [0.6.0] — 2026-08-09

**This is a security release.** If you are running any earlier version, upgrade
and rotate credentials. See [SECURITY.md](SECURITY.md) for the advisory and
[upgrade steps](#upgrading-to-060).

This release implements Phase 0 and much of Phases 1–3 of
`AUDIT_EXECUTION_PLAN.md`.

### Security

- **Fixed OAuth session-token exfiltration (A-8).** The callback appended the
  freshly minted session JWT to a redirect URL taken from the unsigned,
  unvalidated `state` parameter, with no allowlist on the destination. An
  authorize link whose `state` decoded to `{"redirect_uri":"https://evil.example/"}`
  delivered a valid token for the victim to the attacker's access log — account
  takeover with no credential theft. `state` is now HMAC-SHA256 signed with an
  expiry, redirect targets are allowlisted by exact match, and only an index
  into that allowlist travels inside `state`. **The token is no longer placed in
  a URL at all**: the callback returns a single-use code redeemed over
  `POST /api/v1/oauth/exchange`.
- **Fixed OAuth login CSRF (A-9).** The `state` nonce was generated and never
  stored or verified, and the callback did not validate `state` at all. Nonces
  are now registered server-side, consumed exactly once, and bound to the
  provider that issued them.
- **Removed every insecure default (A-10).** `PANGOLIN_JWT_SECRET` fell back to
  `default_secret_for_dev`, a value published in this repository, so anyone
  could forge a `Root` token against a deployment that missed one environment
  variable. The Helm chart shipped `change-me-please` and `password` as
  *working* defaults, and the seeded admin used `password123`. All are gone. The
  server refuses to start without a strong secret, rejects known placeholders,
  and refuses `PANGOLIN_NO_AUTH` on a non-loopback bind. The chart refuses to
  render without a real secret.
- **Fixed authentication bypass via path suffix (A-11).** The whitelist matched
  any path *ending in* `/config` and any path *containing* `/oauth/tokens`, so a
  namespace or table named `config` was reachable unauthenticated — including
  its DELETE route. Matching is now structural per route segment.
- **Fixed an unauthenticated denial-of-service primitive (A-12).** API-key
  authentication ran `bcrypt::verify` against every service user in every
  tenant; at 100 service users one request with a bogus key burned roughly 25
  CPU-seconds, before any rate limiting. Keys now carry a public key ID
  (`pgl_<key-id>_<secret>`), so authentication is one string compare plus at
  most one bcrypt verification.
- **Token revocation now fails closed (A-13).** A store error during the
  revocation check was logged and ignored, so every revoked token was accepted
  again during a database disruption. It now returns `503`.
- **Credentials are compared in constant time (A-14).** The root password used
  `==`, which leaks the matching prefix length through timing.
- **Removed the `admin`/`password` login fallback.** `POST /api/v1/users/login`
  defaulted the root credentials to `admin`/`password` when the environment
  variables were unset.
- **Deleted a dead auth middleware whose fallback allowed every unauthenticated
  request through.**
- **Root tenant impersonation is now audited (C-7)**, along with failed logins,
  rejected API keys and rejected tokens (C-19).
- **The startup bootstrap token is now valid for one hour, not 365 days (C-5).**
  It is printed to stdout, where container log aggregation captures it.
- **`GET /v1/config` no longer exposes credential-bearing warehouse properties.**

### Fixed — Iceberg correctness

- **Commit requirements are enforced (A-1).** Only `assert-current-schema-id`
  and `assert-table-uuid` were implemented; the rest were discarded. The one
  that matters most is `assert-ref-snapshot-id`, which is how a writer says
  "only commit if the branch still points where I think it does". Without it, a
  writer whose compare-and-swap lost would retry against the winner's metadata
  and blindly re-apply its own snapshot, producing forked snapshot lineage and
  orphaned data files with **no error ever surfaced**. All requirements are now
  checked on every attempt, and an unrecognised one is refused.
- **Commit updates are applied or refused, never silently dropped (A-2).**
  Eleven update types — `set-properties`, `remove-properties`, `set-location`,
  `add-spec`, `set-default-spec`, `add-sort-order`, `set-default-sort-order`,
  `remove-snapshots`, `set-snapshot-ref`, `remove-snapshot-ref`, `assign-uuid`,
  `upgrade-format-version` — were discarded while the handler returned `200 OK`
  with a fresh metadata file. All are implemented; an unrecognised update now
  returns `501`.
- **`last_sequence_number` is a monotonic counter again (A-3).** It was being
  assigned the snapshot ID, a random 64-bit value. Sequence numbers govern how
  position and equality delete files apply to data files, so corrupting them can
  produce incorrect query results on merge-on-read tables.
- **`GET /v1/{prefix}/config` returns per-warehouse configuration (A-4).** The
  handler took no arguments at all: it ignored the `prefix` path segment and the
  spec's `?warehouse=` parameter and built `defaults` from process-wide
  environment variables, so a tenant with an Azure warehouse received the
  server's AWS settings. It also never returned `overrides.prefix`.
- **Errors on `/v1/*` use the Iceberg error envelope (A-6)**,
  `{"error":{"message","type","code"}}`, so engines can distinguish
  `NoSuchTableException` from `CommitFailedException` and retry correctly.
- **`TableMetadata` gained `refs`**, required for `set-snapshot-ref` and
  `assert-ref-snapshot-id`.

### Fixed — storage and data integrity

- **PostgreSQL could not be provisioned from a fresh database.**
  `active_tokens` and `federated_sync_stats` were defined only in a schema file
  no runner applied, while a migration created an index on `active_tokens`, so
  the chain aborted and the server could not start. Both are now created before
  the migrator runs, under a `pg_advisory_lock` so concurrent replicas do not
  race.
- **PostgreSQL audit logging never worked.** The migration chain's `audit_logs`
  carried the original `actor`/`resource`/`details` shape with a `BIGINT`
  timestamp while the code inserted the enhanced shape with a `TIMESTAMPTZ`;
  every write failed. The table is rebuilt with the correct schema, `TEXT`
  columns matching the `sqlx` type mapping, and no `ON DELETE CASCADE` foreign
  key — which used to erase a tenant's audit trail exactly when it mattered
  most.
- **`access_requests` gained its missing `tenant_id` column**, without which
  every access-request query failed.
- **Branch merges ran in the wrong direction.** `CatalogStore::merge_branch` is
  declared `(source, target)`; the handler passed `(target, source)`. Backends
  compensated inconsistently, so MemoryStore merged `main` into `dev` when asked
  for `dev` into `main`.
- **A branch merge now moves assets created on the branch.** MemoryStore
  iterated an asset list captured when the branch was created, so anything
  created afterwards was never merged.
- **Backend parity gaps closed (A-25).** MemoryStore implements
  `update_service_user_last_used` and `update_access_request`; both previously
  fell through to "Operation not supported by this store", the latter surfacing
  as a `500` when approving an access request.
- **`search_assets` no longer defaults to success (A-26).** A backend without
  search returned `Ok(vec![])` — "no results found" — instead of an error, so
  users concluded their data was missing.
- **SQLite has versioned migrations (A-27)** via `SqliteStore::run_migrations`,
  recording a version in `_pangolin_schema_version`, and gained the missing
  `revoked_tokens` table without which token revocation failed at runtime.
- **Consolidated three sources of schema truth into one per backend (A-27).**
  The orphaned root `migrations/` tree, `pangolin_store/migrations/sqlite/`
  (skipped by `sqlx::migrate!`) and the superseded `sql/postgres_schema.sql`
  are gone; what only they defined has been folded into the surviving source.
  See `pangolin/pangolin_store/migrations/README.md`.
- **The MongoDB audit log stores `resource_id` as BSON binary**, so reading back
  an entry that had one no longer fails and take the whole listing with it.
- **SQLite honours `DATABASE_MAX_CONNECTIONS` (A-29)**, which only PostgreSQL
  did.

### Added — observability and reliability

- **Prometheus metrics at `/metrics` (A-18)**: RED metrics per route, commit
  success/conflict/retry counters, authentication outcomes, audit-write
  failures, cache hit rates and a readiness gauge. A `ServiceMonitor` template
  ships with the chart.
- **`RUST_LOG` works (A-17).** `tracing-subscriber` was built without the
  `env-filter` feature, so `fmt::init()` installed a fixed-level subscriber and
  the variable was silently ignored — while the Dockerfile set it and the chart
  documented it as a tunable. Adds `LOG_FORMAT=json` for structured logs.
- **Request correlation IDs**, honoured from a trusted gateway's
  `x-request-id`, attached to every log line and echoed on the response.
- **Real health endpoints (A-21).** `/health` was `get(|| async { "OK" })` and
  returned `200` whether or not the database was reachable, with the readiness
  probe pointing at it, so a pod with a dead database stayed in the Service.
  `/health/live` (process only) and `/health/ready` (store round-trip) are now
  distinct.
- **Graceful shutdown (A-19).** SIGTERM and SIGINT stop readiness immediately
  and drain in-flight requests. A commit writes its metadata file before the
  compare-and-swap that publishes it, so a hard kill in that window leaks an
  orphaned file.
- **Request limits (A-20)**: body size limit, per-request timeout, and a global
  concurrency limit. There were none of any kind.
- **A typed, validated `AppConfig` (A-30)**, resolved once at startup instead of
  39 environment variables read from 88 scattered call sites, several of them on
  every request.
- **`PANGOLIN_STORAGE_TYPE` is honoured.** It was read into a variable and never
  used, while `example.env`, `.env` and `values.yaml` all documented it as the
  way to choose a backend.
- **`docker HEALTHCHECK`** via a `--healthcheck` mode on the binary.

### Added — deployment

- **The Helm chart's three missing templates (A-34)**: `serviceaccount.yaml`,
  `ingress.yaml` and `hpa.yaml`. `values.yaml` exposed all three with nothing
  behind them, so enabling autoscaling removed the replica count and created no
  autoscaler, and pods referenced a ServiceAccount that was never created. Adds
  a `PodDisruptionBudget` and a `ServiceMonitor`.
- **Hardened pod and container defaults (A-35)**: `runAsNonRoot`,
  `readOnlyRootFilesystem`, all capabilities dropped, `RuntimeDefault` seccomp.
- **Real resource requests and limits (A-37).** `resources: {}` put pods in the
  BestEffort QoS class, first to be evicted under node pressure.
- **A non-root container (A-35)** running as UID 10001 under `tini`, with
  `libssl3` instead of the `libssl-dev` *development* package, `--locked`
  builds, dependency-caching layers, OCI labels, and a base image matching the
  Rust version the README requires.
- **`image.tag` pinned to the release (A-38)** instead of `latest`.

### Added — engineering substrate

- **CI on push and pull request (C-22)** — build, `cargo fmt --check`, clippy,
  `cargo test --workspace` both with and without databases, `cargo audit`,
  `helm lint`/`helm template`, and a Docker build that asserts the image does
  not run as root. Its absence is the root cause of nearly everything above.
- **The test suite runs again (B-11).** `cargo test --workspace` executed
  **zero** tests: five targets failed to compile because model structs had
  gained fields and functions had gained parameters with no test updated, and
  nothing had compiled the test code in a long time. It now runs **330 tests,
  all passing**, with 10 skipped when no database is configured.
- **The tenant-isolation tests pass (B-13/C-6)**, and now exercise the
  **production** middleware rather than a drifted test-only wrapper that did not
  cover service-user keys or token revocation (A-15).
- **Regression tests for every security and correctness fix**, including the
  lost-update scenario, `config`-named resources, signed-state replay, and
  redirect allowlisting.
- **`cargo fmt --all` across the workspace** — 227 of 238 files were unformatted
  — in one isolated commit recorded in `.git-blame-ignore-revs`.
- **clippy warnings reduced from 314 to under 50**, with a budget file that CI
  fails on if the count grows. `unsafe_code = "forbid"` locks in the workspace's
  existing zero-`unsafe` property.
- **Removed 30+ committed debug artifacts** and the stray
  `pangolin_store/:memory:` file, and stripped code-generation deliberation
  comments left in `main.rs`, `auth.rs` and `pangolin_store/Cargo.toml`.
- **Test fixtures no longer ship in the release binary (B-7).**
- **Backend tests skip rather than fail** when no database is configured, and
  PostgreSQL and MongoDB no longer fight over `DATABASE_URL`.

### Added — documentation

- `SECURITY.md`, `CONTRIBUTING.md` and this changelog, which did not exist.
- `docs/operations/runbook.md` — health, metrics, incidents, upgrades, backup.
- `docs/operations/backend-parity.md` — which features work on which backend.
- `docs/operations/oidc.md` — OAuth configuration and the client change above.
- `pangolin_store/migrations/README.md` — the one source of schema truth.
- **The README's claims are now accurate.** "100% compliant with the Apache
  Iceberg REST spec" and eleven features marked "Production-Ready" were not
  supported by the code. The README now carries a maturity table, an explicit
  list of known limitations, and precise Iceberg REST coverage.

### Changed — breaking

- **OAuth clients must be updated.** The callback redirect carries `code`, not
  `token`; exchange it at `POST /api/v1/oauth/exchange`. See
  [docs/operations/oidc.md](docs/operations/oidc.md).
- **`PANGOLIN_JWT_SECRET` is required.** The server will not start without it
  unless `PANGOLIN_DEV_MODE=true`. Setting it invalidates every existing
  session, which is intended.
- **Service-user API keys should be rotated.** Legacy keys work only with
  `PANGOLIN_ALLOW_LEGACY_API_KEYS=true`.
- **Seeding an admin requires `PANGOLIN_ADMIN_PASSWORD`.** There is no default.
- **`POST /api/v1/users/login` no longer has default root credentials.** Set
  `PANGOLIN_ROOT_USER` and `PANGOLIN_ROOT_PASSWORD`, or use a database user.
- **OAuth redirect targets must be allowlisted** via
  `PANGOLIN_OAUTH_REDIRECT_URIS` (`FRONTEND_URL` is always allowed).
- **Health probes should move** to `/health/live` and `/health/ready`.
  `/health` still works and is equivalent to readiness.
- **Unsupported Iceberg commit operations now return `501`** where they
  previously returned `200 OK` and did nothing. A client that appeared to work
  may now surface a real error.
- **`CatalogStore::search_assets` returns an error** on backends that do not
  implement it, instead of an empty result.
- **The Helm chart will not install without a real `PANGOLIN_JWT_SECRET`.**
- **`AuditLogEntry::legacy_new` is removed.** Use `::new()` or `::success()`.
- **Chart version jumps from 0.1.0 to 0.6.0** to match everything else.

### Upgrading to 0.6.0

1. Generate a signing secret: `openssl rand -base64 48`, and set
   `PANGOLIN_JWT_SECRET`. Existing sessions end.
2. If you use OAuth, set `PANGOLIN_OAUTH_REDIRECT_URIS` and update clients to
   the code-exchange flow.
3. If you seed an admin, set `PANGOLIN_ADMIN_PASSWORD`.
4. Rotate service-user API keys, or set `PANGOLIN_ALLOW_LEGACY_API_KEYS=true`
   temporarily.
5. Repoint Kubernetes probes at `/health/live` and `/health/ready`.
6. Roll one replica first: schema migrations are additive and applied under an
   advisory lock, so a brief mixed-version window is safe.
7. Review your audit log for `tenant_impersonation` events once they start
   being recorded.

### Still outstanding

Honestly stated, and tracked in `AUDIT_EXECUTION_PLAN.md`:

- Multi-statement transactions for administrative operations on PostgreSQL and
  MongoDB (A-24, Phase 1.7). The Iceberg commit path is safe; branch merges,
  branch creation by copy, and cascading deletes are not atomic.
- Rate limiting on authentication endpoints (C-5).
- Full OIDC: PKCE, JWKS, `id_token` validation, discovery, lookup by
  `(provider, subject)` (C-2/C-3, Phase 3.1).
- Asymmetric JWTs with rotation (C-4).
- Envelope encryption for warehouse credentials at rest (C-11).
- Tamper-evident audit records and SIEM export (C-20).
- Tested backup, restore and DR with a published RPO/RTO (C-16).
- Cross-node warehouse cache invalidation and coordinated background jobs
  (A-28/C-14).
- The missing Iceberg endpoints: `loadNamespaceMetadata`, `namespaceExists`,
  `registerTable`, `commitTransaction`, and most of the view API (A-5).
- MongoDB parity: index management, transactions, and four known-failing tests.

## [0.5.1] and earlier

No changelog was kept before 0.6.0. See the git history.

[0.6.0]: https://github.com/AlexMercedCoder/pangolin/releases/tag/v0.6.0
