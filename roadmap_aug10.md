# Pangolin — Roadmap & Audit Findings (August 10, 2026)

**Scope:** full-repo audit — Rust workspace (`pangolin/`, 6 crates, ~58.5k LOC), Python SDK (`pypangolin/`), SvelteKit UI (`pangolin_ui/`), deployment assets, docs, and repo hygiene.
**Baseline:** the day after the 0.6.0 security/hardening release, which executed most of `AUDIT_EXECUTION_PLAN.md`. Items already fixed or explicitly documented as known limitations in the README are **not** re-reported here.

---

## Overview

The 0.6.0 release materially improved the project's substrate, and this audit verified those claims locally:

- `cargo test --workspace --no-fail-fast` is **green** with no external services (verified today; the SQLite, memory, Mongo/Postgres-gated, and API integration targets all pass).
- `cargo fmt --all -- --check` is **clean**; `cargo clippy --workspace --all-targets` sits **exactly at its 36-warning budget** (`pangolin/clippy-warning-budget.txt`).
- CI (`.github/workflows/ci.yml`) now runs fmt, clippy + ratchet, tests (with and without live databases), `cargo audit`, helm lint/template, and a non-root Docker build.
- The Iceberg commit path (`pangolin_api/src/iceberg/commit.rs`) is now a well-tested pure module enforcing all documented requirements and updates.

The remaining problems cluster in five places:

1. **A new class of API-layer authorization bypasses (`pangolin_api`).** Several management and Iceberg handlers are mounted behind authentication but perform **no authorization check at all** — most severely, any authenticated principal can mint a `Root` JWT for any tenant, and any tenant member can vend read+write cloud credentials for a whole warehouse. These are as serious as the OAuth issues 0.6.0 fixed and were *not* in that release's scope. **This is the highest-priority cluster.**
2. **Backend parity and correctness in `pangolin_store`** — the four backends (memory, SQLite, Postgres, Mongo) disagree on tenant scoping, branch scoping, serde formats, CAS enforcement, and pagination determinism. Several are severe (a cross-tenant audit read, a revocation no-op on Mongo, a panic in Postgres search).
3. **Iceberg metadata JSON conformance and commit-path edge cases in `pangolin_core`/`pangolin_api`** — field naming and missing required fields mean metadata files Pangolin writes are not readable as spec-conformant v2 metadata by external engines, and several commit-path handlers mis-handle branches, nested namespaces, and client-supplied values.
4. **The UI's integration seams** — env-var mismatches, endpoints that don't exist on the server, missing 401 handling, and dead tenant switching.
5. **Client ↔ server contract drift** — the Python SDK and both Rust CLIs call dozens of wrong endpoints / wrong field names that fail silently or 404/422, because nothing tests them against the real router. Plus packaging/deployment drift: the quick-start `docker compose up` cannot start the API, and the release compose pins 0.2.0.

The Iceberg *commit-application* module (`commit.rs`) is genuinely solid and well-tested. The rest of the API crate, despite compiling cleanly and passing CI, carries real authorization gaps that CI cannot see because there are no authz tests. Most fixes below are contained and file-level; the meta-fix is a permission-matrix test plus a client↔server contract test.

---

## Bugs

Ordered by severity. Every location is verified against the working tree as of 2026-08-10. The `pangolin_api` authorization items below were spot-verified by reading the handler signatures directly (confirmed: the cited handlers take no `UserSession` and/or contain no `check_permission` call).

### Critical — API authorization bypasses (`pangolin_api`)

These handlers all sit behind `auth_middleware` (so they need a valid credential) but perform no authorization, so the bar is "any authenticated principal — including a lowest-privilege `TenantUser` or any service-user API key."

**B0a. `POST /api/v1/tokens` mints arbitrary-role, arbitrary-tenant JWTs for any authenticated caller — full privilege escalation.**
- `pangolin/pangolin_api/src/token_handlers.rs:45-113` (route `lib.rs:506`). **Verified:** `generate_token` takes only `State` + `Json<GenerateTokenRequest>` — no `Extension<UserSession>`, no permission check — and maps a body-supplied `roles: ["Root"]` straight into signed `Claims` (lines 70-104). Any `TenantUser` can POST `{"tenant_id":"<any-uuid>","roles":["Root"]}` and receive a valid `Root` token; `check_permission` short-circuits `Ok(true)` for `Root` (`authz.rs:16-18`).
- **Fix:** require `Extension<UserSession>`; reject unless caller is `Root`, or is `TenantAdmin` with `payload.tenant_id == session.tenant_id` and a role not exceeding the caller's. Never trust `roles` from the body for a non-Root caller.

**B0b. Credential-vending endpoint has no authz and never checks the table — any tenant member gets read+write warehouse credentials.**
- `pangolin/pangolin_api/src/signing_handlers.rs:200-250`. **Verified:** `get_table_credentials` takes no `UserSession`, calls no `check_permission`, hardcodes `permissions = ["read","write"]` (line 243), and never looks up the asset (`namespace`/`table` are only string-concatenated into the resource path). Any authenticated tenant member obtains read+write cloud storage credentials for the entire warehouse, for a table they have no rights to and that need not exist. Highest-value endpoint in the API to gate.
- **Fix:** load the asset; `check_permission(Read)` for read-only vending, `Write` before adding `"write"`; derive `permissions` from the caller's actual grants.

**B0c. `rename_table` has no permission check.**
- `pangolin/pangolin_api/src/iceberg/tables.rs:992-1036`. **Verified:** the handler binds `Extension(session)` but never calls `check_permission` (every sibling table handler does). Any tenant member can move any table into any namespace — an effective delete/DoS and a way to smuggle a table into a namespace where they have read rights.
- **Fix:** `check_permission(Write|Delete, source_asset)` and `check_permission(Create, dest_namespace)` before `rename_asset`.

**B0d. `update_namespace_properties` has no permission check.**
- `pangolin/pangolin_api/src/iceberg/namespaces.rs:307-363` — binds `Extension(_session)` (deliberately discarded) and never checks. Any tenant member can rewrite namespace properties including `location`, which later table creation derives paths from. This handler also never resolves the catalog (needs a `get_catalog` + 404).
- **Fix:** resolve the catalog, then `check_permission(Write, Namespace{...})`.

**B0e. View endpoints have no permission checks.**
- `pangolin/pangolin_api/src/asset_handlers.rs:66-109` (`create_view`), `:128-158` (`get_view`). Neither takes a `UserSession` or checks. Any tenant member can create views in any namespace and read any view's SQL text (`properties["sql"]`).
- **Fix:** mirror `create_table`/`load_table` scope checks.

**B0f. `perform_maintenance`: no authz AND hardcoded `"default"` catalog.**
- `pangolin/pangolin_api/src/iceberg/tables.rs:146-198`. Two bugs: (1) the catalog from the path (`_prefix`) is discarded and the literal `"default"` is passed to `expire_snapshots`/`remove_orphan_files` (lines 166, 184) — destructive maintenance runs against the wrong catalog; (2) no `UserSession`/`check_permission` — any tenant member can trigger snapshot expiry and orphan-file deletion on tables.
- **Fix:** use `_prefix` as the catalog; add `Write`/`Delete` checks.

**B0g. Iceberg OAuth token endpoint ignores service-user expiry.**
- `pangolin/pangolin_api/src/iceberg/oauth.rs:88-92`. **Verified:** checks only `service_user.active`, not `is_valid()` (`= active && !is_expired()`, `pangolin_core/src/user.rs:126-128`) — the API-key path uses `is_valid()` correctly (`auth_middleware.rs:233`). An **expired** service user can still exchange `client_credentials` for a fresh 1-hour JWT, fully bypassing key expiry. (Secondary: this public endpoint runs an unthrottled bcrypt per call, keyed by attacker-supplied UUID.)
- **Fix:** `if !service_user.is_valid() { return unauthorized }`.

