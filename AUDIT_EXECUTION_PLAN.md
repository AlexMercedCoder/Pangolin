# Pangolin — Improvement Audit & Execution Plan

**Audit date:** 2026-08-09
**Repository root:** `/home/alexmerced/development/personal/Personal/library/2026/pangolin`
**Rust workspace:** `pangolin/` (6 crates) · **Scope of audit:** Rust server, storage, CLI, deployment assets, CI
**Version state at audit:** `pangolin_api` 0.5.1, `pangolin_core`/`pangolin_store` 0.2.0, CLIs 0.5.0, Helm chart 0.1.0
**Status claimed in README:** Alpha, with 11 features marked "Production-Ready"

> This document is an assessment and a plan. No source files were modified as part of this audit.

---

## 1. Executive Summary

Pangolin is a substantially complete lakehouse catalog: ~43,000 lines of Rust across 238 files, four metadata backends (Memory, SQLite, PostgreSQL, MongoDB), three cloud storage integrations, an Iceberg REST surface, Git-style branching and merge, RBAC, audit logging, service users, OAuth SSO, a Python SDK, two CLIs, and a SvelteKit UI. The breadth is genuinely impressive for a project with 186 commits, and several parts are well-built — the Postgres backend has a proper 12-file `sqlx` migration chain, the metadata-pointer commit path uses compare-and-swap with retry, and there is **zero `unsafe` code** in the entire workspace.

The gap is not features. It is that the engineering substrate that would let anyone *trust* those features is largely absent, and as a result several of the flagship features are quietly broken.

Three findings define the current state:

1. **There is no CI that builds or tests the code.** The only GitHub Actions workflow (`.github/workflows/build-binaries.yml`) runs on tag push and only compiles release binaries. No `cargo test`, `clippy`, `fmt`, or dependency audit ever runs on a commit or PR.
2. **Because of that, the test suite has silently rotted to the point where it cannot run.** `cargo test --workspace` executes **zero tests** — it aborts during compilation. Five test targets fail to compile because model structs gained fields (`Permission.tenant_id`, `ServiceUser.last_used`, `Role.created_by`) and function signatures changed, and no test code was updated. Of the ~304 declared test functions, I was able to execute roughly 44 without external infrastructure. Two of those — the **tenant-isolation tests**, which verify the project's headline multi-tenancy guarantee — fail deterministically against the in-memory backend.
3. **There is a critical, remotely exploitable authentication vulnerability**, plus a data-corruption bug in the Iceberg commit path. The OAuth callback mints a session JWT and appends it to a redirect URL taken from the unvalidated, unsigned `state` parameter, enabling full account takeover. Separately, the table-commit handler silently ignores most Iceberg commit requirements and updates, returning `200 OK` for operations it did not perform.

The good news is that the fixes are tractable and mostly independent. The critical security items are each a few dozen lines. The correctness items are contained in one file. The hygiene items are largely mechanical. **The single highest-leverage action is standing up CI**, because without it every fix below is one refactor away from silently regressing — which is precisely how the current state was reached.

> ## Status as of 2026-08-11 — read this first
>
> This document is the **original audit of 2026-08-09**, kept as the historical
> record of what was found. Much of it has since been fixed. Where this file and
> the reconciled status below disagree, **the status below is correct**.
>
> Current state is tracked in:
>
> - [`STATUS.md`](STATUS.md) — the single reconciled view of done vs. outstanding
> - [`CHANGELOG.md`](CHANGELOG.md) — what changed in each release, and why
> - [`SECURITY.md`](SECURITY.md) — the advisory and the remaining security gaps
> - [`docs/operations/`](docs/operations/) — backend parity, encryption, backup
>   and recovery, performance, multiple replicas
>
> The scorecard immediately below has been updated in place; every other section
> of this document is left as it was written on 2026-08-09.

### Readiness scorecard

**Updated 2026-08-11.** Ratings in parentheses are the original 2026-08-09
assessment, kept so the direction of travel is visible.

| Dimension | Rating | One-line justification |
|---|---|---|
| Feature breadth | **Strong** (Strong) | Four backends, three clouds, branching/merge, RBAC, audit, SSO, SDK, UI |
| Iceberg REST correctness | **Good** (Weak) | Requirements and updates enforced; `registerTable`, `listViews`, `viewExists`, `dropView` added. `commitTransaction` deliberately absent — see below |
| Security (authn/authz) | **Adequate** (Critical) | OAuth exfiltration, default JWT secret, bypass path and the 0.7.0 authorization cluster all fixed; rate limiting and credential encryption added. OIDC is still not OIDC |
| Error handling | **Adequate** (Weak) | Iceberg error envelope conforms; `unwrap()` counts unchanged in non-Iceberg paths |
| Observability | **Good** (Absent) | `/metrics` with latency histograms, request tracing, `RUST_LOG` honoured, `/health/live` and `/health/ready` |
| Reliability | **Good** (Weak) | Graceful shutdown, timeouts, body and concurrency limits, readiness that probes the store |
| Data integrity | **Good** (Weak) | Postgres and SQLite wrap catalog delete, branch delete, merge, and branch-create-by-copy. MongoDB wraps the cascade where a session exists |
| Test coverage | **Good** (Critical) | 63 targets / 415 tests green against live PostgreSQL, MongoDB and MinIO; 19 CI jobs including an authz matrix and a four-backend parity suite |
| Code hygiene | **Adequate** (Weak) | `rustfmt` clean; clippy at a ratcheted budget of 30, down from 314 |
| Enterprise readiness | **Partial** (Partial) | Credentials encrypted at rest, backup/restore drilled and measured, multi-replica constraints documented. No HA proof, no tamper-evident audit, no OIDC |
| Deployment | **Good** (Partial) | Helm lints and templates; container runs non-root; release pipeline actually produces a release (it never had) |
| Documentation | **Strong** (Strong user / Absent contributor) | CONTRIBUTING, SECURITY, CHANGELOG, and an operations set covering parity, encryption, backup, performance and replicas |

**Still weak, stated plainly:** OIDC (no PKCE, no JWKS, no `id_token`
validation), no tamper-evident audit trail, no point-in-time recovery,
multi-replica operation untested under load, and `commitTransaction` absent
because the store cannot commit several tables atomically.

| Dimension | Rating | One-line justification |
|---|---|---|
| Feature breadth | **Strong** | Four backends, three clouds, branching/merge, RBAC, audit, SSO, SDK, UI |
| Iceberg REST correctness | **Weak** | Commit requirements/updates silently dropped; ~6 spec endpoints missing; non-spec error envelope |
| Security (authn/authz) | **Critical** | OAuth token exfiltration; default JWT secret; auth-bypass path pattern; O(n) bcrypt key scan |
| Error handling | **Weak** | `ApiError` exists but is used in 5 of 46 API files; 128 `unwrap()` in `pangolin_api`, 155 in `pangolin_store` |
| Observability | **Absent** | No metrics, no request tracing, `RUST_LOG` silently ignored |
| Reliability | **Weak** | No graceful shutdown, no timeouts/body limits, static `/health` |
| Data integrity | **Weak** | Postgres and Mongo backends use **zero** transactions |
| Test coverage | **Critical** | `cargo test --workspace` runs 0 tests; 5 targets don't compile; no CI |
| Code hygiene | **Weak** | 227/238 files fail `rustfmt`; 314 clippy warnings; AI-agent narration left in source |
| Enterprise readiness | **Partial** | RBAC/audit/SSO/multi-tenancy exist but are unverified; no HA, backup, or alerting story |
| Deployment | **Partial** | Helm chart references templates that don't exist; container runs as root |
| Documentation | **Strong (user) / Absent (contributor)** | 134 docs files; no CONTRIBUTING, SECURITY, or CHANGELOG |

---

## 2. Methodology — What Was Run vs. Skipped

Grounding the audit in real tool output mattered more than usual here, because several findings (broken test targets, unformatted tree) are invisible to reading alone.

### Executed

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **Fails.** 2,722 diff hunks across **227 of 238** `.rs` files |
| `cargo clippy --workspace --all-targets` | Completed. **314 unique warnings** + **5 test targets fail to compile** |
| `cargo test --workspace --no-fail-fast` | **Zero tests executed** — aborts on compilation errors |
| `cargo test -p pangolin_core --lib` | 6 passed |
| `cargo test -p pangolin_store --lib` | 14 passed, 27 failed (Postgres/Mongo targets need live databases — environmental) |
| `cargo test -p pangolin_api` (7 compilable integration targets) | 24 passed, 3 failed (`isolation_test` ×2, `merge_tests` ×1) |
| Static review | Workspace manifests, router, auth middleware ×3, Iceberg handlers, `CatalogStore` trait, all four backends, cache layer, Dockerfile, Helm chart, CI, migrations |

### Skipped, and why

