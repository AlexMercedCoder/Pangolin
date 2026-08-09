# OAuth / SSO configuration

**Read this before upgrading to 0.6.0 if you use OAuth.** The token delivery
mechanism changed, and existing clients will break.

## What changed, and why

Before 0.6.0 the callback base64-decoded the `state` parameter, read
`redirect_uri` out of it, and appended the freshly minted session JWT to that
URL as a query parameter:

```rust
let base_url = frontend_url.unwrap_or_else(|| env::var("FRONTEND_URL")…);
let redirect_url = format!("{}?token={}", base_url, token);
Redirect::to(&redirect_url)
```

`state` was plain base64 JSON — unsigned, unencrypted, never stored server-side
— and there was no allowlist on the destination. Sending someone an authorize
link whose `state` decoded to `{"redirect_uri":"https://evil.example/"}` caused
Pangolin to 302 their browser to the attacker's host with a valid token for
their real identity in the URL. Full account takeover, no credential theft
(A-8). Separately, the nonce embedded in `state` was generated and never
verified, which is login CSRF (A-9).

From 0.6.0:

* `state` is HMAC-SHA256 signed with the server's secret and carries an expiry.
* The nonce inside it is registered server-side and consumed exactly once, so a
  captured callback cannot be replayed.
* `state` is bound to the provider it was issued for.
* `redirect_uri` is validated against an operator-configured allowlist by exact
  match, and only an *index* into that allowlist travels inside `state`.
* **The token is never in a URL.** The callback parks it and redirects with a
  short-lived, single-use `code`, which the client exchanges over POST.

## Configuration

```bash
FRONTEND_URL=https://app.example.com/oauth/callback

# Every additional URL an OAuth flow may return to, comma-separated.
# FRONTEND_URL is always allowed and does not need repeating.
PANGOLIN_OAUTH_REDIRECT_URIS=https://app.example.com/oauth/callback,https://admin.example.com/oauth/callback

PANGOLIN_GOOGLE_CLIENT_ID=…
PANGOLIN_GOOGLE_CLIENT_SECRET=…
PANGOLIN_GOOGLE_REDIRECT_URI=https://catalog.example.com/oauth/callback/google
```

Supported providers: `google`, `microsoft` (also needs
`PANGOLIN_MICROSOFT_TENANT_ID`), `github`, `okta` (also needs
`PANGOLIN_OKTA_DOMAIN`).

A `redirect_uri` that is not in the allowlist gets `400` from
`/oauth/authorize/{provider}` with a message naming the variable to add. Matching
is exact — `https://app.example.com` does not match
`https://app.example.com/`.

## The flow

```
 1. Browser  → GET  /oauth/authorize/google?redirect_uri=<allowlisted>
 2. Pangolin → 302 to the provider, carrying signed single-use state
 3. Provider → 302 to /oauth/callback/google?code=…&state=…
 4. Pangolin    verifies the signature, expiry, provider binding and nonce
                exchanges the code, fetches user info, mints a session
 5. Pangolin → 302 to <allowlisted redirect>?code=<one-time exchange code>
 6. Client   → POST /api/v1/oauth/exchange  {"code": "<code>"}
 7. Pangolin → 200 {"token": "<session JWT>", "token_type": "Bearer"}
```

## Client migration

The redirect now carries `code`, not `token`.

```js
// Before
const token = new URLSearchParams(location.search).get('token');

// After
const code = new URLSearchParams(location.search).get('code');
const res = await fetch('/api/v1/oauth/exchange', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ code }),
});
const { token } = await res.json();
```

The code is single-use and expires in two minutes. Redeem it immediately, and
strip it from the URL afterwards (`history.replaceState`).

## Limitations

Pangolin implements the OAuth 2.0 authorization-code grant followed by a call to
the provider's userinfo endpoint. It is **not** a full OIDC relying party:

* No PKCE.
* No `id_token` validation: no JWKS fetch, no signature check, no `iss`/`aud`
  verification, no `nonce` binding to the ID token.
* No discovery document, so onboarding an arbitrary enterprise IdP needs a code
  change — four providers are hardcoded.
* No SAML, SCIM provisioning, or group-to-role mapping.
* Users are found or created by the **email** the provider returns, with no
  `email_verified` check and no domain allowlist. With a provider that permits
  unverified emails this is an account-takeover path. Looking users up by
  `(provider, subject)` is the correct fix and is not done yet.

These are tracked as C-2 and C-3 in `AUDIT_EXECUTION_PLAN.md` (Phase 3.1).

**Until then:** only enable providers that verify email addresses, and restrict
sign-in at the identity provider rather than relying on Pangolin to do it.

## Multi-replica note

The OAuth nonce and exchange-code stores are in-process. With more than one
replica and no session affinity, a callback that lands on a different pod than
the authorize request will be rejected as an unknown nonce. Enable sticky
sessions on your ingress for `/oauth/*`, or run a single replica for the OAuth
path, until these move to the shared store.
