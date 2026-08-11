# Changelog

All notable changes to Pangolin are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/) — with the caveat that
Pangolin is pre-1.0, so a minor release may contain breaking changes.

From 0.6.0 the server, both CLIs, the Python SDK, the UI and the Helm chart all
carry the same version number. Before that they had drifted to five different
values and there was no way to tell which combination had been tested together.

## [Unreleased]

Bucket 2 of the production-readiness work.

### Added — warehouse credentials encrypted at rest (C-11)

A warehouse holds the credentials Pangolin uses to reach a customer's object
storage. They were plaintext JSON in the catalog database, so anything that
could read one row of `warehouses` — a backup, a replica, a snapshot, an analyst
with `SELECT` — held every tenant's cloud keys.

Credential fields are now sealed with AES-256-GCM and a fresh 96-bit nonce per
value, stored as `enc:v1:<base64>`. Only credentials are sealed; bucket, region,
endpoint and account name stay readable, because the object-store factory
compares and concatenates them and they are not secrets.

Deliberate choices worth knowing about:

- **Off unless `PANGOLIN_ENCRYPTION_KEY` is set**, and the server warns loudly
  at startup when it is not. Requiring it would break every existing deployment
  on upgrade; doing nothing silently is the failure mode this audit keeps
  finding, so it is said out loud instead.
- **Reads tolerate plaintext**, so a database written before this exists keeps
  working. Those rows stay unsealed until something rewrites them —
  `docs/operations/encryption.md` explains how to force that and how to find
  what still needs it.
- **The wrong key fails loudly.** GCM authenticates, so a mismatched key gives
  an error naming `PANGOLIN_ENCRYPTION_KEY` rather than returning rubbish.
- **This protects a stolen database, not a compromised host.** The key is in the
  server's environment. That limit is documented rather than implied away.

PostgreSQL, SQLite and MongoDB seal on write and open on read, on both the
create and the update paths — sealing only on create would protect the first
credential and leak every rotation, which is worse than not doing it at all,
because the table would look encrypted. The memory backend is excluded on
purpose: it loses everything on restart, so it has no "at rest".

`warehouse_encryption_tests.rs` reads the raw stored bytes through its own
database connection rather than through the store, and asserts the plaintext is
absent. Asking the store to read back its own writes would pass just as happily
if `seal` were never called.

### Added — rate limiting on the authentication endpoints (C-5)

The login endpoint had no throttle of any kind and was brute-forceable. There
were global concurrency and body limits and a request timeout, but nothing made
the thousandth password guess cost more than the first. Bcrypt slowed each
attempt, which raises the price of a broad campaign and does nothing against a
targeted guess at one weak password - while making the endpoint an efficient way
to burn the server's CPU.

Throttled on two keys, because either alone has a blind spot:

* **by source address** — bounds one attacker working through many accounts;
* **by account** — bounds many addresses working on one account, which is the
  shape of a credential-stuffing run and which a per-address limit cannot see.

Both are checked before any password verification, so a refused attempt costs a
cache lookup rather than a bcrypt round. A successful login clears the account's
counter, so mistyping a password twice and then getting it right does not leave
you near the limit. Refusals answer `429` with `Retry-After` and increment
`pangolin_auth_throttled_total`, which is worth alerting on.

`X-Forwarded-For` is honoured **only** when `PANGOLIN_TRUST_FORWARDED_FOR=true`.
Trusting it unconditionally would let a caller set a fresh value per request and
bypass the per-address half entirely - protection that reads as protection and
is not.

Configuration: `PANGOLIN_AUTH_RATE_LIMIT` (default 10, 0 disables),
`PANGOLIN_AUTH_RATE_WINDOW_SECS` (default 60), `PANGOLIN_TRUST_FORWARDED_FOR`
(default false).

Known limitation, stated rather than buried: the counters are in-process, so the
limit is **per replica**. With N replicas an attacker gets N times the budget.

`main` now serves through `into_make_service_with_connect_info`, without which
the peer address is not available and every attempt would share one bucket.

## [0.7.0] — 2026-08-10

Implements `roadmap_aug10.md`, the full-repo audit of 2026-08-10. **This is a
security release.** Five of the fixes below are exploitable by any
authenticated principal, including the lowest-privilege tenant user and any
service-user API key. If you run 0.6.0, upgrade and rotate tokens.

### Security