| Tool | Reason |
|---|---|
| **Coverage** (`cargo-llvm-cov`, `cargo-tarpaulin`) | Neither is installed. Installing plus an instrumented rebuild of this dependency graph (AWS + Azure + GCP SDKs, `sqlx`, `mongodb`) would take a long time on a cold `target/`. More importantly, **a coverage number is meaningless while five test targets fail to compile** — the measurement should be taken after Phase 2.1. Test counts below are from static analysis and actual run output instead. |
| `cargo audit` / `cargo deny` | Not installed; no advisory database locally. Flagged as a Phase 1 CI deliverable. |
| Live end-to-end (`docker-compose.emulators.yml`, PyIceberg matrix) | Requires MinIO/LocalStack/Postgres/Mongo containers. Out of scope for a read-only audit pass. |
| UI (`pangolin_ui`) and SDK (`pypangolin`) deep review | Out of the stated Rust-focused scope; touched only where they affect versioning and release. |

**Caveat on the 27 store-test failures:** these are Postgres/Mongo parity and merge tests that connect to databases not running in this environment. They are *not* counted as defects. The `isolation_test` and `merge_tests` failures **are** counted, because they use `MemoryStore` and need nothing external.

---

## 3. Area A — Production Readiness of Current Features

### 3.1 Iceberg REST spec compliance and commit correctness

The README states the Iceberg REST Catalog API is "100% Compliant" and lists it as production-ready. The code does not support that claim.

**A-1 (Critical) — Commit requirements are silently discarded.** `pangolin/pangolin_api/src/iceberg/tables.rs:600-628` matches on `CommitRequirement` but implements only `AssertCurrentSchemaId` and `AssertTableUuid`, ending with `_ => {}` at line 628. The Iceberg spec defines roughly eight requirement types; the one that matters most for concurrency is `assert-ref-snapshot-id`, which is how a writer says *"only commit if the branch still points where I think it does."* Dropping it defeats optimistic concurrency control.

The consequence is concrete. The handler does use compare-and-swap on the metadata pointer (`update_metadata_location`, line 683) with a five-attempt retry loop (line 582) — good design. But when writer B's CAS fails because writer A committed first, B **retries against A's new metadata and blindly re-applies its own `AddSnapshot`**. Because `assert-ref-snapshot-id` was never checked, nothing rejects the stale commit. The result is a snapshot whose `parent_snapshot_id` points at a pre-A snapshot, spliced onto a branch that has moved on: **forked snapshot lineage and orphaned data files under concurrent writers.** This is the most severe correctness defect in the codebase.

**A-2 (Critical) — Most commit updates are silently ignored, but return `200 OK`.** The `CommitUpdate` match at lines 630-672 handles `AddSnapshot`, `AddSchema`, and `SetCurrentSchema`, then `_ => {}`. Everything else — `set-properties`, `remove-properties`, `set-location`, `add-partition-spec`, `set-default-spec`, `add-sort-order`, `set-default-sort-order`, `remove-snapshots`, `set-snapshot-ref`, `assign-uuid`, `upgrade-format-version` — is discarded, and the handler returns `200 OK` with a fresh metadata file. A client running `ALTER TABLE ... SET TBLPROPERTIES`, evolving a partition spec, expiring snapshots, or creating a table-level branch/tag receives a success response for an operation that did not happen. Silent success is worse than failure: it cannot be retried or alerted on.

**A-3 (High) — `last_sequence_number` is assigned a snapshot ID.** Lines 645 and 651 set `metadata.last_sequence_number = snapshot_obj.snapshot_id`. In the Iceberg spec these are unrelated: sequence number is a small monotonic counter, snapshot ID is a random 64-bit value. Sequence numbers govern how position/equality delete files are applied to data files, so corrupting them can produce **incorrect query results on merge-on-read tables** — deletes silently applying to the wrong data, or not applying at all.

**A-4 (High) — The config endpoint ignores its own parameters.** `pangolin/pangolin_api/src/iceberg/config.rs:14`: `pub async fn get_iceberg_catalog_config_handler() -> Json<CatalogConfig>` takes **no arguments at all**. It ignores the `:prefix` path segment and the spec's `?warehouse=` query parameter, and builds `defaults` from process-wide environment variables (`S3_ENDPOINT`, `AWS_REGION`). `overrides` is always empty, so the server never returns the `prefix` that the spec uses to tell clients which path to address.

The irony is that Pangolin already models per-warehouse storage configuration — it just never surfaces it here. Every catalog in a multi-warehouse, multi-cloud deployment receives the same S3 endpoint and region. A tenant with an Azure warehouse gets the server's AWS settings.

**A-5 (Medium) — Missing spec endpoints.** Reading the router at `pangolin/pangolin_api/src/lib.rs:82-106`, the following spec operations have no route: `loadNamespaceMetadata` (GET `/v1/{prefix}/namespaces/{ns}` — only DELETE is wired), `namespaceExists` (HEAD), `registerTable` (POST `.../register`), `commitTransaction` (POST `/v1/{prefix}/transactions/commit`, required for multi-table atomic commits), and most of the view API — only `createView` and `loadView` exist, with no list, drop, replace, exists, or rename.

**A-6 (Medium) — Non-conforming error envelope.** The spec requires `{"error": {"message": ..., "type": ..., "code": ...}}`. `pangolin/pangolin_api/src/error.rs:73-75` emits a flat `{"error": "<string>"}`, and most Iceberg handlers bypass `ApiError` entirely, returning bare `(StatusCode, &str)` tuples with a plain-text body (e.g. `tables.rs:559`, `563`). Clients that parse the spec envelope to distinguish `NoSuchTableException` from `CommitFailedException` cannot do so, which breaks engine-side retry logic.

**A-7 (Low) — Route duplication as a compatibility workaround.** Nine Iceberg routes are registered twice, once under `/v1/:prefix/...` and again under `/v1/:prefix/v1/...`, with a comment explaining PyIceberg may double the prefix (`lib.rs:92-103`). This works but doubles the surface that must be secured and tested, and masks the underlying config-endpoint problem in A-4.

### 3.2 Authentication and authorization

**A-8 (Critical) — OAuth callback leaks session tokens to an attacker-controlled host.** At `pangolin/pangolin_api/src/oauth_handlers.rs:202-221`, the callback base64-decodes the `state` parameter, reads `redirect_uri` out of it, and appends the freshly minted session JWT as a query parameter:

```rust
let base_url = frontend_url.unwrap_or_else(|| std::env::var("FRONTEND_URL")...);
let redirect_url = format!("{}?token={}", base_url, token);
Redirect::to(&redirect_url).into_response()
```

The `state` value is plain base64 JSON — not signed, not encrypted, not stored server-side. There is no allowlist on `base_url`. The attack is direct: send a victim an authorize link whose `state` decodes to `{"redirect_uri":"https://evil.com/"}`; the victim authenticates with their real identity provider; Pangolin mints a valid JWT for them and 302-redirects the browser to `https://evil.com/?token=<victim's JWT>`, where it lands in the attacker's access log. **Full account takeover with no credential theft.**

**A-9 (Critical) — The OAuth `state` nonce is decorative.** Lines 262-268 generate `"nonce": Uuid::new_v4()` and embed it in `state`. Nothing ever stores or verifies it — a grep for `nonce` across `pangolin_api/src` and `pangolin_store/src` returns only the generation site. The callback never validates `state` at all. This is a textbook **login-CSRF** hole: an attacker can complete a flow with their own authorization code and silently bind the victim's browser to the attacker's account.

**A-10 (Critical) — The JWT signing secret has a working default.** `pangolin/pangolin_api/src/auth_middleware.rs:236` (and again at `:400`, and again at `oauth_handlers.rs:195`):

```rust
let secret = std::env::var("PANGOLIN_JWT_SECRET")
    .unwrap_or_else(|_| "default_secret_for_dev".to_string());
```

A deployment that forgets one environment variable starts successfully and validates tokens signed with a secret that is published in this repository. Anyone can forge a `Root` token. The server never warns, because the fallback is indistinguishable from success at boot.

This compounds with the Helm chart, which ships `PANGOLIN_JWT_SECRET: "change-me-please"` and `PANGOLIN_ROOT_PASSWORD: "password"` as **functional defaults** in `deployment_assets/helm/pangolin/values.yaml` — `helm install` with no overrides produces a running, publicly reachable deployment with a known signing key and a known root password. `pangolin_api/src/main.rs:73` adds a third default, `password123`, for the seeded admin.

**A-11 (High) — Auth bypass via path suffix.** `auth_middleware.rs:174` whitelists any request whose path ends in `/config`:

```rust
path == "/v1/config" || path.ends_with("/config") || path.contains("/oauth/tokens")
```

`ends_with` matches far more than the Iceberg config endpoint. A namespace literally named `config` produces `/v1/{prefix}/namespaces/config`, which ends in `/config` and therefore **skips authentication entirely** — including its DELETE route. A table named `config` yields `/v1/{p}/namespaces/{ns}/tables/config`, unauthenticated for GET, POST, DELETE, and HEAD. `contains("/oauth/tokens")` is looser still. Whitelisting must be exact-match or route-based, not substring-based.

**A-12 (High) — API-key authentication is O(tenants × service users) bcrypt calls per request.** `auth_middleware.rs:122-166` handles `X-API-Key` by listing **every tenant**, then **every service user in each tenant**, running `bcrypt::verify` against each until one matches. The code comments acknowledge it: *"This is not ideal for performance but works for MVP."*

