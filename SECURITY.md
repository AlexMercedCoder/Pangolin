# Security Policy

Pangolin is the authoritative index of a data lake and holds cloud storage
credentials. We take reports seriously and would rather hear about a problem
early than read about it later.

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Report privately through GitHub's
[private vulnerability reporting](https://github.com/AlexMercedCoder/pangolin/security/advisories/new)
on this repository. If that is unavailable to you, email the maintainer listed
in `Cargo.toml` with `PANGOLIN SECURITY` in the subject line.

Please include:

* what an attacker can do, and what they need in order to do it
  (network position, an account, a specific role);
* the affected version or commit;
* a reproduction — a request sequence, a script, or a failing test is ideal;
* your assessment of the impact.

### What to expect

| Stage | Target |
|---|---|
| Acknowledgement of your report | 3 working days |
| Initial assessment and severity | 10 working days |
| Fix released for a critical issue | 30 days from confirmation |
| Public advisory | With the fix, or sooner by agreement |

We will keep you informed while we work, credit you in the advisory unless you
prefer otherwise, and tell you before we publish. If we disagree that something
is a vulnerability we will explain why rather than going quiet.

## Supported versions

Pangolin is pre-1.0 and moves fast. Only the latest minor release receives
security fixes.

| Version | Supported |
|---|---|
| 0.7.x | Yes |
| 0.6.x | No — upgrade to 0.7.x, see the advisory below |
| 0.5.x and earlier | No — upgrade to 0.7.x |

## Fixed in 0.7.0

**Every version before 0.7.0 is affected by a privilege-escalation
vulnerability and should not be run.** Upgrade to 0.7.0 and rotate every issued
token. The findings below come from a full-repo audit conducted the day after
the 0.6.0 release; the authorization cluster was outside that release's scope
and is new information, not a regression.

Note which version you are actually running. The published container image
`alexmerced/pangolin-api` was last pushed at **0.5.1** — no 0.6.0 image was
ever released — so most deployments are on 0.5.1, which carries everything
described here *and* everything 0.6.0 fixed. The affected range is
`< 0.7.0`, not `0.6.0` alone.

### Exploitable by any authenticated principal

Every issue in this table needs nothing but a valid credential — including the
lowest-privilege `tenant-user` account, or any service-user API key.

| ID | Issue | Impact |
|---|---|---|
| B0a | `POST /api/v1/tokens` took no session and mapped a body-supplied `roles: ["Root"]` straight into signed claims | **Full privilege escalation.** Any authenticated caller could mint a valid `Root` token for any tenant. `check_permission` short-circuits for `Root`, so the resulting token bypasses every subsequent authorization check in the system |
| B0b | The credential-vending endpoint performed no authorization, never resolved the table, and hardcoded `["read", "write"]` | **Cloud storage credential disclosure.** Any tenant member obtained read+write credentials for the entire warehouse, naming a table they had no rights to and which need not exist |
| B0j | Logout revoked `session.user_id`; revocation is keyed by the token's `jti`, which no token carries as its `user_id` | **Logout did nothing.** The token stayed valid for its full 24-hour lifetime. On a shared machine, "signing out" left a working credential behind |
| B0g | The Iceberg OAuth token endpoint checked `active` but not expiry, unlike the API-key path | **Expired credentials were renewable indefinitely.** An expired service user could exchange `client_credentials` for a fresh JWT, repeatedly |
| B0h | The `NO_AUTH` public-bind guard read `no_auth && !dev_mode && !is_loopback(..)` | **Unauthenticated tenant-admin access.** `PANGOLIN_NO_AUTH=true` with `PANGOLIN_DEV_MODE=true` — routinely set together in compose and development setups — started happily on `0.0.0.0` and treated every anonymous request as `TenantAdmin` |
| B0l | OAuth matched existing users on email with no `email_verified` check and no provider binding | **Account takeover.** Anyone able to set a matching address on any configured provider — GitHub permits unverified addresses — logged in as that Pangolin user, including the seeded tenant admin |
| B0i | `PermissionScope::Tenant` matched without comparing the grant's tenant to the resource's | A tenant-wide grant issued in one tenant satisfied authorization for another tenant's resources |
| B0c–B0f | `rename_table`, `update_namespace_properties`, view create/read and `perform_maintenance` performed no authorization check at all | Any tenant member could move any table (an effective delete), rewrite namespace properties including `location`, read any view's SQL, and trigger snapshot expiry and orphan-file deletion. `perform_maintenance` additionally ran against a hardcoded `"default"` catalog rather than the one addressed |
| B0m | `expires_in_hours` reached `chrono::Duration::hours` unclamped, and no catch-panic layer was installed | **Remote panic.** A large value aborted the connection task, taking other in-flight requests on that connection with it |
| B0o | A token whose `jti` was absent or unparseable skipped the revocation check entirely | Such tokens were unrevocable for their full lifetime |

### Availability

| ID | Issue | Impact |
|---|---|---|
| — | `SqliteStore` had no inherent `revoke_token`/`is_token_revoked`, so the trait implementations called themselves | **Remote crash on the SQLite backend.** Revoking a token — which logout does — recursed until the thread stack was exhausted and aborted the process |
| B2 | On MongoDB the revocation write and the revocation check used different field names *and* different types | Revocation could never match. Revoked tokens, including after logout, stayed valid |

### Confidentiality and integrity of records

| ID | Issue | Impact |
|---|---|---|
| B1 | MongoDB's `get_audit_event` discarded the `tenant_id` parameter | **Cross-tenant audit disclosure.** Any tenant holding an audit-event UUID could read another tenant's record: username, IP address, resource names and metadata |
| — | The SQLite `audit_logs` table declared different columns than the code inserted | **No audit trail at all on SQLite.** Every audit write failed at runtime. If you run SQLite, assume you have no audit history prior to 0.7.0 |
| — | The admin CLI and Python SDK wrote their stored bearer tokens at the process umask, typically `0644` | Any local account could read the token. Now `0600` under a `0700` directory |
| — | `generate-code` and `get-token` printed live JWTs into copy-paste output | Tokens landed in shell scrollback, session transcripts and pasted snippets |
| B44 | A live PyPI API publish token was committed to the repository working tree in `.env` | Never tracked by Git, but readable by any local tool and passed into containers by `docker compose`. **The token has been removed and must be treated as compromised** |

### Data loss

Not security boundaries, but silent corruption is worth the same attention:

* **B5** MongoDB's `update_metadata_location` ignored the compare-and-swap
  entirely. Two concurrent Iceberg commits both reported success and one
  snapshot was lost. Memory, Postgres and SQLite all enforced it.
* **B3** SQLite's `delete_branch` referenced a column that does not exist and
  was not transactional: the branch was committed away and its assets orphaned
  permanently, while the caller received an error suggesting nothing happened.
* **B7** All three persistent backends stored the `Debug` spelling of
  `AssetType` and parsed only two of seventeen variants. A `DeltaTable`,
  `MlModel` or `Lance` asset round-tripped as an Iceberg table.
* **B4** Postgres decoded a `TEXT[]` column as `String`; `sqlx::Row::get`
  panics on a decode failure, so any search returning at least one hit panicked
  the request.

### Credential vending could not be built

The `aws-sts`, `azure-oauth` and `gcp-oauth` features - and the
`cloud-credentials` bundle that unions them - did not compile, at any version.
`cargo build` and `cargo test` run with default features and no job ever passed
`--features`, so the entire cloud-credential surface had rotted into code that
could not be built at all: parameters bound as `_name` and referenced as `name`
inside the `cfg` block, a missing macro import, and an STS expiry parsed from a
`DateTime` as though it were an RFC3339 string.

This is not an exploitable defect - unbuildable code ships in no binary. It
matters because it means **STS-based credential vending was not running
anywhere**, so any deployment that believed it was handing out scoped,
time-limited credentials was not. Check what your warehouses are actually
configured with: if `use_sts` is set but the server was built without the
feature (which is to say, always), the static-credential fallback in
`S3Signer::generate_credentials` is what answered - vending your long-lived
warehouse keys, with no expiry, instead of a scoped session token. Where no
static keys were configured either, the call failed with "AWS credentials not
configured", which at least failed closed.

Fixed in 0.7.0, with a CI job that builds every optional feature so it cannot
recur silently.

### Upgrading to 0.7.0

1. **Rotate every issued token.** B0a means any account may have minted a
   `Root` token, and B0j means logging out never invalidated anything. Rotating
   `PANGOLIN_JWT_SECRET` invalidates all existing sessions at once and is the
   fastest way to be sure.
2. **Rotate service-user API keys**, for the same reason: B0g allowed expired
   keys to keep issuing fresh JWTs.
3. **Rotate any cloud storage credentials** reachable through a warehouse that
   an untrusted tenant member could name. B0b vended them to anyone.
4. **Audit for the escalation.** `POST /api/v1/tokens` calls from non-admin
   principals, and credential-vending calls for tables the caller had no grant
   on, are the two signals. Note that on SQLite there is no audit history to
   check, and on MongoDB action-filtered queries returned nothing (B23) — so a
   clean audit log is not evidence of absence on those backends.
5. **Check `PANGOLIN_NO_AUTH` and `PANGOLIN_DEV_MODE`.** If both were set on a
   non-loopback bind, treat the deployment as having been open to anonymous
   tenant-admin access for that period. The server now refuses to start in that
   configuration.
6. **Set `PANGOLIN_OAUTH_EMAIL_LINK_DOMAINS`** if you rely on OAuth accounts
   being matched to existing local users by email. Without it, identity is
   `(provider, subject)` only and an address never adopts an existing account —
   which is the safe default, and a behaviour change.
7. **On SQLite, upgrading migrates the `audit_logs` table** to schema version 2.
   The previous table is preserved as `audit_logs_pre_v2`; it is empty in
   practice, because nothing could ever write to it.
8. **Third-party API clients may need changes.** Server request bodies now
   reject unknown fields with `422` instead of silently ignoring them. If a
   client sent a misspelled or obsolete field, it was already being discarded —
   the request was never doing what it appeared to.

## Fixed in 0.6.0

0.6.0 is a security release. If you run anything earlier, upgrade and rotate
credentials. The following were exploitable:

| ID | Issue | Impact |
|---|---|---|
| A-8 | The OAuth callback appended the session JWT to a redirect URL taken from the unsigned `state` parameter, with no allowlist | **Account takeover.** An authorize link whose `state` decoded to `{"redirect_uri":"https://evil.example/"}` delivered a valid token for the victim to the attacker's access log, with no credential theft required |
| A-9 | The OAuth `state` nonce was generated but never stored or verified; `state` was not validated at all | **Login CSRF.** An attacker could bind a victim's browser to the attacker's account |
| A-10 | `PANGOLIN_JWT_SECRET` fell back to `default_secret_for_dev`, a value published in this repository; the Helm chart shipped `change-me-please` and `password` as *working* defaults, and the seeded admin used `password123` | **Anyone could forge a `Root` token** against a deployment that missed one environment variable |
| A-11 | The authentication whitelist matched any path *ending in* `/config` and any path *containing* `/oauth/tokens` | **Authentication bypass.** A namespace or table named `config` was reachable unauthenticated, including its DELETE route |
| A-12 | API-key authentication ran `bcrypt::verify` against every service user in every tenant | **Unauthenticated denial of service.** At 100 service users a single request with a bogus key burned ~25 CPU-seconds, before any rate limiting |
| A-13 | A store error during the token-revocation check was logged and ignored | **Revoked tokens were accepted again** during any database disruption |
| A-14 | The root password was compared with `==` | Timing oracle on the password |

Correctness defects fixed in the same release — silent data loss rather than a
security boundary, but worth knowing about if you ran concurrent writers:

* **A-1** `assert-ref-snapshot-id` was never enforced, so a writer whose
  compare-and-swap lost would retry and re-apply its snapshot onto a branch that
  had moved on: forked snapshot lineage and orphaned data files, with no error
  ever surfaced.
* **A-2** Eleven commit update types were discarded while returning `200 OK`.
* **A-3** `last_sequence_number` was assigned a snapshot ID, which can produce
  incorrect query results on merge-on-read tables.

### Upgrading

1. **Set `PANGOLIN_JWT_SECRET`** to a real secret (`openssl rand -base64 48`).
   The server now refuses to start without one, and rejects known placeholders.
   Changing it invalidates every existing session — which you want, because any
   token issued under the old default was forgeable.
2. **Rotate service-user API keys.** Keys are now issued in a
   `pgl_<key-id>_<secret>` format that makes lookup O(1). Pre-existing keys keep
   working only if you set `PANGOLIN_ALLOW_LEGACY_API_KEYS=true`, which
   reintroduces the scan and should be temporary.
3. **Set `PANGOLIN_OAUTH_REDIRECT_URIS`** if you use OAuth. Redirect targets are
   now allowlisted, and the token is delivered by a one-time code exchange
   rather than a URL parameter — see `docs/operations/oidc.md` for the client
   change.
4. **Review your audit log** for `tenant_impersonation`, `login_failed` and
   `api_key_rejected` events, which are recorded from 0.6.0 onward.

## Hardening checklist

Before exposing Pangolin to anything you care about:

- [ ] `PANGOLIN_JWT_SECRET` is at least 32 bytes of real entropy and is not in
      Git. Prefer `existingSecret` in the Helm chart over `values.yaml`.
- [ ] `PANGOLIN_NO_AUTH` is unset. It grants tenant-admin to anonymous callers;
      the server refuses to start with it enabled on a non-loopback address, but
      do not rely on that.
- [ ] `PANGOLIN_DEV_MODE` is unset.
- [ ] `PANGOLIN_ROOT_USER` is unset unless you actively need root basic auth.
- [ ] TLS is terminated in front of the server. Pangolin binds plain HTTP.
- [ ] `PANGOLIN_CORS_ALLOWED_ORIGINS` is set. The default allows any origin,
      which is only safe behind a trusted gateway.
- [ ] `PANGOLIN_ALLOW_LEGACY_API_KEYS` is unset once keys are rotated.
- [ ] Containers run as non-root with a read-only root filesystem — the shipped
      chart defaults do this; confirm you have not overridden them.
- [ ] Database backups are encrypted. Warehouse rows contain cloud storage
      credentials in the clear; envelope encryption at rest is not yet
      implemented (C-11).
- [ ] `/metrics` is not exposed publicly. It is unauthenticated by design for
      scraping; set `PANGOLIN_METRICS_ENABLED=false` if you cannot restrict it.

## Known gaps

The full reconciled list, including non-security work, is in
[STATUS.md](STATUS.md).


Stated plainly, because a checklist that hides its limits is worse than none:

* **Rate limiting is in-process, so it is per replica.** The authentication
  endpoints are throttled from 0.7.0, per source address and per account, but
  the counters are not shared between replicas: with N replicas an attacker
  gets N times the configured budget. A shared limiter needs Redis or
  equivalent, which this project does not otherwise require.
* **OAuth is not full OIDC.** No PKCE, no `id_token` signature validation, no
  JWKS, no discovery document. Users are matched on the email a provider
  returns, with no `email_verified` check.
* **Symmetric HS256 JWTs with no key rotation.** Rotating the secret invalidates
  every session at once.
* **Warehouse credentials are encrypted at rest only when a key is set.**
  `PANGOLIN_ENCRYPTION_KEY` enables AES-256-GCM sealing of the credential
  fields; unset, they are plaintext and the server warns at startup. The key
  lives in the server environment, so this protects a stolen database and not a
  compromised host.
* **Audit records are not tamper-evident.** They live in the same database as
  application data, with no hash chaining and no WORM option.
* **No MFA, password policy, or account lockout.**
* **Eight dependency advisories are accepted rather than fixed**, each with its
  reasoning recorded in `.cargo/audit.toml`. None is reachable in a default
  build, but they are exceptions rather than absences: `rsa` (never compiled),
  `quick-xml` (held back by `object_store` 0.11 and `azure_core` 0.20),
  a second `rustls-webpki` arriving via the AWS SDK's `rustls` 0.21,
  `http-types`, and `rand`'s custom-logger unsoundness. Re-check the list when
  dependencies move.

These are tracked in `AUDIT_EXECUTION_PLAN.md` (items C-2 through C-20).