- **Any authenticated caller could mint a `Root` JWT for any tenant (B0a).**
  `POST /api/v1/tokens` took no session at all and mapped a body-supplied
  `roles: ["Root"]` straight into signed claims. Since `check_permission`
  short-circuits for `Root`, this was a total privilege escalation reachable by
  a tenant user. Minting is now restricted to `Root`, or to a `TenantAdmin`
  within its own tenant and never above its own rank.
- **Any tenant member could vend read+write cloud credentials for the whole
  warehouse (B0b).** The credential endpoint performed no authorization, never
  looked the table up, and hardcoded `["read", "write"]` - so it issued
  credentials for tables the caller had no rights to and that need not exist.
  It now resolves the asset, requires `Read`, and adds `"write"` only when
  `Write` is actually held.
- **Logout did not revoke anything (B0j).** Revocation is keyed by the token's
  `jti`; the handler revoked `session.user_id`, which no token ever carries as
  its `jti`. Logout returned 200 and the token kept working for its full
  24-hour lifetime. `UserSession` now carries the `jti`.
- **An expired service user could renew indefinitely (B0g).** The Iceberg OAuth
  token endpoint checked `active` but not expiry, so an expired API key could
  still exchange `client_credentials` for a fresh JWT - bypassing key expiry
  entirely, and renewably.
- **`PANGOLIN_DEV_MODE` waived the `NO_AUTH` public-bind guard (B0h).** The two
  flags are routinely set together in compose and dev setups, and together they
  started a server on `0.0.0.0` that treated every anonymous request as
  `TenantAdmin`. Dev mode now relaxes secret strength only, never exposure.
- **A tenant-wide grant applied across tenants (B0i).** `PermissionScope::Tenant`
  matched without comparing the grant's tenant to the resource's.
- **OAuth linked accounts by unverified email (B0l).** Anyone who could set a
  matching address on any configured provider - GitHub reports unverified ones -
  logged in as that Pangolin user, including the seeded tenant admin. Identity
  is now `(provider, subject)`; email linking needs a verified address and an
  operator domain allowlist.
- **The OAuth login flow could not complete (B0k).** `POST /api/v1/oauth/exchange`
  was not in the public-path allowlist, so the endpoint whose job is to issue
  the first token demanded one. The browser landed with a `?code=` it could
  never redeem.
- Missing authorization on `rename_table` (B0c), `update_namespace_properties`
  (B0d), view create/read (B0e), `perform_maintenance` (B0f), `rebase_branch`
  and `delete_business_metadata`. `perform_maintenance` additionally ran
  destructive snapshot expiry against a hardcoded `"default"` catalog rather
  than the one in the path.
- A caller-supplied `expires_in_hours` could panic token issuance and abort the
  connection task (B0m); clamped, plus a `CatchPanicLayer`.
- A malformed or absent `jti` skipped the revocation check entirely, making such
  tokens unrevocable for their lifetime (B0o).
- The admin CLI and Python SDK wrote their auth tokens world-readable, and
  `generate-code` / `get-token` echoed live JWTs into copy-paste output
  (B_cli7, B_sdk4).
- A live PyPI API token was removed from the repo-root `.env` (B44). **It was
  present in plaintext and must be rotated.**

### Fixed

- **Storage backends disagreed with each other in ten ways (B1-B7, B17-B30).**
  A cross-tenant audit read on Mongo, a revocation that was a silent no-op on
  Mongo, a SQLite branch delete that orphaned its assets and referenced a
  column that does not exist, a Postgres search that panicked on any hit, a
  Mongo compare-and-swap that lost Iceberg snapshots, a memory index that
  broke another tenant's lookups, and all three persistent backends silently
  rewriting 15 of the 17 asset types to `IcebergTable`. Plus pagination that
  could repeat or skip rows everywhere, and four different answers to the same
  search.
- **Two defects the new parity suite found on its first run.** `SqliteStore`
  had no inherent `revoke_token`/`is_token_revoked`, so the trait delegations
  called themselves - revoking a token on SQLite recursed until the stack was
  exhausted and *aborted the process*. And the SQLite `audit_logs` table still
  declared its original column set while the code inserted the full entry, so
  every audit write failed and the backend kept no audit trail at all.