bcrypt at default cost is deliberately ~100-250ms. At 100 service users, a single request burns ~25 CPU-seconds. This is not merely slow — it is an **unauthenticated denial-of-service primitive**, since the scan runs before any rate limiting (there is none) and before the public-endpoint whitelist. A handful of concurrent requests with a bogus API key will saturate every core. The fix is a lookup by key ID: give each API key a public prefix, index on it, and run exactly one bcrypt verification.

**A-13 (High) — Token revocation fails open.** `auth_middleware.rs:257-260`: if `is_token_revoked` returns `Err`, the code logs and continues — the comment says *"don't block on revocation check failure."* During a database blip, **every revoked token is accepted again**. Revocation is a security control; it must fail closed.

**A-14 (Medium) — Root password compared with `==`.** `auth_middleware.rs:196` uses `password == root_pass`, a non-constant-time comparison, against a plaintext password read from an environment variable. Use a constant-time comparison, and prefer a hashed value.

**A-15 (Medium) — Three divergent auth implementations.** `auth_middleware::auth_middleware` (production, wired at `lib.rs:211`), `auth_middleware::auth_middleware_wrapper` (~140 near-duplicate lines), and a third in `auth.rs:64`. They have already drifted: only the production one supports service-user API keys and token revocation.

This is worse than ordinary duplication, because **all seven auth-related integration tests exercise the wrapper, not production code** (`tests/auth_test.rs:103`, `isolation_test.rs:31,98,220`, `rbac_integration_test.rs:77`, `root_auth_tests.rs:28`, `business_metadata_test.rs:38`). The auth path that actually runs in production is not covered by any test.

**A-16 (Low) — Debug logging at ERROR level.** `auth_middleware.rs:339`: `tracing::error!("AUTH CHECK: Checking path '{}'", path)` fires on every request through the wrapper, and will trip any error-rate alert.

### 3.3 Observability

Verified absent by grep across the workspace:

| Capability | Status | Evidence |
|---|---|---|
| Metrics (Prometheus/OTel) | **None** | No `prometheus`, `metrics`, or `opentelemetry` dependency anywhere |
| Request tracing / access logs | **None** | `tower-http` is built with the `trace` feature but `TraceLayer` is never applied (`lib.rs:211-213`) |
| Request IDs / correlation | **None** | No propagation of a request or trace identifier |
| Structured (JSON) logs | **None** | `tracing_subscriber::fmt::init()` at `main.rs:15`, human-readable only |
| Log level control | **Broken** | See below |

**A-17 (High) — `RUST_LOG` is silently ignored.** The workspace declares `tracing-subscriber = "0.3"` with default features, and `env-filter` is **not** a default feature. Confirmed against `Cargo.lock`: the `tracing-subscriber` entry lists only `nu-ansi-term`, `sharded-slab`, `smallvec`, `thread_local`, `tracing-core`, `tracing-log`, and `matchers` (the `env-filter` marker crate) is absent from the lockfile entirely. So `fmt::init()` installs a fixed-level subscriber, and `RUST_LOG` has no effect.

This is not theoretical: `pangolin/Dockerfile:37` sets `ENV RUST_LOG=info` and `values.yaml` documents `RUST_LOG` as a tunable. **Operators believe they can change log verbosity, and they cannot.** During an incident there is no way to raise verbosity without a rebuild.

**A-18 (High) — No metrics means no SLOs.** There is no way to answer "what is p99 commit latency", "what is the table-commit conflict rate", "how full is the Postgres pool", or "how many 5xx in the last five minutes". The Helm chart has no `ServiceMonitor`. Every enterprise monitoring requirement in Area C depends on fixing this first.

### 3.4 Reliability and resource safety

**A-19 (High) — No graceful shutdown.** `main.rs:147-149`:

```rust
let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
axum::serve(listener, app).await.unwrap();
```

No `.with_graceful_shutdown()`, no `SIGTERM` handler (confirmed by grep — zero hits for `with_graceful_shutdown`, `ctrl_c`, or `SIGTERM`). On a Kubernetes rolling update, every in-flight request is severed mid-write. Given that table commits write a metadata file to object storage *before* the CAS (`tables.rs:678-683`), a SIGTERM in that window **leaks an orphaned metadata file** with no cleanup path. The Dockerfile also has no init process, so the binary runs as PID 1 and inherits the default no-op SIGTERM disposition.

**A-20 (High) — No request limits of any kind.** Grep confirms no `DefaultBodyLimit`, `RequestBodyLimitLayer`, `TimeoutLayer`, `ConcurrencyLimitLayer`, or rate limiting anywhere. A single large `POST` of table metadata can be buffered without bound; a slow backend request has no deadline. Combined with A-12, the server has no defence against either accidental or deliberate overload.

**A-21 (High) — `/health` is a string literal.** `lib.rs:76`: `.route("/health", get(|| async { "OK" }))`. It returns `200` whether or not the database is reachable. The Helm `readinessProbe` points at it (`deployment.yaml:71-74`), so **a pod whose database connection is dead stays in the Service and keeps receiving traffic**. There is no `/ready` distinct from `/live`, and no `livenessProbe` at all.

**A-22 (Medium) — Panic-prone startup and hot paths.** `main.rs` uses `.expect()` on store connection, schema application, and password hashing, and `.unwrap()` on bind and serve — a port conflict produces a panic and backtrace rather than a diagnostic. In request paths, `pangolin_api` contains **128 `unwrap()`** and 15 `expect()`; `pangolin_store` contains **155 `unwrap()`**, 40 `expect()`, and 5 `panic!`. Most consequential is `tables.rs:677`, `serde_json::to_string(&metadata).unwrap()`, directly in the commit path. Axum isolates a panicking task, but the client sees a connection reset rather than a structured error.

**A-23 (Medium) — Unbounded task spawning.** `auth_middleware.rs:151` fire-and-forgets a `tokio::spawn` per authenticated service-user request to update `last_used`, discarding the result. Under load this spawns unboundedly with no backpressure and silently drops failures.

### 3.5 Storage backends, transactions, and migrations

**A-24 (Critical) — The Postgres and MongoDB backends use zero transactions.** Verified by grep: `pangolin_store/src/postgres/` contains **no** `.begin()` or `.commit()` calls; `pangolin_store/src/mongo/` contains no `start_session` or `start_transaction`. Only SQLite uses a transaction, once.

For a catalog, this is a data-integrity problem at the core of the product. Multi-statement operations — merging a branch, creating a branch by copying assets, cascading a catalog delete, writing an asset plus its commit plus the branch-head update — are issued as independent statements. **Any failure or process death mid-sequence leaves the catalog in a partially-applied state** with no rollback and no repair tooling. Postgres is the recommended production backend.

**A-25 (High) — Backend parity is enforced at runtime, not compile time.** `pangolin_store/src/lib.rs` defines a 110-method `CatalogStore` trait in which a large fraction of methods carry default bodies returning `Err(anyhow!("Operation not supported by this store"))` (users, roles, permissions, business metadata, access requests, service users, federated catalogs — lines 196-300+).

Because a missing method compiles cleanly, backend gaps surface only as opaque runtime `500`s. The module listing shows this is already reality: `postgres/` has no `business_metadata.rs`, `federated.rs`, `merge.rs`, or `io.rs` module, while `memory/`, `mongo/`, and `sqlite/` do. There is no parity matrix in the docs telling an operator which features work on which backend.

**A-26 (High) — One default returns success instead of an error.** `lib.rs:268`, `search_assets`, defaults to `Ok(vec![])` rather than an error — and its parameters aren't underscore-prefixed, so it also emits unused-variable warnings. A backend that hasn't implemented search returns **"no results found"** instead of "not supported". Users conclude their data is missing.

**A-27 (Medium) — Two migration systems, one of them orphaned.** Postgres does it properly: `postgres/main.rs:57` runs `sqlx::migrate!("./migrations")` against 12 timestamped files in `pangolin/pangolin_store/migrations/`. But:

- SQLite has **no migrations**. `main.rs:31-33` applies the full `sql/sqlite_schema.sql` via `include_str!` on every boot, with no version table. Schema changes require manual intervention on existing databases.
- A **second, disconnected** migration tree exists at the repository root: `migrations/postgres/001_enhanced_audit_logging.sql` and `migrations/sqlite/001_...`. No runner references it. `pangolin_store/migrations/sqlite/` is likewise skipped by `sqlx::migrate!`, which only reads top-level `.sql` files.
- `sql/postgres_schema.sql` appears to be superseded by the migration chain but is still present.
- MongoDB has no schema or index management at all.

Three plausible sources of truth for the Postgres schema is a recipe for a production drift incident.

**A-28 (Medium) — The cache layer is node-local and unaware of peers.** `pangolin_api/src/cached_store.rs:27-28` wraps the store in a `moka` cache with a 60-second TTL for warehouses, which hold storage credentials. Invalidation (`:73`, `:81`) is in-process only. With `replicaCount > 1`, rotating a warehouse credential on node A leaves node B **vending the old credential for up to 60 seconds**. This directly contradicts horizontal scaling and needs either a much shorter TTL, a shared cache, or a cross-node invalidation channel.

