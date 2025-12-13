# Authentication & Access Control

Pangolin supports secure authentication via **Basic Auth** (username/password) and **OAuth 2.0 / OIDC** (Google, GitHub, Microsoft).

## Supported Methods

### 1. Basic Authentication (Root Only)
- **Supported for**: The **Root User** only.
- **Method**: Standard HTTP Basic Auth header (`Authorization: Basic <base64(user:pass)>`).
- **Use Case**: Initial server setup, creating the first tenant/admin.

### 2. Token Authentication (Standard Users & Service Accounts)
- **Supported for**: All Tenant Users (including "Service Accounts").
- **Method**: Bearer Token header (`Authorization: Bearer <jwt_token>`).
- **Use Case**: Day-to-day operations, PyIceberg connections, Data Engineering workflows.

> **Note on Service Accounts**: Pangolin does not currently have a separate "Service Account" entity. To create a service account for a script/pipeline, simply create a standard Tenant User and manage their credentials securely.

## Token Generation Flows

### A. Native Authentication (Username/Password)
If you created a user via the UI or API with a password:

1.  **Request**: POST to `/api/v1/users/login`
    ```json
    {
      "username": "my-service-account",
      "password": "my-secure-password",
      "tenant_id": "optional-tenant-uuid"
    }
    ```
2.  **Response**: Returns a JSON object containing the `token` (JWT).
3.  **Usage**: Use this JWT in the `Authorization: Bearer <token>` header for subsequent requests.

### B. OAuth 2.0 / OIDC (SSO)
Pangolin integrates with providers like Google, GitHub, etc.

**Configuration:**
Set the following environment variables for your chosen provider (e.g., Google):
- `OAUTH_GOOGLE_CLIENT_ID`
- `OAUTH_GOOGLE_CLIENT_SECRET`
- `OAUTH_GOOGLE_REDIRECT_URI`: e.g., `https://your-pangolin.com/oauth/callback/google`
- `OAUTH_GOOGLE_AUTH_URL` (Optional override)
- `OAUTH_GOOGLE_TOKEN_URL` (Optional override)

**Flow:**
1. Client redirects user to `/oauth/authorize/{provider}?redirect_uri={client_callback}`.
2. User authenticates with Provider.
3. Provider redirects to Pangolin Callback.
4. Pangolin creates a session and redirects to `client_callback` with `?token={jwt}`.

## Connecting with PyIceberg

PyIceberg connects to Pangolin using the REST catalog interface. You can authenticate using a token (OAuth) or credentials (Basic).

### Option A: Using OAuth Token (Recommended)

If you have authenticated via the UI or OAuth flow, use the **Access Token** directly.

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/iceberg/default", # 'default' is the catalog name
        "type": "rest",
        "token": "YOUR_PANGOLIN_JWT_TOKEN",            # Obtained from login
        # If accessing storage directly (no vending):
        # "s3.access-key-id": "...",
        # "s3.secret-access-key": "...",
    }
)
```

### Option B: Credential Vending (Secure S3 Access)

Pangolin supports **Credential Vending**. If configured, Pangolin will provide temporary S3 credentials to PyIceberg automatically.

**Prerequisites:**
- The requested Table/Namespace must be in a specific Warehouse.
- The Warehouse must have an associated Role with S3 write permissions.
- **Client**: You only need to provide the `token`.

### Option C: Basic Auth (Root User Only)

**Only the Root user** can use Basic Auth directly in the client.

```python
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080/iceberg/default",
        "type": "rest",
        "credential": "root:root-password", # ONLY for Root
        # "scope": "test_tenant",           # Ignored for root (root has all access)
    }
)
```

## Troubleshooting
- **401 Unauthorized**: Check if your token has expired or if the `credential` format is correct.
- **403 Forbidden**: Your user does not have permission for the requested resource (check RBAC/Tags).
