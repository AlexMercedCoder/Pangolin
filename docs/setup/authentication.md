# Pangolin Authentication Configuration Guide

This guide details how to configure authentication for your Pangolin deployment. Pangolin supports three modes of operation:
1. **No Auth** (Development/Testing)
2. **JWT Authentication** (Standard Production)
3. **OAuth 2.0 / OIDC** (Enterprise Integration)

---

## 1. No Auth Mode (Development Only)
For local development or testing where security is not a concern, you can bypass all authentication checks.
In this mode, all requests are treated as being made by a root superadmin.

**Environment Variable:**
```bash
PANGOLIN_NO_AUTH=true
```

> [!CAUTION]
> Never use this mode in a production environment or any deployment accessible from the internet.

---

## 2. JWT Authentication (Standard)

By default, Pangolin uses JSON Web Tokens (JWT) for secure, stateless session management.
You must configure a strong secret key for signing these tokens.

### Configuration
| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `PANGOLIN_JWT_SECRET` | A secure, random string (at least 32 chars) used to sign tokens. | **YES** | `default_secret_for_dev` (UNSAFE) |
| `PANGOLIN_ROOT_USER` | Username for the initial root admin account. | No | `admin` |
| `PANGOLIN_ROOT_PASSWORD`| Password for the initial root admin account. | No | `password` |

### Generating a Secret
You can generate a secure secret using `openssl`:
```bash
openssl rand -hex 32
```

### Initial Setup
1. Set the environment variables.
2. Start the server.
3. Use the `/api/v1/users/login` endpoint with your root credentials to obtain a Bearer token.
4. Include this token in the `Authorization` header for subsequent requests:
   `Authorization: Bearer <your_token>`

---

## 3. OAuth 2.0 / OIDC Integration

Pangolin supports OAuth 2.0 login with major identity providers. This allows users to sign in with their existing corporate or social accounts.

### Global OAuth Configuration
| Variable | Description | Required |
|----------|-------------|----------|
| `FRONTEND_URL` | The URL of your frontend application. Used for redirects after login. | Yes (e.g., `https://app.pangolin.io`) |

---

### Google Configuration
1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Create a new Project or select an existing one.
3. Navigate to **APIs & Services > Credentials**.
4. create **OAuth 2.0 Client ID**.
5. Set **Application Type** to `Web application`.
6. Add your callback URL to **Authorized redirect URIs**:
   `https://<your-api-domain>/oauth/callback/google`

**Environment Variables:**
```bash
OAUTH_GOOGLE_CLIENT_ID=<your-client-id>
OAUTH_GOOGLE_CLIENT_SECRET=<your-client-secret>
OAUTH_GOOGLE_REDIRECT_URI=https://<your-api-domain>/oauth/callback/google
```

---

### Microsoft (Azure AD) Configuration
1. Go to the [Azure Portal](https://portal.azure.com/).
2. Navigate to **Microsoft Entra ID (formerly Azure AD) > App registrations**.
3. Register a new application.
4. Under **Authentication**, add a **Web** platform.
5. Add your callback URL:
   `https://<your-api-domain>/oauth/callback/microsoft`
6. Generate a client secret under **Certificates & secrets**.

**Environment Variables:**
```bash
OAUTH_MICROSOFT_CLIENT_ID=<your-client-id>
OAUTH_MICROSOFT_CLIENT_SECRET=<your-client-secret>
OAUTH_MICROSOFT_TENANT_ID=<your-tenant-id> (or 'common' for multi-tenant)
OAUTH_MICROSOFT_REDIRECT_URI=https://<your-api-domain>/oauth/callback/microsoft
```

---

### GitHub Configuration
1. Go to **Settings > Developer settings > OAuth Apps**.
2. Register a new application.
3. Set **Authorization callback URL** to:
   `https://<your-api-domain>/oauth/callback/github`

**Environment Variables:**
```bash
OAUTH_GITHUB_CLIENT_ID=<your-client-id>
OAUTH_GITHUB_CLIENT_SECRET=<your-client-secret>
OAUTH_GITHUB_REDIRECT_URI=https://<your-api-domain>/oauth/callback/github
```

---

### Okta Configuration
1. Log in to your Okta Admin Console.
2. Go to **Applications > Create App Integration**.
3. Select **OIDC - OpenID Connect** and **Web Application**.
4. Set **Sign-in redirect URIs** to:
   `https://<your-api-domain>/oauth/callback/okta`

**Environment Variables:**
```bash
OAUTH_OKTA_DOMAIN=<your-org>.okta.com
OAUTH_OKTA_CLIENT_ID=<your-client-id>
OAUTH_OKTA_CLIENT_SECRET=<your-client-secret>
OAUTH_OKTA_REDIRECT_URI=https://<your-api-domain>/oauth/callback/okta
```

---

## 4. Troubleshooting

### "Invalid Token" Error
- Ensure `PANGOLIN_JWT_SECRET` is exactly the same on all instances of the API server.
- Ensure the token was not truncated when copied.
- Check server time synchronization (NTP).

### "Redirect URI Mismatch" Error
- Verify that the `OAUTH_*_REDIRECT_URI` environment variable exactly matches the URL registered with the provider.
- Ensure you are using the correct protocol (`http` vs `https`).