There is a related structural hazard: `CachedCatalogStore` is a hand-written delegating wrapper implementing 112 methods. Any trait method a future contributor forgets to add falls through to the trait default — `Err("Operation not supported")` — silently disabling a working backend feature the moment caching is enabled. A `#[delegate]`-style macro or an explicit exhaustiveness test would remove the footgun.

**A-29 (Low) — SQLite pool is hardcoded.** `sqlite/main.rs:43` sets `max_connections(5)` with no environment override, while Postgres correctly exposes `DATABASE_MAX_CONNECTIONS` / `DATABASE_MIN_CONNECTIONS` / `DATABASE_CONNECT_TIMEOUT`.

### 3.6 Configuration management

**A-30 (High) — 88 scattered `env::var` calls, no typed config, no startup validation.** 39 distinct environment variables are read across `pangolin_api` and `pangolin_store`, with no central struct and no config-file support. Consequences:

- **Nothing is validated at boot.** Misconfiguration surfaces as a runtime failure or, worse, a silent insecure default (A-10).
- **Config is read per request.** `PANGOLIN_NO_AUTH` and `PANGOLIN_JWT_SECRET` are read from the environment on *every single request* (`auth_middleware.rs:89`, `:236`), as is `S3_ENDPOINT`/`AWS_REGION` on every config call.
- **Dead configuration.** `main.rs:19` reads `PANGOLIN_STORAGE_TYPE` into `storage_type` and never uses it — backend selection is driven entirely by the `DATABASE_URL` prefix. Yet `example.env`, `.env`, and `values.yaml` all document `PANGOLIN_STORAGE_TYPE` as the way to choose a backend. **The documented mechanism does nothing.**
- **Unsafe default backend.** With no `DATABASE_URL`, the server silently starts on `MemoryStore` (`main.rs:36-39`) — all catalog metadata is lost on restart, with only an info-level log line.

**A-31 (Medium) — `PANGOLIN_NO_AUTH` grants tenant-admin to anonymous callers.** `auth_middleware.rs:93-118`: when set, any request with no `Authorization` header receives a `TenantAdmin` session. There is a startup banner, which is good, but a single environment-variable typo in production disables all authentication. This should additionally refuse to start if it detects a non-loopback bind or a production-looking database URL.

**A-32 (Low) — CORS is wide open and not configurable.** `lib.rs:62-72` uses `allow_origin(Any)`, hardcoded, with the origin-specific line commented out just above it.

**A-33 (Low) — A PyPI publish token is stored in the runtime env file.** `.env` (correctly git-ignored and untracked — verified) contains `PYPI_token` alongside server runtime configuration. A package-publishing credential should not share a file that gets mounted into the application.

### 3.7 Deployment

**A-34 (High) — The Helm chart references three templates that do not exist.** `deployment_assets/helm/pangolin/templates/` contains only `_helpers.tpl`, `deployment.yaml`, `secret.yaml`, and `service.yaml`. Meanwhile:

- `deployment.yaml:27` sets `serviceAccountName` from a helper that, with the default `serviceAccount.create: true`, resolves to a ServiceAccount name — but **there is no `serviceaccount.yaml`**, so the account is never created and pods will fail admission in most clusters.
- `values.yaml` exposes a full `ingress:` block — **no `ingress.yaml`**.
- `values.yaml` exposes `autoscaling:` and `deployment.yaml:8` gates `replicas` on it — **no `hpa.yaml`**. Enabling autoscaling removes the replica count and creates no autoscaler, scaling the deployment to its default of one.

This has the signature of `helm create` scaffolding with templates deleted but values left behind. The chart should be assumed untested.

**A-35 (High) — Container and pod run as root.** `pangolin/Dockerfile` has no `USER` directive, and `values.yaml` ships `podSecurityContext: {}` and `securityContext: {}` with the hardening options present but commented out. No `runAsNonRoot`, no `readOnlyRootFilesystem`, no dropped capabilities.

**A-36 (Medium) — Dockerfile issues.** Base image is `rust:1.88-slim-bookworm` while the README requires Rust 1.92+ (drift). There is no dependency-caching layer, so every build recompiles the entire AWS/Azure/GCP/sqlx/mongodb graph. The runtime stage installs `libssl-dev` — a **development** package — instead of `libssl3`, adding headers and static libraries to the shipped image. No `--locked` on the build despite `Cargo.lock` being copied, no `HEALTHCHECK`, no image labels, no non-root user, no multi-arch build.

**A-37 (Medium) — No resource requests or limits.** `values.yaml` ships `resources: {}`. Pods land in the `BestEffort` QoS class and are first to be evicted under node pressure. No `PodDisruptionBudget`, and `replicaCount: 1`.

**A-38 (Medium) — `image.tag: "latest"`.** Non-reproducible deployments; no way to know what is running or to roll back deterministically.

---

## 4. Area B — Code Hygiene & Test Coverage

### 4.1 Formatting and lints

**B-1 — The tree is unformatted.** `cargo fmt --all -- --check` produces **2,722 diff hunks across 227 of 238 `.rs` files**. Formatting has effectively never been enforced. This is a one-command fix that should land as a single isolated commit, followed by CI enforcement — done in that order so the noise never has to be reviewed twice.

**B-2 — 314 unique clippy warnings.** By crate: `pangolin_store` 174, `pangolin_api` 105, `pangolin_cli_admin` 32, `pangolin_cli_user` 9, `pangolin_cli_common` 5, `pangolin_core` 2. Dominant themes:

| Count | Warning |
|---|---|
| ~70 | Unused imports (`async_trait::async_trait` ×22, `std::sync::Arc` ×10, `CatalogStore` ×10, `uuid::Uuid` ×6, `HashMap` ×6, …) |
| 20 | `std::io::Error::other(_)` modernization |
| 10 | Use of deprecated `AuditLogEntry::legacy_new` |
| 9 | Functions with too many arguments (8/7) |
| 6 | Collapsible `else { if .. }` |
| 6 | Manual prefix stripping |
| 5 | `assert_eq!` with a literal bool |
| — | Redundant clones, `Copy`-type clones, needless borrows, very complex types, modules named like their parent |

Clippy reports ~180 of these are machine-fixable via `cargo clippy --fix`.

**B-3 — Deprecated internal API still in use.** Ten call sites still use `AuditLogEntry::legacy_new` despite a deprecation attribute pointing at `AuditLogEntry::new()` / `::success()`. Audit-log construction is exactly where inconsistency causes compliance gaps.

**B-4 — AI-agent narration left in committed source.** Several files contain deliberation text from a code-generation session rather than explanatory comments:

- `pangolin_api/src/main.rs:126-134` — nine lines beginning *"The instruction seems to imply adding routes directly here… I will make no change to this file based on the provided snippet."*
- `pangolin_api/src/auth.rs:69-74` — *"This function seems to be Legacy or Unused… I will update it to be minimally compatible… So I MUST update it to use `UserRole`."*
- `pangolin_store/Cargo.toml`, in the `[features]` block — eight lines of first-person reasoning: *"Wait, if I make deps optional, code using them must be conditional… I will NOT mark AWS deps optional yet to avoid massive refactor."*

Also present: duplicated comment lines (`// Run it` twice in `main.rs`, `// Get JWT secret from environment` twice at `auth_middleware.rs:234-235`, `// Parse metadata in blocking task` twice at `tables.rs:614-615`) and a 15-line commented-out block of dead routes at `lib.rs:194-200`. For an open-source project this is a credibility issue as much as a hygiene one — it is the first thing a prospective contributor reads.

**B-5 — Repository is polluted with build and debug artifacts.** Of 865 tracked files, **35+ are committed debug output**: `check_output.txt`, `check_output_2/3/4.txt`, `check_output_clean{,_2,_final}.txt`, `check_output_final{,_2,_3}.txt`, `check_tokens{,_2,_3,_4}.txt`, `api_check_errors{,_2,_3}.txt`, `check_log.txt`, `test_results.txt`, `tests/debug_outputs/*`, and — notably — a file literally named `pangolin/pangolin_store/:memory:`, created by an SQLite connection string being passed as a file path. Additional untracked-but-present clutter includes `server.log`, `curl_verbose.log`, `error*.log`, `compile_error*.log`, and `bulk_test.db`. The `.gitignore` already covers most of these patterns; the files predate it and were never removed.

**B-6 — `unsafe`: none.** Zero occurrences workspace-wide. Worth stating explicitly and worth locking in with `#![forbid(unsafe_code)]` at each crate root so it stays true.

**B-7 — Test helpers ship in the production binary.** `lib.rs:33` declares `pub mod tests_common;` unconditionally, while `verification_tests` and `audit_tests` are correctly `#[cfg(test)]`. Test fixtures should not be compiled into the release artifact.

