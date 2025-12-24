
# No Auth Mode

Pangolin supports a specialized "No Auth" mode designed for local development, testing, and evaluation where setting up a full authentication provider (like OAuth/OIDC) is not feasible or desired.

## Triggering No Auth Mode

To enable No Auth mode, set the following environment variable before starting the `pangolin_api` server:

```bash
export PANGOLIN_NO_AUTH=true
```

## Behavior & Features

### 1. Automatic Tenant Admin Access
When accessing the API without any credentials, the system automatically treats the request as coming from the default **Tenant Admin** of the default tenant.

- **Username**: `tenant_admin`
- **Role**: `TenantAdmin`
- **Tenant ID**: `00000000-0000-0000-0000-000000000000`

This allows you to perform administrative tasks (create users, manage catalogs, etc.) immediately without logging in.

### 2. "Easy Auth" (Selective Authentication)
While No Auth mode defaults to Admin for unauthenticated requests, it **respects the `Authorization` header** if provided. This is crucial for verifying permissions:

- **If you provide a token**: The system processes the request as that specific user.
- **If you provide NO token**: The system processes the request as the default Tenant Admin.

### 3. Password Bypass
To facilitate logging in as specific users (e.g., `TenantUser` for testing access restrictions) without setting up true hashes:

- The `.login()` endpoint bypasses password verification in this mode.
- You can log in as *any* existing user with *any* password.

## Usage Guide

### Using the CLI
Since the CLI handles authentication tokens automatically, you can mix modes:

1. **Admin Mode (Default)**: Just run commands.
   ```bash
   pangolin-admin catalog list  # Works immediately
   ```

2. **User Mode**: Login as a specific user to switch context.
   ```bash
   pangolin-user login --username data_analyst --password any
   # Now subsequent commands run as data_analyst
   ```

### Using the UI
The UI will automatically detect No Auth mode and may auto-login as `tenant_admin`.

To test different roles:
1. **Logout** via the profile menu.
2. **Login** with the username you created (e.g., `data_analyst`) and any password.
3. You are now exploring the UI restricted by that user's permissions.

### Using the API (cURL / Python)
- **Admin Request**:
  ```bash
  curl http://localhost:8080/api/v1/catalogs
  ```
- **User Request** (after getting token via `/login`):
  ```bash
  curl -H "Authorization: Bearer <token>" http://localhost:8080/api/v1/catalogs
  ```

## Testing with PyIceberg

The server prints a convenient configuration snippet on startup when in No Auth mode. This snippet works immediately because the token vended is for the default `tenant_admin`.

### Configuration
Use this snippet in your Python scripts to connect PyIceberg to your local Pangolin instance:

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "local",
    **{
        "type": "rest",
        "uri": "http://127.0.0.1:8080/api/v1/catalogs/sales/iceberg",
        # Default Admin Token (valid for 24h/until restart)
        "token": "YOUR_TOKEN_FROM_SERVER_LOGS",
        # Critical for No Auth mode routing
        "header.X-Pangolin-Tenant": "00000000-0000-0000-0000-000000000000"
    }
)

# Verify connection
print(catalog.list_namespaces())
```

### Key Notes for PyIceberg
1.  **Tenant Header**: You **MUST** include `"header.X-Pangolin-Tenant"` in the config properties. Without it, the server won't know which tenant context to use for the Iceberg REST endpoints, even with a valid token.
2.  **Token**: The token printed in the logs is a valid JWT for the `tenant_admin` role. You can use it as-is.
3.  **URI**: Point to the specific catalog's Iceberg endpoint (e.g., `.../catalogs/sales/iceberg`). Ensure the catalog exists first (e.g., via the seeding script).