- **Iceberg metadata was not spec-conformant (B11-B16o).** `default-spec-id`
  was written under the wrong name, the required `last-partition-id` was
  absent, schemas omitted `"type": "struct"`, and `metadata-log` was never
  appended - so metadata Pangolin wrote could not be read as v2 metadata by an
  external engine. On the commit path: nested namespaces registered under one
  key and looked up under another (every commit to one 404'd), a client could
  jump the sequence counter to `i64::MAX` and overflow the next commit, a
  feature-branch commit moved `main`, `last-updated-ms` only advanced on
  snapshots, `-1` resolved against the whole list rather than what the commit
  added, `create_table` returned the table directory as `metadata-location`
  and dropped every complex-typed column from the schema, and lost
  compare-and-swaps orphaned metadata files.
- **`docker compose up` could not start the API (B8-B10).** No signing secret
  was set and the server has refused to start without one since 0.6.0. Both
  compose files also set a storage variable nothing reads, and the release
  compose file pinned an image four versions old and ran a script that does
  not exist.
- **The management UI was disconnected from the server (B31-B37).** Four
  spellings of the API base URL coexisted and none agreed, so every deployed
  build called the visitor's own localhost; ~13 raw `fetch('/api/v1/...')`
  calls 404'd outside the dev proxy and skipped the tenant header; three
  endpoints the UI called did not exist; nothing handled a 401, so an expired
  token left a permanently broken session; tag-filtered search 400'd end to
  end; there was no way to create a catalog from the catalogs page; and the
  tenant switcher was dead code referencing an unimported store.
- **Both CLIs and the SDK called endpoints that do not exist (B_cli1-8,
  B_sdk1-5).** Roughly 35 sites: wrong paths, wrong field names, wrong types,
  commands that were `Ok(())` stubs reporting success. All of it survived
  because every command swallowed its error and exited 0.

### Fixed — found by running the suites against live databases

The parity suite was written against memory and SQLite, the two backends CI
could run without a service container. Pointing it at a live PostgreSQL and
MongoDB for the first time failed on both. None were regressions; all had been
present for as long as the code had.

- **PostgreSQL: asset search was broken outright.** No migration ever created
  `business_metadata`, while `search_assets` joined it — so every search failed
  with `relation "business_metadata" does not exist`, a hard SQL error rather
  than an empty result. The three CRUD methods were unimplemented, so the
  trait's "Operation not supported by this store" default answered them. Added
  the migration and the implementation.
- **MongoDB: role assignments were unreadable.** `bson::to_document` writes a
  `Uuid` as a string while the deserializer expects BSON Binary, so
  `assign_role` wrote documents that `get_user_roles` could never match and
  that could not be deserialized at all. Every role-derived permission silently
  vanished: **a user holding an admin role was authorized as though they held
  none.** The same asymmetry caused B1 and B2 in two other collections; one
  helper now covers all of them.
- **MongoDB: `get_metadata_location` had no fallback** to the asset's own
  `location`, unlike the other three backends. A table created with a location
  but no explicit metadata-location property reported none, so its metadata
  could not be loaded and its commits compared against a different value than
  the read path returned.
- **MongoDB: the "no transaction support" fallback was unreachable.**
  `start_transaction` is a local call in the Rust driver and cannot fail for
  want of a replica set; the error arrives on the first operation *inside* the
  transaction and was propagated rather than caught. `delete_catalog` failed
  outright on any standalone `mongod` instead of degrading as its own comment
  promised.
- **SQLite: the `audit_logs` fix did not reach existing databases.** The schema
  file is written with `CREATE TABLE IF NOT EXISTS`, which does nothing when the
  table already exists — so fresh installs got the corrected columns and every
  upgraded database kept the broken ones, with a bumped version number now
  claiming otherwise. Added a real v1→v2 migration, keyed off table
  introspection rather than the recorded version, with the old table preserved
  as `audit_logs_pre_v2`.

### Fixed — the MongoDB UUID encoding audit

The string/Binary asymmetry above had by then been fixed four times, in four
collections, each time as its own bug. Auditing every collection at once — with
a round-trip test per entity rather than per feature — found four more, and a
second encoding disagreement nobody had noticed.

There are three ways this codebase converts a `Uuid` to BSON and they all differ:
`to_bson_uuid` gives Binary with the generic subtype, `doc! { "k": uuid }` gives
Binary with the *UUID* subtype, and `bson::to_document` gives a string. Reads
disagree too: a typed `Collection<T>` demands binary, `bson::from_bson` demands a
string. A write and a read chosen independently agree only by luck, and when they
do not, nothing fails loudly — the filter just matches nothing.

- **Every service-user method was a no-op.** `create_service_user` let Mongo
  generate an `ObjectId` while the four by-id methods filtered on
  `{"_id": <uuid string>}`, which matched nothing; the tenant listing and the
  API-key lookup used snake_case field names for a kebab-case struct; and
  `update_service_user_last_used` wrote to a field no reader looks at. The
  consequence that matters: **API-key authentication could never resolve a
  service user on MongoDB.** It fails closed, so this was an outage of
  service-user auth rather than a bypass. Changing a role also wrote the Rust
  variant name instead of its serde form, making the record unreadable
  afterwards.
- **Business metadata could be written but never read.** Only `asset-id` was
  rewritten as Binary; `id`, `created-by` and `updated-by` kept the string form,
  so `get_business_metadata` failed on the first of them. Writing metadata made
  an asset's metadata permanently unreadable.
- **Listing active tokens failed outright.** `store_token` writes timestamps as
  BSON DateTime, which `bson::from_bson::<DateTime<Utc>>` rejects — chrono wants
  an RFC3339 string. The `created_at` arm swallowed the same error and
  substituted `now()`, so even without the hard failure every token would have
  reported the listing time as its creation time.
- **A branch with a head commit could not be read.** `create_branch` writes the
  head through `doc!` (Binary, UUID subtype) and the reader accepted only a
  string. A freshly created branch has no head, so this only bit once a branch
  had been committed to — which is why it survived every existing test.

`from_bson_uuid` now accepts all three encodings, so records already written by
any of them still load, while writes go through `to_bson_uuid` alone.
`mongo_uuid_round_trip_tests.rs` covers all 21 collections and runs against both
MongoDB topologies in CI.

### Fixed — test environment drift

- **`docker-compose.db-test.yml` had no object store.** The store compliance
  tests exercise file IO; with no S3 they fall through to the EC2
  instance-metadata endpoint, hang for eleven seconds and fail with a
  credentials error that names nothing relevant. CI had MinIO and the documented
  local workflow did not, so the two disagreed about what it takes to run the
  suite.
- **The MinIO image CI pulled no longer exists.** `bitnami/minio:latest` was
  withdrawn from Docker Hub and now fails with `manifest unknown`. Both CI and
  the compose file use `minio/minio` with an explicit bucket-creation step —
  `warehouse` for the application, `bucket` and `test-bucket` for the compliance
  tests, whose absence surfaces as `NoSuchBucket`.

### Fixed — the management UI

The UI job in CI built the app and never ran its tests, so the suite had drifted
to **40 failures across 14 files**. Most could not have passed: the global test
setup replaces `$lib/api/catalogs`, `$lib/api/warehouses`, `$lib/stores/auth`,
`$lib/stores/tenant` and `$lib/stores/notifications` with stubs, so the unit
tests *for those modules* were asserting against the stub rather than the code.
Others were asserting behaviour the app no longer had. Running them turned up
real defects underneath:

- **A root user could not create another root user.** The role option's value
  was `Root`; the server's `UserRole` is kebab-case, so the request was
  rejected. The same PascalCase leftovers meant a tenant admin was shown an Edit
  control for root users (`row.role !== 'Root'` never matched), and the role
  badge colours on the users page keyed off `Root`/`TenantAdmin`, which the API
  never returns.
- **A warehouse created in the UI showed no bucket in the UI.** The list page
  read `s3.bucket`/`azure.container`; the create form writes plain
  `bucket`/`container`. Both conventions are now accepted on read, as the server
  already does. The warehouse table also rendered its Type column twice, in
  place of the Region column its own template already had a branch for.
- **`production` is not a branch type.** The API knows exactly `ingest` and
  `experimental`, but the UI's types declared `'experimental' | 'production'`
  and the green badge keyed off `production` - so every branch rendered as
  though it were experimental, and `ingest`, the type that carries a distinct
  permission, had no representation at all.
- **A branch with no recorded parent was displayed as branching from `main`**,
  claiming a lineage the data does not contain.
- **A local catalog could be created with no storage location and no
  warehouse**, leaving it with nowhere to write its tables. The server accepts
  it (the field is optional there, for federated catalogs), so nothing rejected
  it.
- **The role select on the user edit page had no accessible name** - the label
  named nothing - and its fallback value, `TenantUser`, matched none of the
  options, so a user record without a role showed an empty select.
- **`logout()` could throw**, skipping the caller's redirect and stranding the
  user on a page they were no longer authenticated for, if the token-revocation
  call misbehaved. Its `void`/`.catch` pair only covered a rejected promise.
- Every unparameterised list call left a bare `?` on the end of its URL.

`DataTable` moved from `createEventDispatcher` to callback props, with its six
consumers. That is the Svelte 5 idiom, and it is what makes the component
testable at all: `component.$on(...)` was removed in Svelte 5, so the row-click
test had been left as a stub that asserted nothing.

CI now runs `npm test`, and carries a `svelte-check` error budget on the same
ratchet as the clippy one. `svelte-check` errors fell from 166 to 150 - mostly by
giving `StorageConfig` the index signature the server's free-form
`HashMap<String, String>` always implied.

Note that the app runs in Svelte 5's **legacy mode**: none of its 90 components
use runes. That is supported and works; converting them is a separate piece of
work and has not been done here.

### Fixed — the cloud-credential features had never compiled

Found while bumping dependencies: `cargo check -p pangolin_api --features
cloud-credentials` fails, and had been failing at every version. So did each of
`aws-sts`, `azure-oauth` and `gcp-oauth` individually. Nothing built with
`--features`, so nothing noticed.

For a catalog whose job includes vending scoped, time-limited cloud
credentials, that is the feature set. Every deployment using it was running
without it.

The errors were the kind that only appear when a `cfg` block is never
type-checked:

- parameters bound as `_duration`, `_resource_path`, `_permissions` to silence
  unused warnings in the default build, then referenced as `duration`,
  `resource_path`, `permissions` inside the feature block — three files, five
  bindings;
- `anyhow!` used in `gcp_signer.rs` with only `anyhow::Result` imported;
- `creds.expiration()` fed to `chrono::DateTime::parse_from_rfc3339`, but it
  returns an `aws_smithy_types::DateTime`, not text — so the STS credential
  expiry was parsed from a value that was never a string.

`aws-sdk-sts` was also pinned to an exact `=1.50.0` with no comment. It was
protecting nothing — the feature failed identically at that version — and it
blocked `aws-config` from reaching a release that drops the second, vulnerable
TLS stack. Relaxed to `1.109`.

A `features` CI job now builds each optional feature, and `pangolin_store`'s
`azure` and `gcp` backends alongside them.

### Changed — minimum supported Rust version is now 1.94

Raised from 1.92 to pick up the AWS SDK releases carrying fixed `aws-lc-sys`
and `rustls-webpki`. That is what cleared the certificate-validation
advisories — two of them high severity, on the path Pangolin uses to reach S3,
Azure Blob Storage and GCS.

`rust-version` is a promise to consumers and nothing verified it: every job ran
`stable`. An `msrv` job now reads the declared version out of the workspace
manifest and builds with exactly that toolchain, so the floor is checked rather
than asserted. `Dockerfile`, `README.md`, `CONTRIBUTING.md` and the deployment
guide are all in step.

### Security — dependency advisories

`cargo audit` reported 26 vulnerabilities, the one job still red after the CI
repair. Now zero, by a combination of upgrades and eight deliberate,
individually justified exceptions in `.cargo/audit.toml`.

Cleared by upgrading: the `aws-lc-sys` cluster (including two high-severity
certificate-validation bypasses and a PKCS7 signature-validation bypass),
`rustls-webpki` name-constraint and CRL-parsing defects, `quinn-proto`,
`hickory-proto`, `bytes`, `crossbeam-epoch`, `time`.

Accepted, with the reason recorded against each ID: `rsa`'s Marvin attack
(never compiled — `sqlx-mysql` is an optional dependency this workspace does
not enable), `quick-xml` (held by `object_store` 0.11 and by `azure_core` 0.20,
which is behind an optional feature), the second `rustls-webpki` copy that
arrives via the AWS SDK's `rustls` 0.21, `http-types`, and `rand`'s
custom-logger unsoundness. The ignore list names specific advisory IDs, so a
new advisory — including a new one against these same crates — still fails CI.

### Fixed — a regression caught by the release gate

Found by running the new release smoke test against the built 0.7.0 image, and
fixed before 0.7.0 shipped.

- **The server exited 25 seconds after startup, having received no signal.**
  The B16n change meant to bound the shutdown *drain* was written as
  `tokio::time::timeout(shutdown_grace, serve)`. `serve` is the whole server,
  not the drain, so it bounded the lifetime of the process:
  `PANGOLIN_SHUTDOWN_GRACE_SECS` became a countdown to a clean exit rather than
  a limit on how long draining may take. Every container would have
  crash-looped. The deadline is now armed inside `shutdown_signal`, only once a
  signal has actually been seen.

  All 18 CI jobs passed with this present, as did the full workspace suite:
  nothing ran the binary for longer than the 25-second default. The `docker`
  job now starts the built image with a 5-second grace, waits 20 seconds, and
  fails if it is no longer serving - then checks it still stops promptly when
  told to.

- **Release verification inherited the developer's `.env`.** Compose auto-loads
  it, so a local `PANGOLIN_ROOT_USER` / `PANGOLIN_ROOT_PASSWORD` fed the
  harness - and where that password is a placeholder, the server's config guard
  refuses to start, so verification failed for reasons having nothing to do
  with the artifact. The harness now takes `RELEASE_*` names that cannot
  collide with a real deployment's.

- **The release stack shared a Compose project with the development database
  stack**, both deriving `pangolin` from the directory name, so `up` and
  `down -v` in one stopped containers belonging to the other. It now declares
  `name: pangolin-release`, and its MinIO no longer publishes host ports it
  never used.

### Added

- **A permission matrix test (improvement #0).** Drives each sensitive route as
  Root / tenant admin / ungranted tenant user / foreign tenant admin and
  asserts the expected 200 or 403. Every bug in the authorization cluster was
  invisible to CI precisely because nothing asserted this.
- **A cross-backend parity suite (improvement #1).** Runs the same assertions
  against memory, SQLite, Postgres and Mongo. Nearly half the storage findings
  were one backend diverging from the others, which no per-backend test can
  see.
- `#[serde(deny_unknown_fields)]` on all 34 server request structs, so a client
  sending the wrong field name gets a 422 naming it rather than a 200 for a
  request the server silently emptied. It caught five such payloads in the
  project's own tests immediately.
- CI jobs for the Python SDK, the UI, configuration drift, and the two
  guardrail suites above (improvements #1-#3). Previously CI covered only Rust,
  Helm and Docker - which is exactly where the SDK and UI rot happened.
- `loadNamespaceMetadata`, `namespaceExists`, `DELETE /api/v1/branches/{name}`
  and `GET /api/v1/oauth/providers`; spec pagination (`pageToken`/`pageSize`
  and `next-page-token`) and the spec error envelope across the Iceberg
  handlers.
- `scripts/check_env_var_docs.sh`, which regenerates the environment-variable
  reference check from `config.rs`. The old page documented three variables
  that do not exist and omitted 34 that do (B43).
- `scripts/bump_version.sh`, which sets the version across all five artifacts
  and their inter-crate requirements, with a `--check` mode wired into CI
  (improvement #8). The "one version everywhere" property had already drifted
  in two places a day after the release that introduced it.
- CI now runs the parity suite against live PostgreSQL and MongoDB service
  containers, and **fails if any backend was skipped**. A skipped backend
  passing silently is how two of them went untested through a security release.
- An upgrade guide at `docs/upgrading/0.6-to-0.7.md`, and a security advisory
  in `SECURITY.md`.

### Removed

- 18 unused UI runtime dependencies, ~260 KB of tracked debug output, two
  ~4k-line `.bak` store monoliths that any grep of the storage layer would hit,
  27 `console.log` calls (one logging a bearer-token prefix), and an
  unauthenticated file-read route in the UI with no callers and a
  blacklist-based traversal guard.

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

- **Transactions for the multi-statement admin paths (A-24, partial).** The
  PostgreSQL and MongoDB backends used zero transactions, so any failure midway
  through a cascading delete or a merge left the catalog partially applied.
  PostgreSQL now wraps `delete_catalog` (five statements), `delete_branch` (two)
  and `merge_branch` (three) in a transaction; MongoDB wraps `delete_catalog` in
  a session-backed transaction where the deployment supports one, falling back
  to sequential deletes with a warning against a standalone `mongod`. Two bugs
  surfaced while writing the regression tests: `delete_branch` deleted assets by
  a column that had been renamed, so the statement failed every time and
  deleting a branch orphaned its assets; and each backend carried an inherent
  `merge_branch(target, source)` alongside the trait's `merge_branch(source,
  target)`, which Rust resolves in favour of the inherent method — the inherent
  one is renamed `merge_branch_into` and takes the trait's argument order.
  **Branch creation by copying assets is still not atomic.**
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
  nothing had compiled the test code in a long time. It now runs **334 tests,
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

- Transactional **branch creation by copy** — the remainder of A-24. PostgreSQL
  now wraps `delete_catalog`, `delete_branch` and `merge_branch` in a
  transaction, and MongoDB wraps `delete_catalog` where the deployment supports
  a session, but copying assets into a new branch is still a sequence of
  independent statements.
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
