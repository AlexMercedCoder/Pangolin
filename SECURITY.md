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
| 0.6.x | Yes |
| 0.5.x and earlier | No — upgrade to 0.6.x |

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

Stated plainly, because a checklist that hides its limits is worse than none:

* **No rate limiting on authentication endpoints.** The login endpoint is
  brute-forceable. There are global concurrency and body limits, and a request
  timeout, but no per-IP or per-account throttle.
* **OAuth is not full OIDC.** No PKCE, no `id_token` signature validation, no
  JWKS, no discovery document. Users are matched on the email a provider
  returns, with no `email_verified` check.
* **Symmetric HS256 JWTs with no key rotation.** Rotating the secret invalidates
  every session at once.
* **Warehouse credentials are stored unencrypted** in the catalog database.
* **Audit records are not tamper-evident.** They live in the same database as
  application data, with no hash chaining and no WORM option.
* **No MFA, password policy, or account lockout.**

These are tracked in `AUDIT_EXECUTION_PLAN.md` (items C-2 through C-20).