**B0h. `PANGOLIN_DEV_MODE=true` waives the NO_AUTH public-bind guard.**
- `pangolin/pangolin_api/src/config.rs:216-218`. **Verified:** `if no_auth && !dev_mode && !is_loopback(...)`. The `!dev_mode` term means `PANGOLIN_NO_AUTH=true PANGOLIN_DEV_MODE=true` on the default `0.0.0.0` bind starts happily and treats every anonymous request as `TenantAdmin` (`auth_middleware.rs:209-221`) — and both flags are commonly set together in compose/dev setups. This breaks the invariant `auth_middleware.rs:203-204` documents.
- **Fix:** drop `!dev_mode`; the bind restriction should be unconditional whenever `no_auth` is on.

**B0i. `PermissionScope::Tenant` matches unconditionally — cross-tenant grant leak.**
- `pangolin/pangolin_api/src/authz_utils.rs:21,52,92`: `PermissionScope::Tenant => true` in all three access checks, never comparing `perm.tenant_id` against the resource's tenant. A `Tenant`-scoped grant issued in tenant A satisfies access for resources in tenant B. Masked today only because callers pre-scope at the store layer; any cross-tenant result path (root impersonation, search, dashboards) leaks.
- **Fix:** thread the resource tenant id in and require `perm.tenant_id == resource_tenant_id`.

**B0j. Logout / `revoke_current_token` revokes the wrong ID — tokens are never actually revoked.**
- `pangolin/pangolin_api/src/token_handlers.rs:196-201`. **Verified:** revokes `session.user_id`, but the middleware checks revocation by the token's `jti` (`auth_middleware.rs:372-378`), a fresh `Uuid::new_v4()` per token. Logout returns 200 and the token keeps working until natural expiry (24h). Wrong-variable bug; also blocks `rotate_token`.
- **Fix:** carry `jti` into `UserSession` and revoke that.

**B0k. `POST /api/v1/oauth/exchange` is not in the public-path allowlist — the OAuth login flow cannot complete.**
- Route `lib.rs:516-519`, handler `oauth_handlers.rs:339`, allowlist `public_paths.rs:27-56`. **Verified:** `is_public_path` does not match `["api","v1","oauth","exchange"]`, so the middleware demands a bearer token on the very endpoint whose job is to obtain the first token. The 0.6.0 A-8 remediation (callback → one-time code → POST exchange) is unreachable in production — the browser lands with `?code=...` it can never redeem.
- **Fix:** add `["api","v1","oauth","exchange"] => true` and a regression test.

**B0l. OAuth account linking by unverified email.**
- `pangolin/pangolin_api/src/oauth_handlers.rs:193-198`: existing-user match includes `|| u.email == user_info.email` with no `email_verified` check and no provider/domain binding. An attacker who sets a matching email on any configured provider (GitHub allows unverified addresses) logs in as that Pangolin user, including the seeded `TenantAdmin`. (Also O(all users) per login; truncated pagination silently forces a `create_user` instead.)
- **Fix:** match on `(provider, subject)` only; gate email linking behind a verified `id_token` + operator domain allowlist.

**B0m. Request-controlled panics in token issuance.**
- `pangolin/pangolin_api/src/token_handlers.rs:56-61` (also `:143-145`): `chrono::Duration::hours(expires_in as i64)` panics for a huge `expires_in_hours` before the `.unwrap()` on `checked_add_signed` even runs. `expires_in_hours` is attacker-controlled `Option<u64>`; there is no `CatchPanicLayer` in the stack (`lib.rs:551-575`), so the panic aborts the connection task.
- **Fix:** clamp `expires_in` to a configured max, use `checked_*` and return 400; add `tower_http::catch_panic::CatchPanicLayer`.

### Critical — data integrity & tenant isolation (storage layer)

**B1. Mongo `get_audit_event` ignores `tenant_id` — cross-tenant audit-log read.**
- `pangolin/pangolin_store/src/mongo/mod.rs:461-467` discards the `tenant_id` parameter (`_tenant_id`), and `pangolin/pangolin_store/src/mongo/audit.rs:34-42` filters only on `{ "id": ... }`.
- Postgres (`postgres/audit.rs:164`) and SQLite (`sqlite/audit_logs.rs:176`) both scope by tenant. On Mongo, any tenant holding an audit-event UUID can read another tenant's audit record (username, IP, resource names, metadata).
- **Fix:** add `"tenant_id": to_bson_uuid(tenant_id)` to the filter and thread the parameter through.

**B2. Mongo token revocation is a silent no-op — revoked JWTs stay valid.**
- `pangolin/pangolin_store/src/mongo/tokens.rs:79-101`: revocations are inserted via serde (UUID → string), but `is_token_revoked` queries with a BSON Binary UUID (`to_bson_uuid`). The filter can never match, so revocation checks always return `false` on the Mongo backend.
- `cleanup_expired_tokens` (`tokens.rs:103-112`) has the same type mismatch (`$lt` BSON DateTime vs stored RFC3339 string) — cleanup never deletes anything and the collection grows unbounded.
- **Fix:** write via an explicit `doc!` using `to_bson_uuid`/`Bson::DateTime` (mirroring `store_token` at lines 66-74). Add a store-level revoke→check roundtrip test that runs against all backends.

**B3. SQLite `delete_branch` references a non-existent column and is non-transactional — orphans branch assets.**
- `pangolin/pangolin_store/src/sqlite/branches.rs:109-130`: the branch row is deleted and committed, then `DELETE FROM assets ... AND branch = ?` fails ("no such column" — the schema column is `branch_name`, `sql/sqlite_schema.sql:71`). The branch is gone, its assets are permanently orphaned, and the caller receives an error.
- Postgres was fixed for exactly this (`postgres/branches.rs:127-129` comment) and wraps both statements in a transaction; SQLite was never patched.
- **Fix:** `branch = ?` → `branch_name = ?`, and wrap both statements in `self.pool.begin()`/`tx.commit()`.

**B4. Postgres `search_assets` panics on any match — `TEXT[]` decoded as `String`.**
- `pangolin/pangolin_store/src/postgres/main.rs:1391-1392`: `row.get::<String>("namespace_path")` against a `TEXT[]` column (`migrations/20251212000000_initial_schema.sql:47`). `sqlx::Row::get` panics on decode failure, so any search with ≥1 hit panics the request. The correct decode (`Vec<String>`) is used at `main.rs:1449` and `postgres/assets.rs:87`.
- SQLite has the sibling bug at `sqlite/business_metadata.rs:156-157` — no panic, but every result's namespace is one element of raw JSON (`["[\"a\",\"b\"]"]`) instead of the parsed path.
- **Fix (pg):** `let namespace: Vec<String> = row.get("namespace_path");` **Fix (sqlite):** `serde_json::from_str(&namespace_path).unwrap_or_default()`.

**B5. Mongo `update_metadata_location` drops the compare-and-swap — lost Iceberg commits.**
- `pangolin/pangolin_store/src/mongo/main.rs:213`: `_expected_location` is ignored; the update is an unconditional `$set`. Memory (`memory/io.rs:62`), Postgres (`postgres/main.rs:1552,1564`), and SQLite (`sqlite/assets.rs:383`) all enforce the CAS. On Mongo, two concurrent commits both "succeed" and one snapshot is silently lost — the exact failure class 0.6.0 fixed at the API layer.
- **Fix:** put the expected location in the update filter (`"properties.metadata_location": expected`, or `$exists: false` when `None`) and error when `modified_count == 0`. This needs no multi-document transaction, so it works on standalone `mongod`.

**B6. Memory `delete_catalog` corrupts other tenants' asset-by-id index.**
- `pangolin/pangolin_store/src/memory/catalogs.rs:124`: `self.assets_by_id.retain(|_, v| v.0 != name)` filters on catalog **name only** — tenant A deleting catalog `sales` breaks `get_asset_by_id` for tenant B's `sales`.
- **Fix:** include `tenant_id` in the `assets_by_id` value and match on both.

**B7. All three persistent backends silently rewrite asset types to `IcebergTable`.**
- Write path stores `format!("{:?}", asset.kind)`, read path parses only `IcebergTable`/`View` and defaults everything else: `postgres/assets.rs:25` + `:53-57,:89-93,:140-144`; `sqlite/assets.rs:27` + `:142-146,:181-185,:234-238`; `mongo/assets.rs:36` + `:75-79,:131-135,:183-187`.
- `AssetType` has 17 variants (`pangolin_core/src/model.rs:93-111`) — a `DeltaTable`, `MlModel`, `Lance`, etc. round-trips as `IcebergTable`. This defeats a headline feature ("tracks any lakehouse asset type").
- **Fix:** serialize via serde (the enum already has a rename policy) and hard-error on unknown values; add a parity test that round-trips a third variant.

