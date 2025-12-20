# Authentication & Access Control

Pangolin supports secure authentication via **JWT Tokens**, **API Keys (Service Users)**, and **OAuth 2.0 / OIDC** (Google, GitHub, Microsoft, Okta).

---

## 🚀 Authentication Modes

Pangolin can be configured in three primary authentication modes via environment variables:

### 1. No-Auth Mode (Development Only)
Disables all authentication checks. All requests are treated as being made by the `root` superadmin.
- **Environment Variable**: `PANGOLIN_NO_AUTH=true`
- **Use Case**: Local development, quick prototyping, automated integration tests.
> [!CAUTION]
> Never use No-Auth mode in production or any environment accessible from the internet.

### 2. JWT Authentication (Standard)
The default mode for secure, stateless session management.
- **Root Credentials**: Initial setup uses `PANGOLIN_ROOT_USER` (default: `admin`) and `PANGOLIN_ROOT_PASSWORD` (default: `password`).
- **Secret Key**: You **must** set `PANGOLIN_JWT_SECRET` to a strong random string (at least 32 characters).
- **Flow**: POST credentials to `/api/v1/users/login` -> Receive JWT -> Include in header: `Authorization: Bearer <jwt>`.

### 3. OAuth 2.0 / OIDC (Enterprise SSO)
Allows users to sign in with external identity providers.
- **Supported Providers**: Google, Microsoft (Entra ID), GitHub, Okta.
- **Configuration**: Requires `FRONTEND_URL` and provider-specific Client IDs/Secrets. See [Provider Setup](#provider-setup) below.

---

## 🔑 Service Users & API Keys

For programmatic access (CI/CD, ETL, automation), use **Service Users**.
- Service users use **X-API-Key** authentication rather than JWTs.
- They are managed via the UI or the `pangolin-admin service-users` CLI.
- See the [Service Users Guide](./features/service_users.md) for details.

---

## 🛠️ Provider Setup (OAuth)

### Google
1. Create a project in [Google Cloud Console](https://console.cloud.google.com/).
2. Setup OAuth 2.0 Client ID for a Web Application.
3. Redirect URI: `https://<api-domain>/oauth/callback/google`.
- **Env**: `OAUTH_GOOGLE_CLIENT_ID`, `OAUTH_GOOGLE_CLIENT_SECRET`, `OAUTH_GOOGLE_REDIRECT_URI`.

### Microsoft (Azure AD)
1. Register an app in the [Azure Portal](https://portal.azure.com/).
2. Add a Web platform and Redirect URI: `https://<api-domain>/oauth/callback/microsoft`.
- **Env**: `OAUTH_MICROSOFT_CLIENT_ID`, `OAUTH_MICROSOFT_CLIENT_SECRET`, `OAUTH_MICROSOFT_TENANT_ID`, `OAUTH_MICROSOFT_REDIRECT_URI`.

### GitHub
1. Create an OAuth App in **Settings > Developer settings**.
2. Callback URL: `https://<api-domain>/oauth/callback/github`.
- **Env**: `OAUTH_GITHUB_CLIENT_ID`, `OAUTH_GITHUB_CLIENT_SECRET`, `OAUTH_GITHUB_REDIRECT_URI`.

---

## 🐍 Connecting with PyIceberg

### Option A: Manual Token (Access Token)
If you have a JWT from a previous login:
```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "token": "YOUR_JWT_TOKEN",
    }
)
```

### Option B: Credential Vending (Recommended)
Pangolin automatically vends S3/Cloud credentials if the Warehouse is configured with a role. You only need to provide the Pangolin token.
- See [Credential Vending](./features/security_vending.md) for warehouse setup.

---

## 🛡️ Troubleshooting

### 401 Unauthorized
- **Cause**: Token missing, expired, or invalid secret.
- **Fix**: Verify `PANGOLIN_JWT_SECRET` matches across all instances. Regenerate token via login.

### 403 Forbidden
- **Cause**: User lacks RBAC permissions for the resource (Catalog/Namespace/Asset).
- **Fix**: Verify roles in the Management UI or via `pangolin-admin permissions`.