**B-8 — Dependency hygiene.** No `cargo audit` or `cargo deny` anywhere, and no advisory scanning in CI. Some drift: `pangolin_api` uses `reqwest` 0.11 while `pangolin_store` uses 0.12, so both are compiled in. `aws-sdk-sts` is pinned to `=1.50.0` in `pangolin_api` but floats at `1.0` in `pangolin_store`. `utoipa-swagger-ui` v6 is paired with `utoipa` v4. Several `thiserror`/`anyhow` versions are 1.x while 2.x is current. No `[workspace.package]` block, so versions and metadata are duplicated per crate.

**B-9 — Error-type design is inconsistent.** `ApiError` (`error.rs`) is a reasonable design, but it is used in only **5 of 46** files in `pangolin_api` — `tenant_handlers`, `dashboard_handlers`, `iceberg/oauth`, `optimization_handlers`, and its own definition. Everything else returns `impl IntoResponse` with ad-hoc `(StatusCode, &str)` tuples; there are 87 such sites and 400+ raw `StatusCode::` references. The storage layer compounds this by returning `anyhow::Error` throughout the `CatalogStore` trait, so the API layer cannot distinguish "not found" from "constraint violation" from "connection lost" without string matching — which is why so many handlers collapse every failure into `500 Internal Server Error` (e.g. `tables.rs:559`, `:563`, `:568`). A typed `StoreError` enum in `pangolin_store` is the prerequisite for accurate HTTP status codes.

**B-10 — Documentation comments are sparse.** No `#![warn(missing_docs)]` on any crate. The 110-method `CatalogStore` trait — the primary extension point, and the subject of a dedicated docs page — carries almost no doc comments describing invariants, error semantics, or which methods a new backend must implement.

### 4.2 Test coverage

**B-11 (Critical) — `cargo test --workspace` executes zero tests.** Even with `--no-fail-fast`, the run aborts during compilation. Five test targets fail to build:

| Target | Errors |
|---|---|
| `pangolin_api` (lib test) | 6 — `Permission` missing `tenant_id` (`asset_handlers.rs:481`, `authz_utils.rs:201,219,238,286`); arity change (`authz.rs:205`) |
| `pangolin_store` `postgres_tests` | 4 — `ServiceUser` missing `last_used`; `Role` missing `created_by`; arity change; `UserRole` has no field `id` |
| `pangolin_store` `postgres_comprehensive_tests` | 1 — arity change at `:460` |
| `pangolin_store` `sqlite_comprehensive_tests` | 1 — arity change at `:552` |
| `pangolin_store` `mongo_comprehensive_tests` | 1 — arity change at `:401` |

Every one is the same story: a struct gained a field or a function gained a parameter, production code was updated, test code was not, and **nothing in the project ever compiled the test code again**. The `pangolin_api` lib-test failure alone disables all 39 in-source unit tests, including `verification_tests` and `audit_tests`.

**B-12 (High) — Actual measured pass rate.** Running the targets that do compile:

| Target | Result |
|---|---|
| `pangolin_core --lib` | 6 passed |
| `pangolin_store --lib` | 14 passed, **27 failed** (Postgres/Mongo — require live databases, environmental) |
| `pangolin_api` `rest_spec_tests` | 6 passed |
| `pangolin_api` `iceberg_endpoints_tests` | 14 passed |
| `pangolin_api` `api_tests` | 1 passed |
| `pangolin_api` `auth_test` | 1 passed |
| `pangolin_api` `rbac_integration_test` | 1 passed |
| `pangolin_api` `test_pagination` | 1 passed |
| `pangolin_api` `merge_tests` | **1 failed** |
| `pangolin_api` `isolation_test` | **2 failed** |

Roughly **44 tests actually execute and pass** without external infrastructure, against ~304 declared test functions.

**B-13 (Critical) — The tenant-isolation tests fail.** `isolation_test::test_warehouse_isolation` and `::test_catalog_isolation` both panic:

```
called `Result::unwrap()` on an `Err` value: Error("expected value", line: 1, column: 1)
   at pangolin_api/tests/isolation_test.rs:110  /  :232
```

These use `MemoryStore` (`isolation_test.rs:23,39,161`) and need nothing external, so this is a real, deterministic failure. The serde error means the login endpoint returned a non-JSON body — the test logs in as `user_a` and unwraps the response as JSON, and the request is failing. Multi-tenant isolation is Pangolin's headline enterprise property; its tests are currently red. (Note the caveat from A-15: these tests exercise `auth_middleware_wrapper`, so even when green they would not validate the production auth path.)

**B-14 (High) — Coverage is thin where risk is highest, and test names oversell.** `rest_spec_tests.rs`, despite the name, contains six tests covering only the config endpoint, tenant-header propagation, and root basic auth — **no Iceberg spec conformance at all**: no namespace CRUD, no table create/commit/rename, no commit-requirement conflict testing. Several impressively-named files contain exactly one test (`api_tests`, `auth_test`, `rbac_integration_test`, `test_pagination`). Nothing exercises:

- Concurrent commits to the same table (the A-1 lost-update scenario)
- Commit requirement enforcement or conflict responses
- Credential vending scope correctness
- Cross-tenant access denial through the **production** middleware
- Service-user API key authentication
- Token revocation
- Graceful degradation when the backend is unavailable

There are no property-based tests (no `proptest`/`quickcheck`) and no fuzzing, despite the commit path being a natural fit for both.

**B-15 (High) — Backend tests require live databases with no fallback.** The 27 store failures are connection errors. There is no `testcontainers` integration, so `cargo test` cannot pass on a clean checkout. `docker-compose.db-test.yml` and `docker-compose.emulators.yml` exist but are not wired to the test run, and there is no documented one-command path from clone to green.

**B-16 — Coverage is currently unmeasurable.** Neither `cargo-llvm-cov` nor `cargo-tarpaulin` is installed, and measuring is pointless until B-11 is fixed. Based on the executable-test inventory, true line coverage is likely **under 20%**, concentrated in `pangolin_store` CRUD paths. A first real number should be taken immediately after Phase 2.1.

---

## 5. Area C — Enterprise Readiness

### 5.1 Authentication, authorization, and identity

**What exists:** JWT sessions with revocation and rotation; bcrypt password hashing; a `Root`/`TenantAdmin`/user role hierarchy; a granular RBAC model (`pangolin_core/src/permission.rs`, `pangolin_api/src/authz.rs`) with roles, permissions, scopes, and actions; service users with API keys and expiry; OAuth 2.0 for Google, Microsoft, GitHub, and Okta.

That is a genuinely strong feature set. The problems are in the implementation and the verification.

**C-1 (Critical) — Fix the authentication defects in §3.2 before any enterprise conversation.** A-8 through A-14 are blockers. An enterprise security review will find the OAuth token leak immediately.

**C-2 (High) — OAuth is not OIDC.** The flow uses the authorization-code grant and then calls the provider's userinfo endpoint. There is no PKCE, no `id_token` validation, no JWKS fetching or signature verification, no `iss`/`aud` checking, and no `nonce` binding. There is also no discovery document support, so onboarding an arbitrary enterprise IdP requires a code change per provider (`get_oauth_config` hardcodes four). SAML is absent. Enterprise buyers will ask for generic OIDC discovery, SCIM provisioning, and group-to-role mapping — none exist.

**C-3 (High) — Users are auto-provisioned by email with no verification.** The callback finds or creates a user keyed on the email returned by the provider, with no `email_verified` check and no domain allowlist. With a provider that permits unverified emails, this is an account-takeover path. The code comments acknowledge the design is provisional: *"Ideally look up by (provider, subject)"* — which is the correct fix, since email is mutable and subject is not.

**C-4 (Medium) — No JWT key rotation.** A single symmetric HS256 secret, read per request from the environment, with no key ID (`kid`) in the header and no support for overlapping keys. Rotating the secret invalidates every session simultaneously. Enterprises will expect asymmetric signing (RS256/EdDSA) with a published JWKS and graceful rotation.

**C-5 (Medium) — No password or session policy.** No complexity requirements, no expiry, no history, no lockout or throttling on failed logins (which also makes the login endpoint brute-forceable, since there is no rate limiting anywhere), no MFA, no configurable session lifetime. The seeded admin token is issued with a **365-day** lifetime (`main.rs:99`) and printed to stdout, where it will be captured by container log aggregation.

### 5.2 Multi-tenancy

**What exists:** A `TenantId` extension injected by middleware and threaded through nearly every `CatalogStore` method as a required parameter — a sound design that makes it hard to forget the tenant scope.

**C-6 (Critical) — The isolation guarantee is unverified.** Per B-13, the two tenant-isolation tests fail. This is the property most likely to be probed in a security review or a customer POC, and there is currently no passing evidence for it.

**C-7 (High) — Root tenant override is unaudited.** `auth_middleware.rs:287-296` lets any `Root` session set `X-Pangolin-Tenant` to impersonate any tenant. This is a legitimate administrative capability, but it produces no audit event and there is no break-glass workflow. Cross-tenant access by a privileged operator is exactly what compliance auditors want logged.

**C-8 (Medium) — No per-tenant quotas or isolation of resources.** No limits on catalogs, namespaces, assets, request rate, or storage per tenant. A single tenant can exhaust the connection pool or the API for everyone. There is no notion of tenant tiers, and no per-tenant encryption keys.