### Critical — deployment & quick start

**B8. `docker compose up` cannot start the API.**
- `docker-compose.yml:34-49` (service `pangolin-api`) sets no `PANGOLIN_JWT_SECRET`; since 0.6.0 the server refuses to start without one (`pangolin/pangolin_api/src/config.rs:49`), and `PANGOLIN_NO_AUTH` is refused on the default `0.0.0.0` bind (`config.rs:215-217`). The documented quick start yields a crash-looping container.
- **Fix:** add `PANGOLIN_JWT_SECRET=${PANGOLIN_JWT_SECRET:?generate with openssl rand -base64 48}` to the compose file (fail fast with a clear message), and document it in the quick start.

**B9. Compose files set the wrong storage env var.**
- `docker-compose.yml:43` and `docker-compose.release.yml:42` set `PANGOLIN_STORE_TYPE`; the server reads `PANGOLIN_STORAGE_TYPE` (`pangolin/pangolin_api/src/main.rs:187`). The variable is silently ignored — anyone editing it to `postgres` would still get the memory backend.
- **Fix:** rename to `PANGOLIN_STORAGE_TYPE` in both files (and see B18 for the docs side).

**B10. `docker-compose.release.yml` is stale and self-broken.**
- Line 37 pins `image: alexmerced/pangolin-api:0.2.0` (four releases behind the 0.6.0 workspace), and line 68 runs `scripts/test_release_v0.2.0.py`, which does not exist in `scripts/`.
- **Fix:** pin `0.6.0` (or parameterize `${PANGOLIN_VERSION}`), point at a real verification script (e.g. `scripts/integration_test.py`), and add a CI job that `docker compose -f docker-compose.release.yml config`-validates the file.

### High — Iceberg spec conformance (core metadata model)

**B11. Table metadata JSON is not spec-conformant: wrong name for `default-spec-id`.**
- `pangolin/pangolin_core/src/iceberg_metadata.rs:17`: `current_partition_spec_id` under `rename_all = "kebab-case"` serializes as `current-partition-spec-id`. The Iceberg v2 spec field is `default-spec-id`. Metadata files Pangolin writes are unreadable as conformant v2 metadata by external engines reading the file directly, and a conformant engine's metadata can't round-trip in.
- **Fix:** `#[serde(rename = "default-spec-id", alias = "current-partition-spec-id")]` so old Pangolin-written files still parse.

**B12. Required v2 field `last-partition-id` is missing entirely.**
- `pangolin/pangolin_core/src/iceberg_metadata.rs:8-33` (struct `TableMetadata`); `grep -r last_partition_id` over the workspace returns nothing. The v2 spec requires `last-partition-id` (highest assigned partition field id); Java-based readers reject metadata without it.
- **Fix:** add the field with a serde default computed from `partition_specs` on read; maintain it in `apply_updates`' `AddSpec` arm (`pangolin_api/src/iceberg/commit.rs:369-375`).

**B13. `metadata-log` is initialized empty and never appended.**
- `pangolin/pangolin_api/src/iceberg/tables.rs:400` creates `metadata_log: Some(vec![])`; no code path ever pushes a `MetadataLogEntry`. The spec expects each commit to record the previous metadata file — engines use it for metadata time-travel and previous-metadata cleanup (`write.metadata.previous-versions-max`).
- **Fix:** in the commit handler, before writing the new metadata file, append `{timestamp_ms, metadata_file: <previous location>}` and truncate to the configured max.

**B14. `Schema` JSON omits `"type": "struct"`; optional fields serialize as explicit `null`.**
- `pangolin/pangolin_core/src/iceberg_metadata.rs:53` (struct `Schema`) has no `type` field — spec schemas are struct types and conformant writers emit `"type": "struct"`. Also `NestedField.doc` and `Schema.identifier_field_ids` lack `skip_serializing_if`, producing `"doc": null` noise that some strict parsers reject.
- **Fix:** add a `#[serde(rename = "type")] type_: String` defaulted to `"struct"`, and `skip_serializing_if = "Option::is_none"` on the optional fields.

**B15. Client-supplied sequence numbers can jump the counter arbitrarily.**
- `pangolin/pangolin_api/src/iceberg/commit.rs:481-485`: a snapshot whose `sequence-number` exceeds `last_sequence_number` is honored verbatim. A client can submit `i64::MAX`, after which the next commit computes `last_sequence_number + 1` (line 480) — overflow (panic in debug builds, wrap in release, corrupt ordering either way).
- **Fix:** accept the client value only if it equals `last_sequence_number + 1`; otherwise assign the next counter value (or reject with `CommitError::Invalid`).

**B16. Committing to a non-main branch mutates main-visible state.**
- `pangolin/pangolin_api/src/iceberg/commit.rs:497` sets `current_snapshot_id` to the new snapshot for **any** branch, and lines 518-529 fabricate a `main` ref pointing at the branch's snapshot when no `main` ref exists. If a metadata document is ever shared across Pangolin branches (or exported), a `dev`-branch commit changes what `main` readers resolve.
- **Fix:** only update `current_snapshot_id`/insert the `main` ref when `branch == MAIN_REF`; add a test asserting a feature-branch commit leaves `ref_snapshot_id(metadata, "main")` unchanged.

### High — additional Iceberg commit/handler bugs (`pangolin_api`)

**B16a. Nested namespaces are parsed two different ways — commit/delete/HEAD on a nested namespace 404.**
- In `pangolin/pangolin_api/src/iceberg/tables.rs`, `list_tables`/`create_table`/`load_table` use `parse_table_identifier` (lines 75, 266, 570) yielding a single-element namespace, while `update_table`/`delete_table`/`table_exists` use `namespace.split('\x1F')` (lines 747, 1118, 1226) yielding a multi-element path. Creating a table in namespace `a\x1Fb` registers it under `["a\x1Fb"]`, but the commit path looks it up under `["a","b"]` → `404 Table not found`, and the CAS loop never runs. `parse_table_identifier` also splits on `@`, so `update_table` mishandles a `ns@branch` suffix that `load_table` accepts.
- **Fix:** one shared `parse_namespace(&str) -> (Vec<String>, Option<String>)` (split on `0x1F`, strip trailing `@branch`) used in all six handlers plus `namespaces.rs:242,334` and `asset_handlers.rs:79,139`.

**B16b. `last-updated-ms` only changes on `add-snapshot` commits.**
- `pangolin/pangolin_api/src/iceberg/commit.rs:496` is the only assignment (inside `add_snapshot`). A commit of only `set-properties`/`add-schema`/`set-location`/`add-spec`/`set-snapshot-ref`/`remove-snapshots` publishes a new metadata file with an unchanged `last-updated-ms`, so consumers that order or dedupe metadata by that field treat the two versions as identical.
- **Fix:** set `metadata.last_updated_ms = Utc::now().timestamp_millis()` once in `update_table` after `apply_updates` returns `Ok`.

**B16c. `set-current-schema: -1` (and `set-default-spec`/`set-default-sort-order: -1`) resolve against the full list, not "added in this commit."**
- `pangolin/pangolin_api/src/iceberg/commit.rs:275-295, 377-395, 407-425`. Because `metadata.schemas` is never empty for an existing table, a `-1` sent *without* a preceding `add-schema` doesn't hit the intended `None`/error arm — it silently points the table at whatever is `.last()` in the persisted vector (arbitrary if not in creation order). The error message asserts a check the code doesn't perform. `Add*` arms also don't reject duplicate IDs.
- **Fix:** track `last_added_{schema,spec,sort_order}_id: Option<i32>` locals in `apply_updates`, resolve `-1` to those, error when `None`; reject or remap duplicate `Add*` IDs.

