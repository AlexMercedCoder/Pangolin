# Status

**Updated 2026-08-11.** One reconciled view of what is done and what is not.

Two audit documents exist and are kept as historical records —
[`AUDIT_EXECUTION_PLAN.md`](AUDIT_EXECUTION_PLAN.md) (2026-08-09) and
[`roadmap_aug10.md`](roadmap_aug10.md) (2026-08-10). Both carry status headers
pointing here. **Where they disagree with this file, this file is correct.**

## Verification standard

Everything marked done below is verified by tests that run against live
PostgreSQL, MongoDB and MinIO, not by inspection:

- **63 test targets, 415 tests, zero failures**
- **19 CI jobs green**, including an authorization matrix, a four-backend parity
  suite, both MongoDB topologies, an MSRV check, and a build of every optional
  feature
- `cargo audit` clean; clippy at a ratcheted budget of 30 (from 314)

That standard exists because this project has repeatedly had things that
compiled, passed CI, and did not work. Twice in the 0.8.0 work alone: the
cloud-credential features had never compiled at any version, and a server that
exited 25 seconds after startup passed all 18 CI jobs and the full suite.

## Done

### Security

| Item | Where |
|---|---|
| Authorization bypasses (any principal could mint a `Root` JWT; any tenant member could vend warehouse credentials) | 0.7.0 |
| OAuth token exfiltration, decorative `state` nonce, default JWT secret, auth-bypass path suffix | 0.6.0 |
| Rate limiting on the authentication endpoints, per source address **and** per account | 0.8.0 |
| Warehouse cloud credentials encrypted at rest (AES-256-GCM) | 0.8.0 |
| Dependency advisories: 26 → 0, with eight justified exceptions | 0.7.0 |
| OIDC: PKCE, `id_token` signature validation via JWKS, `iss`/`aud`/`exp`/`nonce` checks, discovery, rate-limited key rotation | 0.8.0 |

### Correctness

| Item | Where |
|---|---|
| Iceberg commit requirements and updates enforced rather than silently dropped | 0.6.0 / 0.7.0 |
| Four-backend parity: tenant scoping, branch scoping, serde formats, CAS, pagination | 0.7.0 |
| The MongoDB UUID encoding audit — one bug wearing eight hats across 21 collections | 0.7.0 |
| Transactional branch-create-by-copy; the API no longer returns `200` on a failed copy | 0.8.0 |
| MongoDB index management, including the uniqueness the SQL backends get from primary keys | 0.8.0 |
| `registerTable`, `listViews`, `viewExists`, `dropView` | 0.8.0 |

### Operations

| Item | Where |
|---|---|
| CI that actually runs — 19 jobs | 0.7.0 / 0.8.0 |
| A release pipeline that produces a release (it never had; `macos-13` was retired and hung every tag for 24h) | 0.7.0 |
| A release gate that verifies the published image over HTTP | 0.7.0 |
| Token-cleanup sweep that **runs** (it was dead code) and staggers across replicas | 0.8.0 |
| Backup/restore drilled and measured: 7s backup, 53s restore, 1345 rows | 0.8.0 |
| Load harness with measured figures, reporting client **and** server-side latency | 0.8.0 |
| Operations docs: backend parity, encryption, backup and recovery, performance, multiple replicas | 0.8.0 |

## Not done

Ordered by how much it would block a production deployment.

### 1. GitHub logins cannot be OIDC-validated

OIDC is implemented and applies to Google, Microsoft, Okta and any IdP given
`PANGOLIN_<PROVIDER>_ISSUER`. **GitHub is not an OIDC provider** — it issues no
`id_token` and publishes no JWKS — so a GitHub login still rests on the userinfo
endpoint and on GitHub's own token scoping.

`PANGOLIN_OIDC_REQUIRE=true` refuses any provider that cannot be
OIDC-validated. It is off by default because turning it on would break a working
GitHub deployment on upgrade with no warning; an operator who wants every login
validated should set it.

Also outstanding on this path: no back-channel logout, no refresh-token
handling, no per-tenant provider configuration, and the PKCE verifier is held in
process — so OAuth needs session affinity across replicas.

### 2. Multi-replica is constrained and unproven

Works with PostgreSQL, one caveat each way:

- OAuth requires session affinity (the nonce and code-exchange stores are
  in-process)
- Rate limiting is per replica, so N replicas give N× the budget
- A rotated warehouse credential can be served by a peer for up to the cache TTL
  (5s)

Documented in [`docs/operations/running-multiple-replicas.md`](docs/operations/running-multiple-replicas.md).
**Not load tested and not soaked.**

### 3. `commitTransaction` is absent, deliberately

The spec promises multi-table atomicity; the store commits per table with
compare-and-swap and has no cross-table transaction. Routing the endpoint and
committing tables one at a time would be worse than leaving it out — an engine
that sees it relies on atomicity that is not there. A test pins the decision.

### 4. Revocation fails open

If the revocation check errors, the request proceeds (A-13). During a database
blip, every revoked token is accepted again. Watch
`pangolin_token_revocation_check_errors_total`.

### 5. Smaller gaps

- No tamper-evident audit trail, no SIEM export
- Symmetric HS256 JWTs with no rotation; rotating invalidates every session
- No MFA, password policy, or account lockout
- No point-in-time recovery — dump and restore only
- Session tokens are stored at rest in plaintext, not hashed
- MongoDB has no versioned schema migrations
- Management API error envelope is still flat `{"error": "..."}`
- `replaceView` and `renameView` not implemented
- The UI's 90 components are all Svelte 4 style in Svelte 5 legacy mode
- Eight accepted dependency advisories to re-check when dependencies move
- clippy 30 and svelte-check 150 backlogs, both ratcheted

## Not shipped

**0.7.0 and 0.8.0 are not published.** The work is merged to the branch and CI
is green, but the merge, tag, Docker push and PyPI upload have not been made.
The most recent published artifact is `alexmerced/pangolin-api:0.5.1` from
2025-12-30 — so **anything running Pangolin today is on 0.5.1**, which predates
every security fix listed above.

The `SECURITY.md` advisory covers `< 0.7.0` for that reason.

The PyPI token has been rotated. Still requiring a person: decide whether to
publish a GHSA once a fixed version actually exists.

## If you are deciding whether to run this

The honest summary: the security holes found in the audits are fixed and there
is now CI that would catch them coming back. For untrusted multi-tenant use, the
remaining gaps are multi-replica being unproven under load, GitHub logins not
being OIDC-validatable, and revocation failing open.

The smallest credible posture today: PostgreSQL, one replica, OAuth disabled,
network-restricted, `PANGOLIN_ENCRYPTION_KEY` set, a backup you have actually
restored once, and the drill script run against a copy of your own data.