**C-9 (Medium) — Nil-UUID as the default tenant is fragile.** `00000000-0000-0000-0000-000000000000` is hardcoded in at least six places, and sessions without a tenant fall back to it (`auth_middleware.rs:283-285`). A bug that drops the tenant claim silently routes a request into the default tenant rather than failing closed.

### 5.3 Security and secrets

**C-10 (High) — Secrets are plaintext environment variables end to end.** No integration with Vault, AWS Secrets Manager, Azure Key Vault, GCP Secret Manager, or the Kubernetes CSI secrets driver. The Helm chart base64-encodes values from `values.yaml` into a `Secret` (`templates/secret.yaml`), which encourages committing production credentials to a values file in Git. There is no External Secrets support and no rotation story.

**C-11 (High) — Warehouse cloud credentials are stored in the catalog database.** Warehouse objects carry storage credentials and are cached in process (A-28). There is no envelope encryption at rest, and a database backup therefore contains cloud credentials in the clear.

**C-12 (Medium) — No TLS termination in-process.** The server binds plain HTTP on `0.0.0.0` (`main.rs:145`). That is a defensible choice when a load balancer terminates TLS, but there is no mTLS option and no documented requirement — and combined with tokens-in-URLs (A-8), plaintext transit is a real exposure.

**C-13 (Medium) — No security policy or disclosure path.** No `SECURITY.md`, no advisory process, no CVE handling, no `cargo audit` in CI, no SBOM, and no signing or checksums on released binaries (`build-binaries.yml` uploads raw artifacts). Enterprises will ask for all of these.

### 5.4 High availability and scalability

**C-14 (High) — No verified horizontal-scaling story.** `replicaCount: 1`, no PDB, no HPA template (A-34), no leader election, no distributed locking. Two specific correctness barriers to running N > 1:

- The node-local warehouse cache (A-28) serves stale credentials for up to 60s after a peer's update.
- `cleanup_job.rs` runs a background token-cleanup task **in every replica** with no coordination, so N replicas run N concurrent cleanups.

**C-15 (High) — No backpressure or capacity controls.** No connection limits, request timeouts, body limits, or rate limiting (A-20). No documented capacity model, no load-test results, no published throughput or latency figures. `docs/best-practices/scalability.md` exists but has no measurements behind it.

**C-16 (Medium) — No backup, restore, or disaster recovery.** No documented backup procedure, no point-in-time recovery guidance, no restore runbook, no tested RPO/RTO, and no export/import tooling for catalog metadata. For a system that is the authoritative index of a data lake, losing the catalog means losing the ability to read the lake. This is a top-three enterprise requirement and it is entirely absent.

**C-17 (Medium) — Non-atomic operations have no repair path.** Given A-24 (no transactions), there is no consistency checker, no orphaned-metadata-file reaper (A-19), and no reconciliation tool to detect or repair a catalog left in a partial state.

### 5.5 Audit logging and compliance

**What exists:** A well-developed audit model — `pangolin_core/src/audit.rs` (466 lines) covering 40+ actions across 19 resource types, with filtering, counting, and per-backend implementations, plus a partitioning-prep migration for Postgres. This is one of the stronger parts of the codebase.

**C-18 (High) — Audit writes are best-effort and silently dropped.** In the commit path, `let _ = store.log_audit_event(...)` (`tables.rs:685`) discards the result. If the audit write fails, the operation still succeeds and no record exists. For SOC 2 or HIPAA, audit logging must be reliable — buffered and retried, or fail-closed on the operation.

**C-19 (High) — Coverage gaps in what gets audited.** No audit events for authentication attempts (success or failure), token issuance or revocation, permission checks that were denied, root tenant impersonation (C-7), or configuration changes. Auth events are the first thing an incident responder looks for.

**C-20 (Medium) — No tamper-evidence or retention controls.** Audit records live in the same database as application data, writable through the same credentials, with no hash chaining or signing, no WORM/immutable storage option, no configurable retention or legal hold, and no export to an external SIEM (Splunk, Datadog, S3+Athena). No compliance-oriented documentation exists for SOC 2, GDPR (no data-subject deletion workflow), or HIPAA.

### 5.6 Monitoring and alerting

**C-21 (High) — Nothing to monitor with.** Per A-17/A-18: no metrics endpoint, no `ServiceMonitor`, no dashboards, no alert rules, no SLO definitions, no error budgets, no distributed tracing, and no runbooks. An operator running Pangolin today has a log stream whose level they cannot change.

### 5.7 Release, versioning, and packaging

**C-22 (High) — No CI on push or pull request.** The single workflow is tag-triggered release building. Nothing verifies a commit compiles, passes tests, is formatted, is lint-clean, or is free of known-vulnerable dependencies. Every finding in this document is a direct or indirect consequence.

**C-23 (Medium) — Version drift across five artifacts.** `pangolin_api` 0.5.1, `pangolin_core`/`pangolin_store` 0.2.0, CLIs 0.5.0, `pypangolin` 0.5.1, UI 0.5.0, Helm chart 0.1.0/appVersion 0.1.0, and the docs refer to a "v0.4.0" known-issues section. There is no `[workspace.package]` version, no `CHANGELOG.md`, no release notes, no documented compatibility matrix between server, SDK, CLI, and UI, and no stated API stability or deprecation policy.

**C-24 (Medium) — Release pipeline gaps.** No Docker image build or publish in CI (despite `values.yaml` pointing at `alexmerced/pangolin-api:latest`), no multi-arch images, no checksums, no signing (cosign/sigstore), no SBOM, no provenance attestation, and no dependency caching — four runners rebuild the entire dependency graph from scratch on every tag.

**C-25 (Medium) — No contributor governance.** No `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue or PR templates, or architecture decision records — against 134 user-facing documentation files. The user docs are a real asset; the contributor path is missing entirely, which is a problem for an MIT-licensed project seeking adoption.

**C-26 (Medium) — Operator documentation lacks the operational core.** The docs cover features well but have no production runbook, no incident-response procedures, no upgrade or rollback guide, no capacity-planning guidance, no backend feature-parity matrix (A-25), and no security-hardening checklist. `docs/best-practices/deployment.md` exists but predates most of these findings.

---

## 6. Phased Execution Roadmap

Effort key: **S** ≈ 1-3 days · **M** ≈ 1-2 weeks · **L** ≈ 3+ weeks (one engineer).

Sequencing rationale: Phase 0 exists because **fixing the build gate first is what makes every later phase stick** — and because the security items are actively exploitable. Phase 1 hardens what already exists. Phase 2 pays down debt and rebuilds confidence. Phase 3 addresses enterprise requirements, which mostly depend on Phase 1 foundations (you cannot alert without metrics, or claim compliance without reliable audit).

### Phase 0 — Stop the bleeding (Weeks 1-2)

Nothing else should start until these land.

| # | Item | Description | Rationale | Effort | Depends on |
|---|---|---|---|---|---|
| 0.1 | **Fix OAuth token exfiltration** | Validate `redirect_uri` against a server-side allowlist; drop `redirect_uri` from `state` entirely. Deliver the token via a short-lived one-time code exchanged over POST, or a `Secure`/`HttpOnly`/`SameSite` cookie — never a query parameter. (`oauth_handlers.rs:202-221`) | A-8: remote account takeover | S | — |
| 0.2 | **Enforce OAuth `state`** | Store the nonce server-side (or HMAC-sign `state`) and verify it on callback; reject mismatches. Add PKCE. (`oauth_handlers.rs:262-268`) | A-9: login CSRF | S | 0.1 |
| 0.3 | **Remove all insecure defaults** | Fail startup if `PANGOLIN_JWT_SECRET` is unset or matches a known default. Remove `default_secret_for_dev` (3 sites), `password123` (`main.rs:73`), and the Helm `change-me-please`/`password` values. Generate a random secret in dev only, and log loudly. | A-10: forgeable Root tokens | S | — |
| 0.4 | **Fix the auth whitelist** | Replace `path.ends_with("/config")` and `path.contains("/oauth/tokens")` with exact-match or route-level public marking. (`auth_middleware.rs:168-179`, `:337-350`) | A-11: auth bypass via resource name | S | — |
| 0.5 | **Fix API-key lookup** | Add a public key-ID prefix to API keys, index it, and do one indexed lookup plus one bcrypt verify. (`auth_middleware.rs:122-166`) | A-12: unauthenticated DoS | M | — |
| 0.6 | **Make revocation fail closed** | Return `401` when the revocation check errors; add a metric for check failures. (`auth_middleware.rs:257-260`) | A-13: revoked tokens accepted | S | — |
| 0.7 | **Enforce commit requirements** | Implement all `CommitRequirement` variants — especially `assert-ref-snapshot-id` — and return `409` on mismatch. Replace `_ => {}` with an explicit error for unknown variants. (`tables.rs:600-628`) | A-1: lost updates, forked lineage | M | — |
| 0.8 | **Reject unsupported commit updates** | Replace the `_ => {}` at `tables.rs:672` with a `422`/`501` listing unsupported update types. Correctness first; implement the missing updates in 1.9. | A-2: silent data loss behind `200 OK` | S | — |
| 0.9 | **Fix `last_sequence_number`** | Maintain a proper monotonic counter; stop assigning snapshot IDs. Add a migration/repair note for tables already written. (`tables.rs:645,651`) | A-3: incorrect MOR query results | S | — |
| 0.10 | **Stand up CI** | GitHub Actions on push and PR: `cargo build`, `cargo test --workspace`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`. Use `Swatinem/rust-cache`. Start `clippy`/`fmt` non-blocking, flip to blocking after Phase 2. | C-22: root cause of nearly everything | S | 2.1 for tests to pass |
| 0.11 | **Fix the broken test targets** | Update the 13 compilation errors across 5 targets so `cargo test --workspace` runs. | B-11: prerequisite for 0.10 | S | — |
| 0.12 | **Fix the tenant-isolation tests** | Diagnose the login failure in `isolation_test.rs:110,232` and get both green — against the **production** middleware, not the wrapper. | B-13/C-6: headline guarantee unverified | S | 0.11 |
| 0.13 | **Publish `SECURITY.md`** | Disclosure policy, contact, supported versions. | C-13 | S | — |

