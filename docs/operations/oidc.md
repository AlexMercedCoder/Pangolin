# OpenID Connect

Before 0.8.0 this page described a gap. Pangolin's "OAuth" login was
*authorization*, not authentication: it exchanged a code for an access token,
called the provider's userinfo endpoint, and believed the response. That is only
sufficient if the access token could not have come from anywhere else — which is
precisely what OIDC exists to establish.

From 0.8.0 the flow does real OIDC where the provider supports it.

## What is verified now

| Check | What it prevents |
|---|---|
| **PKCE (S256)** | An attacker who intercepts the authorization code — from a referrer header, a proxy log, shell history on a shared machine — cannot redeem it without the verifier |
| **`id_token` signature** against the provider's JWKS | Identity comes from something the provider signed, not from an HTTP response any holder of some access token could have elicited |
| **`aud`** must contain our `client_id` | A token minted for a *different* application at the same provider logging its holder in here — the classic confused deputy |
| **`iss`** must match the discovery document | A token from any other issuer being accepted |
| **`exp`** with 60s leeway | Replay of an expired assertion; the leeway is for clock skew, which otherwise produces logins that fail for one second and nobody can diagnose |
| **`nonce`** must match this login | An `id_token` observed in one flow being replayed into another |
| **Asymmetric `alg` only** | An attacker setting `alg: HS256` and signing with the provider's *public* key, which for HMAC is also the verification key |

Retained from 0.6.0/0.7.0: signed single-use `state` (CSRF), an allowlisted
redirect resolved to an index so the URL never travels inside `state`, the
session token never placed in a redirect URL, and `(provider, subject)` as the
identity — email only adopts a pre-existing account when the provider says it is
verified *and* its domain is operator-allowlisted.

## Configuration

Nothing new is required. If a provider has a known issuer, OIDC applies
automatically.

| Variable | Purpose |
|---|---|
| `PANGOLIN_OIDC_REQUIRE` | `true` refuses any provider that is not OIDC-capable. Off by default |
| `PANGOLIN_<PROVIDER>_ISSUER` | Override the issuer URL — needed for self-hosted Keycloak, Auth0, a private Okta, or any internal IdP |

Known issuers, derived automatically:

- **Google** — `https://accounts.google.com`
- **Microsoft** — from `PANGOLIN_MICROSOFT_TENANT_ID`
- **Okta** — from `PANGOLIN_OKTA_DOMAIN`

## GitHub is not an OIDC provider

GitHub's OAuth issues no `id_token` and publishes no JWKS. There is no honest
way to validate a GitHub login the way the table above describes.

So a GitHub login still uses the userinfo endpoint, and the code says so rather
than reporting validation it did not perform. `PANGOLIN_OIDC_REQUIRE=true`
refuses GitHub outright — which is the right behaviour for an operator who has
decided every login must be OIDC-validated, and the reason the setting exists.

GitHub also does not report `email_verified`, so its addresses are treated as
unverified and can never adopt a pre-existing account.

## Turning on strict mode

```bash
PANGOLIN_OIDC_REQUIRE=true
```

With this set:

- a provider with no issuer is refused at authorize **and** at callback;
- a discovery failure fails the login rather than silently proceeding without
  PKCE.

It is off by default because turning it on would break a working GitHub
deployment on upgrade with no warning. That is a deliberate choice about
upgrades, not a judgement that GitHub logins are fine.

## Key rotation

JWKS documents are cached for an hour. When a token arrives with a `kid` that is
not in the cache — which is what key rotation looks like — the JWKS is refetched
once, rate-limited to one forced refetch a minute per provider.

Both halves matter. Without the refetch, a provider rotating keys breaks every
login until the hour expires. Without the rate limit, a stream of tokens
carrying junk `kid`s becomes a denial-of-service against the provider's JWKS
endpoint and against our own latency, since every such request would block on an
outbound fetch.

## Multi-replica

The PKCE verifier and OIDC nonce are held **in process**, keyed by the state
nonce. If the callback lands on a different replica than the one that started
the flow, the login fails.

The verifier is deliberately *not* carried in `state`: `state` travels through
the browser in the same URL as the authorization code, so anyone positioned to
steal the code would also hold the verifier, and PKCE would protect nothing in
the one situation it exists for.

So OAuth still requires session affinity. See
[running-multiple-replicas.md](running-multiple-replicas.md). Moving this to the
database would remove the constraint and is not done.

## What is still not done

- **No `email_verified` enforcement at the point of account creation.** A
  verified email is required to *adopt an existing* account; a new account is
  created from the provider's subject regardless.
- **No back-channel logout** (RP-initiated or front-channel).
- **No refresh-token handling.** Sessions are Pangolin JWTs with their own
  lifetime; when one expires the user logs in again.
- **No dynamic client registration**, and no per-tenant provider configuration —
  providers are process-wide environment variables.
- **The nonce and verifier stores are in-process**, as above.

## Testing

`pangolin_api/tests/oidc_validation_tests.rs` stands up a provider with
`wiremock`, serving a real discovery document and JWKS, and signs tokens with a
real 2048-bit RSA key. Nothing is mocked at the crypto layer, because the
properties under test *are* the crypto — a test that stubbed signature checking
would pass against code that skipped it.

Each case names the attack it prevents, and the suite was checked for being
load-bearing: disabling audience validation makes the confused-deputy test fail,
and weakening the nonce comparison makes the replay test fail.