**B16d. Metadata files are written before the CAS and never reclaimed on loss — object-storage leak under contention.**
- `pangolin/pangolin_api/src/iceberg/tables.rs:857-944`: each of up-to-5 retry iterations writes a full metadata file then attempts `update_metadata_location`; on CAS loss it `continue`s, orphaning the just-written file, and up to 5 orphans remain on final give-up. No reaper; orphans are indistinguishable from live metadata (compounds B13's empty `metadata-log`).
- **Fix:** best-effort delete `new_metadata_location` on the CAS-failure branch, or derive the location deterministically from `(table_uuid, version)` so a retry overwrites.

**B16e. `create_table` returns the table *directory* as `metadata-location`.**
- `pangolin/pangolin_api/src/iceberg/tables.rs:496` returns `Some(location.clone())` (the table root) where it should return `metadata_location` (the file, computed at line 405). `load_table` returns the file correctly (line 678). A client keeping the returned `Table` (PyIceberg does) has a `metadata_location` it cannot read or refresh from. Straight wrong-variable bug.
- **Fix:** `Some(metadata_location.clone())`.

**B16f. `create_table` hand-parses the schema: drops nullability, widens `int`→`long`, and silently deletes complex-typed columns.**
- `pangolin/pangolin_api/src/iceberg/tables.rs:331-371`: `required = false` hardcoded (line 338); `"int"|"integer" => long` (line 342); `field.get("type")?.as_str()?` inside a `filter_map` returns `None` for any `struct`/`list`/`map`/`decimal`/`fixed` field, so those columns are dropped and `last_column_id` is miscomputed — table creation succeeds `200 OK` with a schema missing columns.
- **Fix:** deserialize the incoming schema straight into `pangolin_core::iceberg_metadata::Schema` (as `commit.rs:265` does) and 400 on failure.

**B16g. `create_table` registers the asset before the metadata file is durable.**
- `pangolin/pangolin_api/src/iceberg/tables.rs:426-441`: `create_asset` runs first, `write_file` second; if the write fails the asset remains registered pointing at a nonexistent file, permanently breaking the table (`load_table` 500s, `update_table` 404s) with no repair path. Inverted vs. the commit path.
- **Fix:** write the metadata file first, then `create_asset`; best-effort delete the file on `create_asset` failure.

**B16h. `update_namespace_properties` silently ignores `removals`.**
- `pangolin/pangolin_api/src/iceberg/namespaces.rs:336-347`: only `updates` are applied; a request with `removals` gets `200 OK` with `removed: []`/`missing: []` while nothing was removed — the exact "silent success" failure class 0.6.0 fixed elsewhere.
- **Fix:** implement removals and populate `updated`/`removed`/`missing` honestly.

**B16i. Iceberg list pagination is non-conforming; `next-page-token` is never returned.**
- `pangolin/pangolin_api/src/iceberg/types.rs:12-15,107-110`, `namespaces.rs:40,93`, `tables.rs:46,102`: the API uses `{limit,offset}` and the list responses have no `next-page-token` field. A spec client sending `pageToken`/`pageSize` has both ignored and cannot detect truncation → silent partial results on large namespaces.
- **Fix:** accept `pageToken`/`pageSize`, encode offset into an opaque token, add `next-page-token` to both response types.

**B16j. Most Iceberg handlers still return plain-text bodies, not the spec error envelope.**
- `iceberg/tables.rs:69,84,259,276,586,600,739,760,774,801,813,1110,1131,1145` and `iceberg/namespaces.rs:70,82,146,158,236,252,285,350` return `(StatusCode, "…")` text bodies; only `commit_error_response` uses the `iceberg/error.rs` helpers. Engines that switch on `error.type` to decide commit-retry cannot parse these. (This is the old A-6 — it was **not** in the 0.6.0 fix set despite the module doc reading as though resolved.)
- **Fix:** route every bare-tuple return in `iceberg/` through the `iceberg_error(...)` helpers.

**B16k. Federated forwarding is missing on create/delete namespace and the namespace tree.**
- `pangolin/pangolin_api/src/iceberg/namespaces.rs:127-205, 224-287, 381-464` don't call `check_and_forward_if_federated` (which `list_namespaces`/`update_namespace_properties` do). On a `Federated` catalog, creating a namespace makes a local shadow returning `200` while `GET` lists the remote — the two views permanently diverge; delete reports success for a namespace still present upstream.
- **Fix:** add the forwarding guard to all three; audit `asset_handlers`/`signing_handlers` for the same omission.

### High — API middleware / reliability (`pangolin_api`)

**B16l. Timeout layer is nested inside the concurrency limiter — queued requests have no deadline.**
- `pangolin/pangolin_api/src/lib.rs:561-569`: effective order is `concurrency-limit → timeout → body-limit`, so a request waiting for one of the 512 permits has no deadline; the 30s timer starts only after admission. Under sustained overload the queue grows unbounded and clients see latency far past `PANGOLIN_REQUEST_TIMEOUT_SECS`.
- **Fix:** register `TimeoutLayer` after (outside) `GlobalConcurrencyLimitLayer`; consider `LoadShedLayer` outside both to return 503.

**B16m. `delete_warehouse` invalidates the cache *before* deleting — a concurrent read re-poisons it.**
- `pangolin/pangolin_api/src/cached_store.rs:110-113`: invalidate then delete. A `get_warehouse` racing in between misses, reads the still-present row, and re-inserts it with a full TTL — the deleted warehouse's cloud credentials keep being vended for up to the cache TTL after delete returns success.
- **Fix:** delete first, then invalidate (and invalidate again on the error path).

**B16n. `shutdown_grace` is logged but never applied — SIGTERM can hang forever.**
- `pangolin/pangolin_api/src/main.rs:340-370`: the grace value is only logged; there is no `sleep` and no `timeout` around the drain, so `with_graceful_shutdown` waits for in-flight connections indefinitely. A single hung upstream blocks SIGTERM past the k8s grace period into SIGKILL mid-commit; `PANGOLIN_SHUTDOWN_GRACE_SECS` has no effect.
- **Fix:** sleep briefly for LB deregistration, then wrap the serve future in `tokio::time::timeout(grace, …)`.

**B16o. Token revocation is skipped for tokens with a missing/malformed `jti`; API-key auth runs before the public-path check.**
- `auth_middleware.rs:372-393`: both `if let`s fail open, so a token minted without a UUID `jti` is unrevocable for its lifetime (`Claims.jti` is `Option<String>` "for compatibility"). Separately, the `X-API-Key` branch (`auth_middleware.rs:227-277`) returns before `is_public_path` (line 275), so a client sending `X-API-Key` globally can't reach `/v1/config`, `/health`, or the OAuth token endpoint — the opposite of the documented ordering.
- **Fix:** reject a present-but-unparseable `jti`; gate legacy no-`jti` acceptance behind a default-off flag; move the public-path check above the API-key branch.

### High — storage-layer parity & correctness (continued)

**B17. Memory backend namespace delete/update use a different key encoding than create — always "not found" for nested namespaces.**
- `pangolin/pangolin_store/src/memory/namespaces.rs`: create/get key with `join(".")` (lines 13, 60) but delete/update key with `join("\x1F")` (lines 73, 88). Multi-level namespaces can never be deleted or updated on the memory backend, while the SQL backends succeed.
- **Fix:** a single shared `ns_key()` helper using `join(".")`.

**B18. SQLite `PRAGMA foreign_keys = ON` applies to one pooled connection.**
- `pangolin/pangolin_store/src/sqlite/main.rs:58,118,158`: the pragma is per-connection; `execute(&pool)` configures one arbitrary connection out of the pool, so `ON DELETE CASCADE` fires nondeterministically depending on which connection serves a request. Line 118's `OFF` can additionally stick on a connection the later `ON` never touches.
- **Fix:** `SqliteConnectOptions::new().foreign_keys(true)` (or an `after_connect` hook) so every connection is configured.

**B19. SQLite `get_metadata_location` ignores the branch — cross-branch metadata reads.**
- `pangolin/pangolin_store/src/sqlite/assets.rs:322,327`: `_branch` is discarded and the query matches rows from every branch; `fetch_optional` returns an arbitrary one. Reading a table on `dev` can return `main`'s pointer. Postgres (`postgres/main.rs:1521`) and Mongo (`mongo/main.rs:196`) scope by branch.
- **Fix:** `AND branch_name = ?` binding `branch.unwrap_or("main")`.

**B20. SQLite `update_metadata_location` leaves the `metadata_location` column stale.**
- `pangolin/pangolin_store/src/sqlite/assets.rs:393` updates only `properties`, but reads populate `Asset.location` from the column (`assets.rs:152-154,244-246`) — so on SQLite, `Asset.location` is frozen at creation time after every Iceberg commit. Postgres updates both (`postgres/main.rs:1552`).
- **Fix:** `UPDATE assets SET metadata_location = ?, properties = ? ...`.

**B21. SQLite `delete_catalog`: non-transactional cascade that purges children even when the catalog doesn't exist.**
- `pangolin/pangolin_store/src/sqlite/catalogs.rs:146-189`: five sequential deletes with no transaction, and the "not found" check is the **last** statement — `delete_catalog(tenant, "nonexistent")` deletes matching tags/branches/assets/namespaces, then errors. Postgres wraps the identical cascade in a transaction (`postgres/catalogs.rs:158-204`).
- **Fix:** check existence first, then run the cascade inside `pool.begin()`/`tx.commit()`, matching Postgres.

**B22. SQLite audit log: multi-word actions all read back as `CreateCatalog`.**
- `pangolin/pangolin_store/src/sqlite/audit_logs.rs:22` stores `format!("{:?}", action)` (`"CreateBranch"`), and lines 107-108/187-188 deserialize `"createbranch"` against serde's `snake_case` (`create_branch`) — no match — then `unwrap_or(AuditAction::CreateCatalog)` swallows it. The audit trail on SQLite misattributes nearly every action.
- **Fix:** persist the serde name symmetrically and propagate parse errors instead of defaulting.

**B23. Mongo audit filters never match; listing is unsorted and unbounded.**
- `pangolin/pangolin_store/src/mongo/audit.rs:80-85` filters with Debug names (`"CreateBranch"`) against serde-written snake_case documents — action/resource-type filters always return zero rows and `count_audit_events` returns 0. `resource_id`/`result` filters are ignored, and `list_audit_events` (lines 58-71) applies no sort/limit/offset while the SQL backends use `ORDER BY timestamp DESC LIMIT 100`.
- **Fix:** build filters with `bson::to_bson`, add the missing fields, and mirror the SQL sort/limit/skip.

**B24. Postgres `list_catalogs` fabricates `catalog_type: Local` and `federated_config: None`.**
- `pangolin/pangolin_store/src/postgres/catalogs.rs:65,77,80`: the SELECT omits both columns and hardcodes the values; `get_catalog` decodes them properly. Every federated catalog appears Local in a Postgres listing — anything branching on `catalog_type` over a listing takes the wrong path. SQLite and Mongo return real values.
- **Fix:** select and decode `catalog_type, federated_config` as in `get_catalog` (line 25).

**B25. Memory `merge_branch` reuses asset IDs and never advances the target head.**
- `pangolin/pangolin_store/src/memory/branches.rs:161-212`: copied assets keep the same `asset.id`, so `assets_by_id` is repointed at the target-branch copy (source lookups now resolve wrong); and `target_branch.head_commit_id` is never set, unlike all three other backends (`postgres/branches.rs:180`, `sqlite/branches.rs:161`, `mongo/branches.rs:172-176`).
- **Fix:** mint `Uuid::new_v4()` per copied asset and set the target head from the source branch.

**B26. Memory CAS skips the check entirely when `expected_location` is `None`.**
- `pangolin/pangolin_store/src/memory/io.rs:62`: `if let Some(expected) = ...` means the create-path CAS (expected = None must require "no existing location") is not enforced — a create-table race that Postgres/SQLite correctly reject silently succeeds in dev/tests.
- **Fix:** compare `current_loc != expected_location` unconditionally (the SQLite form).

**B27. Pagination is nondeterministic almost everywhere: `LIMIT/OFFSET` with no `ORDER BY`.**
- Only three list queries in the crate are ordered (`postgres/assets.rs:127`, `postgres/service_users.rs:77`, `postgres/access_requests.rs:62`). Every other paginated query in postgres/sqlite/mongo, and every memory-backend `skip()/take()` over DashMap iteration order, can repeat or skip rows between pages. (Full site list: see the storage audit notes — ~35 call sites.) Memory additionally sorts `list_catalogs`/`list_tenants` while the SQL backends don't — a direct parity divergence.
- **Fix:** add `ORDER BY name` (or `id`) to every paginated query; sort memory-backend snapshots before `skip/take`. Cheap, mechanical, and testable with a "two pages cover the set exactly once" parity test.

**B28. Search behavior diverges across all four backends.**
- LIKE/ILIKE wildcards unescaped: `postgres/main.rs:1331,1401,1438,1459`, `sqlite/business_metadata.rs:87,166,200,231` (`format!("%{}%", query)`) — a query containing `%`/`_` is a wildcard; Mongo escapes correctly (`regex::escape`), memory uses literal `contains`. Four backends, four answers.
- Tag-filter semantics: memory & SQLite = ANY-match; Postgres (`@>`) & Mongo (`$all`) = ALL-match. Empty tag list returns zero results on memory, everything on the others.
- **Fix:** escape `%`, `_`, `\` + `ESCAPE '\'` in the SQL backends; pick one tag semantic, document it on the trait (`lib.rs:490-512`), and align.

**B29. Memory audit log: ascending order and unbounded when no filter is given.**
- `pangolin/pangolin_store/src/memory/audit.rs:14,22-23,80-85`: insertion order (oldest first) vs `ORDER BY timestamp DESC` elsewhere, and the limit/offset logic is nested inside `if let Some(filter)`, so a filterless list returns every event while SQL caps at 100.
- **Fix:** sort descending by timestamp; hoist pagination defaults out of the filter branch.

**B30. Memory `delete_tenant` has no cascade.**
- `pangolin/pangolin_store/src/memory/tenants.rs:56-63` (`// TODO`): warehouses, catalogs, assets, and cached credentials survive tenant deletion on the memory backend. The retain-based cascade pattern already exists at `memory/catalogs.rs:107-124`.

### High — Management UI (`pangolin_ui`)

**B31. The API base URL env var is never defined anywhere — deployed UIs call `http://localhost:8080`.**
- `pangolin_ui/src/lib/api/client.ts:5` reads `env.PUBLIC_API_URL`; nothing sets it — `.env.example:1` declares `VITE_API_URL`, `docker-compose.yml:60` passes `VITE_API_URL`, `vitest.config.ts` sets `VITE_API_URL`. The fallback always wins, so every deployed build calls the **end user's** localhost. The login page compounds it by reading a *third* variant, `import.meta.env.VITE_API_URL || 'http://127.0.0.1:8080'` (`src/routes/login/+page.svelte:216-217`).
- **Fix:** standardize on `PUBLIC_API_URL` everywhere (SvelteKit dynamic public env requires the `PUBLIC_` prefix), default to `''` (same-origin, reverse-proxy friendly), and fail loudly in prod if unset.

**B32. ~12 raw `fetch('/api/v1/...')` calls only work under the dev proxy — 404 in production, and omit the tenant header.**
- Sites: `src/routes/permissions/+page.svelte:34,41,68,91`; `src/routes/commits/+page.svelte:14`; `src/routes/access-requests/+page.svelte:21,37`; `src/routes/search/+page.svelte:29`; `src/routes/assets/[id]/+page.svelte:31,65`; `src/lib/components/explorer/TableDetail.svelte:57,81`; `src/lib/stores/auth.ts:43`. With `adapter-node`, relative paths hit the SvelteKit server (no `/api/v1` routes → 404); they also skip `X-Pangolin-Tenant`, so root users resolve against the wrong tenant.
- **Fix:** route all of them through `apiClient`.

**B33. UI calls endpoints that don't exist on the server.**
- `DELETE /api/v1/branches/{catalog}/{name}` (`src/lib/api/branches.ts:64-67`) — the router (`pangolin/pangolin_api/src/lib.rs:255`) registers only GET on `/api/v1/branches/:name`; **branch deletion from the UI always 404s.**
- `GET /api/v1/users/{id}/permissions` (`src/lib/api/permissions.ts:107-111`, duplicated raw at `src/routes/permissions/+page.svelte:41`) — no such route; the permissions page and `EditPermissionsDialog.svelte:53` show an empty list forever.
- `GET /api/v1/oauth/providers` and `/api/v1/oauth/{provider}` (`src/lib/api/auth.ts:105-115`) — router has `/oauth/authorize/:provider` etc.; both 404 (currently dead code, which is why the login page hardcodes 4 provider buttons unconditionally).
- **Fix:** add the missing server routes (branch DELETE; permissions-by-user filter or `GET /api/v1/permissions?user_id=`) and wire the UI to real paths; render OAuth buttons from a real capability endpoint.

**B34. No 401 handling anywhere; logout is client-side only.**
- `client.ts:51-60` returns errors generically; zero `401` handling in `src/` — an expired JWT leaves the user in a permanently broken "authenticated" session (`auth_token` never cleared, no redirect). Logout (`src/lib/stores/auth.ts:236-248`) only clears localStorage; `authApi.logout()`/token revocation endpoints are never called, so the JWT stays valid server-side.
- **Fix:** on 401 in `ApiClient.request`, `authStore.logout()` + `goto('/login')`; call the server revocation endpoint on logout.

**B35. `/search` sends repeated `tags` params that the axum handler cannot deserialize.**
- `src/routes/search/+page.svelte:22-31` appends `tags` repeatedly; the server uses `Query<SearchRequest>` with `tags: Option<Vec<String>>` (`pangolin_api/src/business_metadata_handlers.rs:189-192,205-208`) — `serde_urlencoded` can't parse repeated keys into `Vec` → HTTP 400. **Tag-filtered search is broken end-to-end** (also see B28 for the store-side divergence).
- **Fix:** comma-join tags client-side and split server-side, or switch the handler to `axum_extra::extract::Query`.

**B36. Catalogs page: create button never renders; pagination refetches the same unpaginated list.**
- `src/lib/components/ui/DataTable.svelte:72-84`: the only `<slot name="actions"/>` outlet is inside `{#if searchable && !serverSide}` — the catalogs page passes `searchable={false} serverSide={true}` (`src/routes/catalogs/+page.svelte:130-131`), so the "New Catalog" control (lines 137-145) never renders and there is **no way to create a catalog from the catalogs page**.
- Same page never sends `limit`/`offset` (`+page.svelte:44-53` calls `catalogsApi.list()` bare), never sets `hasNextPage` — Next is permanently disabled. (Playwright's `test-results/.last-run.json` records 4 failed `pagination_verify` tests consistent with this.)
- Bonus: HTML comments in attribute position (`+page.svelte:127,130-135`) are parsed by Svelte as boolean props (`<!--`, `Updated`, `-->` etc. spread onto the component).
- **Fix:** move the actions slot outside the guard (render when `searchable || $$slots.actions`); pass `pageSize`/`offset` and set `hasNextPage = items.length === pageSize`; move comments out of attribute lists.

**B37. Root layout references `tenantStore` without importing it; tenant switcher is dead.**
- `src/routes/+layout.svelte:88-103` uses `tenantStore.selectTenant/clearTenant` with no import (latent `ReferenceError`), the handler isn't wired to any element, and the tenant loader is commented out (lines 54-65) — root users cannot switch tenants, so `X-Pangolin-Tenant` is never set. `TenantSelector.svelte` exists and is unused.
- **Fix:** import the store, re-enable the selector.

### Medium — Python SDK (`pypangolin`)

**B38. `__version__ = "0.1.0"` while the package publishes 0.6.0.**
- `pypangolin/src/pypangolin/__init__.py:23` vs `pypangolin/pyproject.toml:7`. The one-version-everywhere property 0.6.0 introduced (workspace `Cargo.toml` comment) is already broken in the SDK.
- **Fix:** single-source it — `importlib.metadata.version("pypangolin")` or have CI assert equality.

**B39. No request timeouts anywhere in the SDK — calls can hang forever.**
- `pypangolin/src/pypangolin/client.py:115`: `requests.request(method, url, headers=headers, **kwargs)` with no `timeout=`; `grep -rn timeout` over `src/` finds none. A hung server blocks the caller indefinitely (and the CLI built on it).
- **Fix:** add a configurable `timeout=(connect, read)` default (e.g. `(5, 30)`) on `PangolinClient` and pass it through; also use a `requests.Session` for connection reuse and `raise ... from e` to preserve tracebacks (`client.py:116-117`).

**B40. `requires-python = ">=3.8"` is unsatisfiable with the declared dependencies.**
- `pypangolin/pyproject.toml:10` vs `pydantic>=2.0.0` and `pyiceberg>=0.5.0` (lines 19-27), both of which require ≥3.9 (current pyiceberg ≥3.9/3.10). A 3.8 install resolves to broken or ancient dep versions.
- **Fix:** bump to `>=3.9` (or the floor pyiceberg actually needs) and add classifiers to match.

**B41. Importing `pypangolin` at all requires the full heavy dependency set; tests cannot even collect without it.**
- `pypangolin/src/pypangolin/__init__.py:2` → `catalog.py:1` eagerly imports `pyiceberg`. Verified: `pytest tests/` fails collection with `ModuleNotFoundError: No module named 'pyiceberg'` even though the tests under test (`tests/test_cli_config.py`, `tests/test_cli_commands.py`) only touch the CLI/config. There is also no pytest config making the `src/` layout importable without an editable install.
- **Fix:** make `get_iceberg_catalog` import pyiceberg lazily inside the function and move `pyiceberg` to an optional extra (`pypangolin[iceberg]`); add `[tool.pytest.ini_options] pythonpath = ["src"]` or document `pip install -e .` in a test README; add SDK tests to CI (see improvements).

### High — client ↔ server contract drift (both Rust CLIs and the Python SDK)

Neither CLI crate nor the SDK is tested against the real router, so dozens of commands call wrong endpoints or send wrong field names. They fail as 404/405/422 or — worse — as silent no-ops that print success, because every CLI command swallows exceptions with no non-zero exit (`pypangolin/src/pypangolin/cli/*.py`; `pangolin_cli_admin/src/main.rs:434` returns `Ok(())` after `eprintln!`). Representative confirmed cases (not exhaustive — ~35 sites total):

**Rust admin CLI (`pangolin/pangolin_cli_admin/src/handlers/`)**
- **B_cli1. `create-catalog` silently creates a catalog with no warehouse and the wrong type.** `catalogs.rs:50-54` sends `{"name","warehouse","type":"pangea"}`; the server wants `warehouse_name`/`catalog_type` (`pangolin_handlers.rs:693-700`). Serde ignores the unknown fields, so `warehouse_name` is `None` (skips the existence check) and `catalog_type` defaults to `Local`; the required `--warehouse` flag is thrown away and the CLI prints success. **Fix:** `{"name","warehouse_name","catalog_type":"Local"}`.
- **B_cli2. All six merge commands hit non-existent routes.** `merge.rs:21,61,100/102,179,201,223` use `/api/v1/merges/...`; real routes are `/api/v1/catalogs/:catalog/merge-operations`, `/api/v1/merge-operations/:id[/conflicts|/complete|/abort]`, `/api/v1/conflicts/:id/resolve` (`lib.rs:262-282`). Every merge command 404s.
- **B_cli3. Four federated-catalog commands hit wrong paths; a fifth creates a `Local` catalog.** `federated.rs:12,28,142` should target `/api/v1/federated-catalogs/{name}/{sync,stats,test}` (`lib.rs:362-370`); `federated.rs:63-70` posts to `/api/v1/catalogs` with a `type` field the request struct lacks (→ `Local`); `:106` filters on `i["type"]=="federated"` where the response key is `catalog_type` with value `"Federated"` — `list-federated-catalogs` always prints empty.
- **B_cli4. Both token-revocation commands 404** (`tokens.rs:8,35` → `/api/v1/tokens/revoke*`; real routes are `/api/v1/auth/revoke*`, `lib.rs:435-442`) — the documented logout path never works.
- **B_cli5. `delete-user` passes a username where a UUID is required** (`users.rs:8-10` vs `Path<Uuid>` at `user_handlers.rs:381`) → 400; **`update-warehouse --id`/`update-catalog --id` key on name not id** (`warehouses.rs:241`, `catalogs.rs:92` vs `Path<String>` name); **`revoke-permission`/`request-access` use routes/methods that don't exist** (`governance.rs:286-291,472`); **`.unwrap()` panics on JSON in `resolve_scope`** (`governance.rs:136,164,189`).
- **B_cli6. Several flags are parsed then silently dropped:** `update-user --username` (`users.rs:99`), `list-audit-events --tenant-id` (`audit.rs:28`), `resolve-conflict --merge-id` (`main.rs:310`); `assign-role`/`revoke-user-role` are unreachable non-interactively (`main.rs:432` wildcard). Multiple list commands render always-blank columns from wrong keys (`i["role"]`, `i["storage_type"]`, `i["type"]`).
- **B_cli7. `ConfigManager::new(...).unwrap()` panics** when `$HOME`/`XDG_CONFIG_HOME` are unset (`main.rs:36`; the user CLI uses `?` correctly). No request timeout on `reqwest::Client::new()` (`pangolin_cli_common/src/client.rs:13`); config file with the auth token is written `0644` (`config.rs:58-66`).

**Rust user CLI (`pangolin_cli_user`)**
- **B_cli8.** `merge-branch` sends `source`/`target` where the server needs `source_branch`/`target_branch` → 422 (`handlers.rs:280`); `request-access` is a no-op that reports success (`handlers.rs:384-391`); `search` is a hardcoded placeholder despite two working endpoints (`handlers.rs:80`); `get-token` sends a null tenant + ignored `description` (`handlers.rs:399`); `generate-code` prints the live JWT into copy-paste output (`handlers.rs:94,117`).

**Python SDK (`pypangolin/src/pypangolin/`)**
- **B_sdk1.** `create_user` role default `"TenantUser"` vs kebab-case `tenant-user` → 422 (`cli/admin.py:68`); `PermissionClient.grant` emits `{"type","id"}` but `PermissionScope` uses `catalog_id`/`namespace`/`asset_id`/`tag_name` → 422 (`governance.py:37-49`); `models.PermissionScope` uses kebab-case field aliases serde never emits, so scopes parse as empty (`models.py:78-83`); `Role.permissions` typed `List[Permission]` where the server returns `PermissionGrant` → `ValidationError` on any role with grants (`models.py:92`).
- **B_sdk2.** `BusinessMetadataClient.delete(asset_id, key)` deletes *all* metadata (server ignores `key`, `governance.py:136`); `request_access` sends `motivation` where the server reads `reason` (dropped, `governance.py:140`); `FederatedCatalogClient.create` drops `uri`/`warehouse`/`credential` (`federated.py:11`); `TokenClient.generate` sends `name`/`user_id`/`expires_in_days` — all ignored, token silently 24h (`admin.py:79`); `BranchClient.rebase` is missing the required `name`, ignores `base_branch`, and raises `TypeError` on the empty success body (`git.py:60-68`).
- **B_sdk3.** Broken CLI commands that crash on every call (masked by blanket `except`): `admin grant-permission` (`TypeError`, `cli/admin.py:209`), `user merge-branch` (`TypeError`, `cli/user.py:191`), `admin list-warehouses` (`AttributeError: no 'id'`, `cli/admin.py:150`), `user search` (`AttributeError: no 'score'`, `cli/user.py:80`).
- **B_sdk4. Secret handling:** connection-asset encryption key is stored inline next to its ciphertext by default (`assets/connections/base.py:68,101`); CLI writes the JWT to a `0644` `~/.pangolin/profiles.yaml` (`cli/config.py:33`); `generate-code`/`get-token` echo the raw token (`cli/user.py:94,252`).
- **B_sdk5. No console-script entry point** (`pyproject.toml` has no `[project.scripts]` despite docs calling the CLI "installed automatically"); no request timeout / `Session` reuse (`client.py:115`, `auth.py:14`); no `py.typed`; leftover `print("DEBUG CLIENT: …")` in `git.py:19`.

Because these fail through blanket exception handlers with a zero exit code, scripts and CI cannot detect them — which is exactly why they have survived. **Fix (all):** a `wiremock`/`responses` contract-test per client method asserting path + payload against the router, `#[serde(deny_unknown_fields)]` on every server request struct so wrong field names 422 loudly, and generating the CLI/SDK request types from the existing `openapi::ApiDoc` instead of hand-writing them three times. Make every CLI command exit non-zero on failure.

### Medium — API layer

**B42. Management-API pagination is applied before permission filtering.**
- `pangolin/pangolin_api/src/pangolin_handlers.rs:745-786` (`list_catalogs`): the store paginates (`limit`/`offset`), then `authz_utils::filter_catalogs` removes unauthorized rows. A `TenantUser` gets variable-size pages, including **empty pages while more authorized data exists** — clients that stop on an empty page silently miss data. The same fetch-then-filter pattern applies to sibling list handlers.
- **Fix:** either filter in the store query (pass the permitted set down) or paginate after filtering; return an explicit `next_offset`/`next_page_token` so emptiness is unambiguous.

### Low — hygiene & docs

**B43. `docs/environment-variables.md` documents variables that don't exist and misses ~20 that do.**
- It lists `PANGOLIN_HOST`, `PANGOLIN_PORT`, `PANGOLIN_STORE_TYPE` — none are read by the server (the real set includes `PANGOLIN_BIND_ADDRESS`, `PANGOLIN_STORAGE_TYPE`, `PANGOLIN_MAX_BODY_BYTES`, `PANGOLIN_REQUEST_TIMEOUT_SECS`, `PANGOLIN_CORS_ALLOWED_ORIGINS`, `PANGOLIN_WAREHOUSE_CACHE_TTL_SECS`, `PANGOLIN_SESSION_TTL_SECS`, `PANGOLIN_METRICS_ENABLED`, all OAuth vars, etc. — verified against `grep "PANGOLIN_" pangolin_api/src`). `docs/getting-started/env_vars.md` is closer but also incomplete.
- **Fix:** regenerate the reference from `pangolin_api/src/config.rs` (single source of truth); delete or redirect the stale file.

**B44. A live PyPI API token sits in plaintext in the repo-root `.env`.**
- `/home/alexmerced/development/personal/Personal/library/2026/pangolin/.env` contains `PYPI_token=pypi-AgEI...`. Verified **not** tracked by git and never in history — but it is one `.gitignore` edit (or one `git add -f`) away from leaking, and any local tool with repo read access can exfiltrate it.
- **Fix:** rotate the token now, remove it from `.env`, and keep publish credentials in a keyring / CI secret (`pypangolin/PUBLISHING.md` already describes the `secrets.PYPI_API_TOKEN` flow).

**B45. Debug debris is committed to git.**
- Tracked: `pangolin/.test_output.txt`, `pangolin/logs/api_log.txt`, `pangolin_ui/check_catalogs_list.txt`, `check_fed_cat.txt`, `check_final.txt`, `check_service_users*.txt` (~260 KB), plus untracked-but-present `pangolin/logs/*.log`, `video/render.log`, and two stale monoliths `pangolin_store/src/memory.rs.bak` / `mongo.rs.bak` (~4k lines of divergent query copies — a grep trap).
- **Fix:** `git rm --cached` the tracked ones (ignore patterns already exist; they were added before the ignore), delete the `.bak` files, add `logs/`, `test-results/`, `playwright-report/` to `pangolin_ui/.gitignore`.

**B46. UI package/test plumbing defects.**
- `package-lock.json:3` still says `0.1.0` (root `package.json` is 0.6.0); `@vitest/coverage-v8` missing so `npm run test:coverage` fails; no `test:e2e` script despite Playwright specs; `playwright.config.ts:11` targets port 5175 while `vite dev` serves 5173 and `webServer` is commented out; 18 unused runtime deps (all `@smui/*`, `marked`, `material-icons`) bloating the Docker image; `client.test.ts:17-20` mocks lack `.text()` so all 5 "passing" tests actually exercise the error path; `src/tests/CatalogsList.test.ts:44-47` asserts on a button that no longer exists.
- **Fix:** regenerate the lockfile, add the coverage dep and `test:e2e` script, align Playwright port + enable `webServer`, prune deps, fix the fetch mocks.

---

## Recommended Improvements

Ordered by leverage.

0. **Add a permission-matrix (authz) test and a client↔server contract test — the two highest-leverage additions.** Every bug in the "API authorization bypasses" cluster (B0a-B0m) and the "contract drift" cluster (B_cli*/B_sdk*) is invisible to the current CI: it compiles, it's formatted, it's lint-clean, and the unit tests pass — because nothing asserts *who is allowed to call what* or *whether a client's request matches the router*. Add (a) a table-driven test that, for each mounted route, drives it as `Root`/`TenantAdmin`/`TenantUser`/wrong-tenant/service-user and asserts the expected 200/403 — this catches every missing `check_permission`; and (b) a `wiremock`/`responses` contract suite for both CLIs and the SDK that asserts each method's path + payload against the real handler. Pair with `#[serde(deny_unknown_fields)]` on every server request struct so wrong field names fail loudly instead of defaulting.

1. **Build a cross-backend parity test harness and make it the gate.** Nearly half the bugs above (B1-B7, B17-B30) are one backend silently diverging from the others. A single test suite that runs every `CatalogStore` method against all four backends and asserts identical observable behavior (including sort order, pagination determinism via the "two pages = whole set" property, serde round-trips of *every* enum variant, and tenant isolation on every read) would have caught all of them and will keep them fixed. Wire it into the existing `ci.yml` services matrix. `docs/operations/backend-parity.md` can then be generated from the suite instead of maintained by hand.
2. **Add pypangolin and pangolin_ui jobs to CI.** `.github/workflows/ci.yml` covers only Rust, Helm, and Docker. Add: `pytest` (after B41 makes collection possible), `ruff` + `mypy` for the SDK; `npm run check`, `vitest`, and Playwright (after B46) for the UI. The UI/SDK are exactly where 0.6.0's "no CI → silent rot" lesson is currently repeating.
3. **Fail-fast config validation for deployment artifacts.** A CI step that runs `docker compose config` on every compose file and boots the API container with the compose-provided env (catching B8/B9/B10-class drift), plus a script that greps compose/Helm/docs for `PANGOLIN_*` names and diffs them against `config.rs`.
4. **Unify the error envelope.** Management API emits flat `{"error": "<string>"}` while Iceberg handlers emit the spec envelope; the UI already guesses (`errorData.error || errorData.message`, `client.ts:54`) and renders `[object Object]` for structured errors. Standardize on one management envelope (message + code + request_id), and finish migrating handlers off bare `(StatusCode, &str)` tuples.
5. **Reduce panic surface.** 92 non-test `unwrap()` in `pangolin_api`, 38 in `pangolin_store`, including 14 `unwrap()`s on database-supplied BSON in the Mongo backend (`mongo/branches.rs:58-105`, `mongo/assets.rs:82-190`, `mongo/business_metadata.rs:104-218`, etc.) where a single legacy document panics the request. Convert to typed errors; then drive the clippy budget from 36 to 0 and flip CI to `-D warnings`.
6. **Token handling:** store only hashes of session tokens at rest (`sqlite/tokens.rs:11-19`, `mongo/tokens.rs:66-75` currently persist and even return raw tokens via `list_active_tokens`) — the `service_users.api_key_hash` pattern already exists. On the UI side, move toward an `httpOnly` cookie session (or at minimum short TTL + revoke-on-logout) instead of long-lived JWTs in `localStorage`.
7. **SDK ergonomics:** `requests.Session` with retry/backoff (429/5xx), `raise ... from e`, Pydantic `ConfigDict` migration (deprecation warnings at `models.py:132,196`), and typed pagination helpers that iterate all pages.
8. **Docs generation over hand maintenance:** env-var reference from `config.rs` (B43), Iceberg endpoint coverage table from the router, and a single CHANGELOG-driven version bump script that touches `Cargo.toml`, `pyproject.toml`, `__init__.py`, `package.json` + lockfile, and the Helm chart in one commit (B38/B46 show the manual process already drifting one day after 0.6.0).
9. **Known-limitation burndown (README already admits these; they are the right next hardening targets):** per-IP/per-account rate limiting on `/api/v1/users/login` and the token endpoints (e.g. `tower_governor`); encrypt warehouse cloud credentials at rest (envelope encryption, KMS-or-env master key); make branch-create-by-copy transactional on Postgres.
10. **UI cleanup pass:** remove dead code (`src/lib/stores.ts`, `app.scss`, the unauthenticated file-read route `src/routes/api/docs/[...path]/+server.ts` — broken in Docker, no callers, blacklist-based traversal guard), strip ~25 `console.log`s (one logs a token prefix, `stores/auth.ts:76`), replace native `alert()`/`confirm()` with the existing `ConfirmDialog`, and delete the "unresolved deliberation" comments that describe the very bugs above (`routes/login/+page.svelte:204-215`, `routes/permissions/+page.svelte:6-12`).

---

## Recommended New Features

1. **Complete the Iceberg REST surface** (the README's own "not implemented" list): `loadNamespaceMetadata` (GET namespace), `namespaceExists` (HEAD), `registerTable`, `commitTransaction` (multi-table atomic commits), and the rest of the view API (list/drop/replace/exists/rename). `registerTable` + full views are the cheapest wins; `commitTransaction` builds directly on the now-solid `commit.rs` requirement machinery and would be a genuine differentiator at this maturity level.
2. **Table maintenance as a first-class service:** scheduled snapshot expiration, orphan-file detection/cleanup, and manifest compaction driven from `pangolin_core/src/maintenance.rs`, surfaced in the UI and CLI. Pairs naturally with fixing `metadata-log` (B13) since expiration needs the metadata history.
3. **Backup/restore + disaster recovery tooling:** a `pangolin-admin backup`/`restore` command per backend (pg_dump-wrapping for Postgres, file copy for SQLite, mongodump for Mongo) with a documented, *tested* RPO/RTO — the README currently ships "undocumented and untested" for this row, and several bugs above (B3, B21) make backups the only recovery path.
4. **Shared-state option for multi-replica deployments:** a Redis (or Postgres-advisory-lock) backing for the warehouse cache, OAuth nonce store, and token-cleanup job coordination — converting the README's "HA unproven" row into a supported topology. The cache already documents its node-locality (`cached_store.rs:30,45`); this is the designed-for next step.
5. **Full OIDC:** PKCE, `id_token` validation via JWKS, provider discovery, and an `email_verified` check — closing the documented OAuth gaps in `docs/operations/oidc.md` and letting the UI's provider buttons render from a real `GET /api/v1/oauth/providers` capability endpoint (B33).
6. **Webhooks / event stream on catalog changes:** the audit pipeline already classifies 40+ actions across 19 resource types; emitting them to configurable HTTP/queue sinks enables cache invalidation for downstream engines, data-product notifications, and CDC-style integration — a feature none of the small OSS catalogs do well.
7. **Real search:** replace `LIKE '%q%'` with Postgres `pg_trgm`/`tsvector` (and equivalent per-backend strategies) with ranked results, and expose facets (asset type, tags, namespace) — the business-catalog feature set already stores the metadata; the query layer is the missing piece (and B28 shows the current one is inconsistent anyway).
8. **Load-test harness + published capacity numbers:** a `k6`/`goose` scenario pack (login, listing, commit contention, credential vending) run in CI-nightly, publishing p50/p99 and a measured capacity model — directly addresses the README's "no published performance figures" and would catch regressions like the pre-0.6.0 bcrypt scan.
9. **Soft-delete / recycle bin for catalogs and branches:** given that cascading deletes are the most dangerous operations in the system (B3, B21, and the documented non-atomic branch copy), a two-phase delete (mark, then purge after N days) is cheap insurance and a strong operator-trust feature.
10. **`pypangolin` async client (`httpx.AsyncClient`)** sharing the same models, plus a documented PyIceberg-compat test matrix in CI (the emulator compose file already exists at `docker-compose.emulators.yml` — wire it to a nightly job).

---

*Method note: findings above were verified directly against the working tree (file:line citations checked on 2026-08-10), with `cargo test --workspace`, `cargo fmt --check`, `cargo clippy` executed locally, plus targeted deep-dives across `pangolin_store` (all four backends), `pangolin_api` (Iceberg + management handlers), `pangolin_ui`, and `pypangolin`.*