### Phase 1 — Production hardening (Weeks 3-8)

| # | Item | Description | Rationale | Effort | Depends on |
|---|---|---|---|---|---|
| 1.1 | **Graceful shutdown** | `axum::serve(...).with_graceful_shutdown(...)` on SIGTERM/SIGINT with a drain deadline; add `tini` to the image; set `terminationGracePeriodSeconds`. | A-19: severed commits, orphaned metadata | S | — |
| 1.2 | **Real health endpoints** | Split `/health/live` (process) from `/health/ready` (store round-trip + pool status). Point the Helm readiness probe at `/health/ready` and add a liveness probe. | A-21: dead pods keep serving | S | — |
| 1.3 | **Request middleware stack** | `TraceLayer` with request IDs, `TimeoutLayer`, `DefaultBodyLimit`, `ConcurrencyLimitLayer`, and per-IP/per-token rate limiting on auth endpoints. | A-20: no overload or brute-force defence | M | — |
| 1.4 | **Enable `env-filter` + JSON logs** | Add the `env-filter` feature to `tracing-subscriber`; build the subscriber explicitly with `EnvFilter::from_default_env()`; add a `LOG_FORMAT=json` mode. | A-17: `RUST_LOG` silently ignored | S | — |
| 1.5 | **Prometheus metrics** | `/metrics` with RED metrics per route, commit success/conflict/retry counters, pool gauges, cache hit rates, auth outcomes. Add a `ServiceMonitor` to the chart. | A-18/C-21: nothing is measurable | M | 1.3 |
| 1.6 | **Typed configuration** | One `Config` struct parsed and validated once at startup (`figment`/`config` + `serde`), supporting env and file. Remove all per-request `env::var`. Fail fast with actionable messages. Delete or implement `PANGOLIN_STORAGE_TYPE`. | A-30: 88 scattered reads, dead config | M | 0.3 |
| 1.7 | **Transactions in Postgres and Mongo** | Wrap every multi-statement operation (merge, branch create, cascading delete, asset+commit+branch-head) in a transaction/session. | A-24: partial writes corrupt the catalog | L | — |
| 1.8 | **Typed `StoreError` + consistent `ApiError`** | Replace `anyhow` in the `CatalogStore` trait with a typed error enum; map it to correct HTTP statuses; migrate all 46 API files onto `ApiError`; emit the Iceberg error envelope on `/v1/*`. | B-9/A-6: everything is a 500 | L | — |
| 1.9 | **Complete the Iceberg surface** | Implement the missing commit updates (properties, specs, sort orders, refs, snapshot removal), plus `loadNamespaceMetadata`, `namespaceExists`, `registerTable`, `commitTransaction`, and the full view API. | A-2/A-5: spec claim is unsupported | L | 0.8 |
| 1.10 | **Per-warehouse config endpoint** | Give the handler its `prefix`/`warehouse` parameters; return per-warehouse `defaults`/`overrides` including `prefix`; remove the duplicated `/v1/:prefix/v1/...` routes once clients resolve correctly. | A-4/A-7: wrong storage config in multi-cloud | M | 1.6 |
| 1.11 | **Orphaned-metadata cleanup** | Track metadata files written before a failed CAS and reap them; extend the existing cleanup job. | A-19/C-17: unbounded storage growth | M | 1.1 |
| 1.12 | **Unify migrations** | Delete the orphaned root `migrations/` tree and the superseded `sql/postgres_schema.sql`; put SQLite on `sqlx::migrate!`; add index/collection management for Mongo; document the upgrade path. | A-27: three sources of schema truth | M | — |
| 1.13 | **Cache coherence** | Cut the warehouse TTL, and add cross-node invalidation (Postgres `LISTEN/NOTIFY` or Redis) or make credential reads write-through. | A-28: stale credentials across replicas | M | 1.5 |
| 1.14 | **Harden the container** | Non-root `USER`, `cargo-chef` caching, `libssl3` instead of `libssl-dev`, `--locked`, `HEALTHCHECK`, OCI labels, multi-arch, base image aligned to the documented Rust version. | A-35/A-36 | S | 1.2 |
| 1.15 | **Fix the Helm chart** | Add the missing `serviceaccount.yaml`, `ingress.yaml`, and `hpa.yaml`; set default resource requests/limits; enable `runAsNonRoot`, `readOnlyRootFilesystem`, dropped capabilities; add a PDB; pin `image.tag`; add `helm lint`/`helm template` to CI. | A-34/A-35/A-37/A-38: chart is broken as shipped | M | 1.14 |

### Phase 2 — Hygiene and test coverage (Weeks 6-12, overlaps Phase 1)

| # | Item | Description | Rationale | Effort | Depends on |
|---|---|---|---|---|---|
| 2.1 | **Format the tree** | One isolated commit: `cargo fmt --all`. Add `rustfmt.toml`. Record the SHA in `.git-blame-ignore-revs`. | B-1: 227/238 files unformatted | S | 0.11 |
| 2.2 | **Clear clippy** | Apply the ~180 auto-fixes, then hand-fix the rest. Add `#![forbid(unsafe_code)]` and workspace lint config. Flip CI to `-D warnings`. | B-2: 314 warnings hide real bugs | M | 2.1, 0.10 |
| 2.3 | **Purge artifacts and narration** | Remove the 35+ committed debug files and `:memory:`; strip the AI-agent deliberation comments from `main.rs`, `auth.rs`, and `pangolin_store/Cargo.toml`; delete dead commented-out routes; drop `legacy_new` call sites. | B-4/B-5/B-3: contributor-facing credibility | S | — |
| 2.4 | **Consolidate auth middleware** | Delete `auth.rs::auth_middleware` and `auth_middleware_wrapper`; point all tests at the production middleware. | A-15: tests don't cover production auth | M | 0.12 |
| 2.5 | **Hermetic backend tests** | `testcontainers` for Postgres and Mongo so `cargo test` passes on a clean checkout with only Docker. | B-15: 27 tests unrunnable locally and in CI | M | 0.11 |
| 2.6 | **Real Iceberg conformance suite** | Replace the misnamed `rest_spec_tests.rs` with genuine coverage: namespace and table CRUD, every commit requirement and update, error envelopes, and the missing endpoints. Add the official Iceberg REST compatibility checks and a PyIceberg matrix job. | B-14/A-1/A-2: the core product is untested | L | 1.9, 2.5 |
| 2.7 | **Concurrency and property tests** | A test that races N concurrent commits against one table and asserts linearizable snapshot lineage — the direct regression test for A-1. Add `proptest` for metadata serialization round-trips and merge conflict detection. | A-1: highest-severity correctness bug | M | 0.7 |
| 2.8 | **Security test suite** | Cross-tenant denial through production middleware, service-user API keys, token revocation, root impersonation, the `/config` bypass, and OAuth state/redirect validation. | A-8/A-11/C-6 | M | 2.4 |
| 2.9 | **Coverage measurement and target** | Add `cargo-llvm-cov` to CI with Codecov reporting. Establish a baseline immediately after 0.11, then ratchet: **50%** by end of Phase 2, **70%** by end of Phase 3, with `pangolin_api/src/iceberg/`, `auth_middleware.rs`, and `authz.rs` held to **85%**. | B-16: currently unmeasurable | S | 0.11, 0.10 |
| 2.10 | **Module and API structure** | Split `postgres/main.rs` (1,598 lines) and `iceberg/tables.rs` (930 lines). Consider narrowing the 110-method `CatalogStore` god-trait into composable traits (`TenantStore`, `AssetStore`, `AuthStore`, `AuditStore`) so backend parity becomes a compile-time property. Replace hand-written `CachedCatalogStore` delegation with a macro. | A-25/A-26: runtime-only parity | L | 1.8 |
| 2.11 | **Documentation comments** | `#![warn(missing_docs)]` on public crates; document `CatalogStore` invariants and error semantics; add `cargo doc` to CI. | B-10 | M | 2.10 |
| 2.12 | **Dependency hygiene** | Add `[workspace.package]`; unify `reqwest` on 0.12 and the AWS SDK pins; upgrade `thiserror`/`anyhow` to 2.x; align `utoipa`; add `cargo-deny` for licenses and advisories; enable Dependabot. | B-8/C-13 | M | 0.10 |
| 2.13 | **Move test helpers behind `cfg(test)`** | `tests_common` should not ship in the release binary. | B-7 | S | — |

### Phase 3 — Enterprise readiness (Weeks 12-24)

| # | Item | Description | Rationale | Effort | Depends on |
|---|---|---|---|---|---|
| 3.1 | **Proper OIDC** | Discovery documents, JWKS validation, `id_token` verification with `iss`/`aud`/`nonce`, PKCE, generic provider config. Look users up by `(provider, subject)`, not email; add domain allowlisting and `email_verified` enforcement. | C-2/C-3 | L | 0.1, 0.2 |
| 3.2 | **Asymmetric JWTs with rotation** | RS256/EdDSA, `kid` headers, published JWKS, overlapping keys for zero-downtime rotation. | C-4 | M | 1.6 |
| 3.3 | **Account and session policy** | Password policy, failed-login lockout, configurable session lifetime, refresh tokens, MFA/TOTP. Remove the 365-day startup token. | C-5 | M | 1.3 |
| 3.4 | **Secrets manager integration** | Vault / AWS Secrets Manager / Azure Key Vault / GCP Secret Manager plus Kubernetes CSI; External Secrets support in the chart; envelope encryption for warehouse credentials at rest. | C-10/C-11 | L | 1.6 |
| 3.5 | **Reliable, tamper-evident audit** | Never drop audit writes (buffer + retry, or fail the operation). Add auth attempts, token lifecycle, denied permission checks, root impersonation, and config changes. Add hash chaining, retention/legal hold, and SIEM export. | C-18/C-19/C-20/C-7 | L | 1.7 |
| 3.6 | **Verified HA** | Coordinate the background cleanup job across replicas (leader election or advisory lock); prove N>1 correctness under load; ship PDB + HPA defaults; document the scaling model with real numbers. | C-14 | L | 1.13, 1.15 |
| 3.7 | **Backup, restore, and DR** | Documented and *tested* backup/restore per backend, metadata export/import tooling, a catalog consistency checker and repair tool, published RPO/RTO, and a DR runbook. | C-16/C-17 | L | 1.7 |
| 3.8 | **Tenant quotas and isolation** | Per-tenant limits on objects, request rate, and storage; tenant tiers; connection-pool fairness. | C-8 | M | 1.3, 1.5 |
| 3.9 | **Monitoring and SLOs** | Grafana dashboards, Prometheus alert rules, defined SLIs/SLOs (availability, p99 commit latency, conflict rate), error budgets, and per-alert runbooks. | C-21 | M | 1.5 |
| 3.10 | **Release engineering** | Unified `[workspace.package]` versioning, `CHANGELOG.md`, semantic-versioning and API-stability policy, multi-arch Docker publish, checksums, cosign signing, SBOM, provenance, and a published server/SDK/CLI/UI compatibility matrix. | C-23/C-24 | M | 0.10 |
| 3.11 | **Operator documentation** | Production runbook, incident response, upgrade/rollback guide, capacity planning, security-hardening checklist, and a **backend feature-parity matrix**. | C-26/A-25 | M | 3.7, 3.9 |
| 3.12 | **Contributor governance** | `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, issue/PR templates, ADRs, and an architecture guide. | C-25 | S | — |
| 3.13 | **Compliance groundwork** | SOC 2 control mapping, GDPR data-subject deletion workflow, data-residency options, encryption-at-rest documentation, and a penetration test. | C-20 | L | 3.4, 3.5 |
| 3.14 | **Performance baseline** | Load-test harness, published throughput and latency figures, `criterion` benchmarks on the commit path, and regression detection in CI. | C-15 | M | 1.5, 3.6 |

---

## 7. Do-First Shortlist — Ranked by Impact vs. Effort

The top eight are all **S** or **M** effort against **critical** impact. They are the clearest wins available.

| Rank | Item | Impact | Effort | Why this order |
|---|---|---|---|---|
| 1 | **0.1 + 0.2** Fix OAuth token exfiltration and enforce `state` | Critical | S | Remotely exploitable account takeover with no credential theft required. Nothing else matters if this ships. |
| 2 | **0.3** Remove default JWT secret and passwords | Critical | S | A single missing env var makes every deployment forgeable, and the Helm defaults are functional. Three sites plus a values file. |
| 3 | **0.11** Fix the 5 broken test targets | Critical | S | 13 mechanical compile errors. Unblocks CI, coverage, and every regression test below. Highest leverage per hour in the document. |
| 4 | **0.10** Stand up CI on push/PR | Critical | S | The root cause. Without it, items 1-3 regress silently, exactly as the test suite did. |
| 5 | **0.4** Fix the `ends_with("/config")` whitelist | Critical | S | A namespace or table named `config` is unauthenticated. One-line class of fix. |
| 6 | **0.7** Enforce `assert-ref-snapshot-id` | Critical | M | Concurrent writers currently fork snapshot lineage and orphan data. The worst *silent* bug — no error is ever surfaced. |
| 7 | **0.8** Reject unsupported commit updates | Critical | S | Turns silent data loss into an honest error. Small change, large trust gain, buys time for 1.9. |
| 8 | **0.12** Fix the tenant-isolation tests | Critical | S | Multi-tenancy is the headline enterprise claim and its tests are red. |
| 9 | **0.5** Fix O(n) bcrypt API-key scan | High | M | Unauthenticated DoS primitive; also unblocks any service-user scale claim. |
| 10 | **1.4** Enable `env-filter` | High | S | Two lines. `RUST_LOG` currently does nothing while docs, Dockerfile, and chart all promise it works. |
| 11 | **1.2** Real health endpoints | High | S | Pods with dead databases currently stay in the Service. |
| 12 | **1.1** Graceful shutdown | High | S | Every rolling update severs in-flight commits and leaks metadata files. |
| 13 | **2.1 + 2.3** Format tree, purge artifacts and AI narration | Medium | S | Cheap, highly visible. Determines what a prospective contributor sees first. |
| 14 | **0.6** Revocation fails closed | High | S | Security control that currently degrades open under load. |
| 15 | **1.5** Prometheus metrics | High | M | Prerequisite for every monitoring, SLO, and alerting item in Phase 3. |
| 16 | **1.7** Transactions in Postgres and Mongo | Critical | L | Highest impact of the large items — but genuinely L, and safe to sequence after the quick wins. |

### A note on the README

Until Phase 1 completes, the README's claims — "100% compliant with Apache Iceberg REST spec" and eleven features marked "Production-Ready" — are not supported by the code. **Softening those claims should accompany the first fix commit**, not wait for the roadmap. Being early and honest is an asset for an alpha project; being caught overclaiming is expensive. The status line already says "Alpha," which is the right frame — the feature table should match it.

---

## 8. Appendix A — Codebase Metrics

| Metric | Value |
|---|---|
| Rust files / lines | 238 / ~43,100 |
| `pangolin_store` | 121 files, 19,016 lines |
| `pangolin_api` | 74 files, 18,157 lines |
| `pangolin_cli_admin` | 19 files, 3,039 lines |
| `pangolin_core` | 13 files, 1,868 lines |
| `pangolin_cli_user` | 3 files, 601 lines |
| `pangolin_cli_common` | 7 files, 334 lines |
| `CatalogStore` trait methods | 110 |
| Largest files | `postgres/main.rs` 1,598 · `iceberg/tables.rs` 930 · `pangolin_handlers.rs` 906 |
| `unsafe` blocks | **0** |
| `unwrap()` / `expect()` / `panic!` | 297 / 55 / 6 |
| TODO / FIXME / HACK | 9 |
| Declared test functions | ~304 (105 in `src`, 199 in `tests/`) |
| Tests actually executing | ~44 |
| Test targets failing to compile | 5 |
| `cargo fmt` diff hunks / files | 2,722 / 227 |
| Clippy warnings (unique) | 314 |
| Env vars read / `env::var` call sites | 39 / 88 |
| Tracked files / committed debug artifacts | 865 / 35+ |
| Documentation files | 134 |
| CI workflows running tests | **0** |
| Git commits | 186 |

## 9. Appendix B — Reproducing This Audit

```bash
cd /home/alexmerced/development/personal/Personal/library/2026/pangolin/pangolin

# Formatting (currently fails: 2,722 hunks / 227 files)
cargo fmt --all -- --check

# Lints + the 5 broken test targets (currently 314 warnings)
cargo clippy --workspace --all-targets --message-format short

# Currently executes zero tests — aborts during compilation
cargo test --workspace --no-fail-fast

# Targets that do compile and need no external services
cargo test -p pangolin_core --lib
cargo test -p pangolin_api --test rest_spec_tests --test iceberg_endpoints_tests \
                           --test isolation_test --test merge_tests --no-fail-fast

# Not run during this audit — install first, and only after Phase 0.11
# cargo install cargo-llvm-cov && cargo llvm-cov --workspace --html
# cargo install cargo-audit   && cargo audit
```
