# Authentication & Authorization Architecture

## Overview

Pangolin implements a multi-layered authentication and authorization system supporting multiple authentication methods and role-based access control (RBAC). This document provides a comprehensive guide to how authentication works across all components.

## Authentication Methods

### 1. Basic Authentication (Root Users Only)

**Location**: [auth_middleware.rs:L174-210](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/auth_middleware.rs#L174-210)

Basic Auth is **ONLY** supported for Root-level administrative access via environment variables.

#### Configuration
```bash
export PANGOLIN_ROOT_USER=admin
export PANGOLIN_ROOT_PASSWORD=your_secure_password
```

#### Usage
```bash
curl -u admin:your_secure_password http://localhost:8080/api/v1/tenants
```

#### How It Works
1. Client sends `Authorization: Basic <base64(username:password)>` header
2. Middleware decodes the credentials
3. Compares against `PANGOLIN_ROOT_USER` and `PANGOLIN_ROOT_PASSWORD` environment variables
4. If match: Creates a Root session with `UserRole::Root`
5. If no match: Falls through to Bearer token authentication

\u003e [!IMPORTANT]
\u003e **Basic Auth is NOT supported for tenant users**. Tenant admins and users MUST use Bearer tokens obtained via the `/api/v1/users/login` endpoint.

### 2. Bearer Token Authentication (All Users)

**Location**: [auth_middleware.rs:L213-232](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/auth_middleware.rs#L213-232)

Bearer tokens are JWT tokens used for all tenant-level authentication.

#### Obtaining a Token

**Login Endpoint**: `POST /api/v1/users/login`

```bash
curl -X POST http://localhost:8080/api/v1/users/login \\
  -H "Content-Type: application/json" \\
  -d '{
    "username": "tenant_admin",
    "password": "password123"
  }'
```

**Response**:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "...",
    "username": "tenant_admin",
    "role": "TenantAdmin",
    "tenant_id": "..."
  }
}
```

#### Tenant-Scoped Login

For multi-tenant deployments, users can have the same username across different tenants. To resolve username collisions, include the `tenant-id` field in login requests:

**Root Login** (no tenant context):
```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password",
    "tenant-id": null
  }'
```

**Tenant-Scoped Login** (with tenant context):
```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user",
    "password": "pass123",
    "tenant-id": "<tenant-uuid>"
  }'
```

> [!IMPORTANT]
> The field name is `tenant-id` (kebab-case), not `tenant_id` (underscore). This is due to the `LoginRequest` struct using `#[serde(rename_all = "kebab-case")]`.

#### Usage
```bash
curl -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..." \\
  http://localhost:8080/api/v1/catalogs
```

#### Token Structure

Tokens are signed JWTs containing:
- `sub`: User ID
- `jti`: Token ID (for revocation)
- `username`: Username
- `tenant_id`: Tenant ID (null for Root users)
- `role`: User role (`Root`, `TenantAdmin`, `TenantUser`)
- `exp`: Expiration timestamp
- `iat`: Issued at timestamp

#### Token Verification Flow

1. Extract token from `Authorization: Bearer <token>` header
2. Verify JWT signature using `PANGOLIN_JWT_SECRET` (default: `default_secret_for_dev`)
3. Check if token is revoked (via `is_token_revoked` store method)
4. Extract claims and create session
5. Insert session into request extensions

### 3. API Key Authentication

**Location**: [auth_middleware.rs:L135-154](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/auth_middleware.rs#L135-154)

API keys provide long-lived authentication for programmatic access.

#### Header Format
```
X-API-Key: <api_key_value>
```

#### How It Works
1. Extract API key from `X-API-Key` header
2. Look up key in store via `get_api_key_by_value`
3. Verify key is not expired
4. Create session from key's user context
5. Insert session into request extensions

\u003e [!NOTE]
\u003e API keys are managed through the `/api/v1/users/{user_id}/tokens` endpoints.

## Authentication Flow

```mermaid
graph TD
    A[Incoming Request] --\u003e B{Public Endpoint?}
    B -->|Yes| C[Allow Request]
    B -->|No| D{X-API-Key Header?}
    D -->|Yes| E[Verify API Key]
    E -->|Valid| F[Create Session from Key]
    E -->|Invalid| G[401 Unauthorized]
    D -->|No| H{Authorization Header?}
    H -->|No| G
    H -->|Yes| I{Basic or Bearer?}
    I -->|Basic| J{Root Credentials?}
    J -->|Yes| K[Create Root Session]
    J -->|No| L{Bearer Token?}
    I -->|Bearer| L
    L -->|Yes| M[Verify JWT]
    L -->|No| G
    M -->|Valid| N{Token Revoked?}
    N -->|No| O[Create Session from Claims]
    N -->|Yes| G
    F --\u003e P[Inject Session into Request]
    K --\u003e P
    O --\u003e P
    P --\u003e Q[Continue to Handler]
```

## User Roles & Permissions

### Role Hierarchy

1. **Root** (`UserRole::Root`)
   - Full system access
   - Can manage all tenants
   - Can override tenant context via `X-Pangolin-Tenant` header
   - Authenticated via Basic Auth only

2. **TenantAdmin** (`UserRole::TenantAdmin`)
   - Full access within their tenant
   - Can manage users, warehouses, catalogs
   - Cannot access other tenants
   - Authenticated via Bearer tokens

3. **TenantUser** (`UserRole::TenantUser`)
   - Limited access based on permissions
   - Can read/write data based on grants
   - Cannot manage tenant resources
   - Authenticated via Bearer tokens

### Authorization Checks

**Location**: [authz.rs](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/authz.rs)

Authorization is enforced through:
- **Role-based checks**: Verify user role meets minimum requirement
- **Tenant isolation**: Ensure users can only access their tenant's resources
- **Permission grants**: Fine-grained access control via RBAC

## Session Management

### Session Structure

```rust
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub tenant_id: Option\u003cUuid\u003e,
    pub role: UserRole,
    pub expires_at: i64,
}
```

### Session Lifecycle

1. **Creation**: Session created during authentication
2. **Injection**: Inserted into request extensions via middleware
3. **Extraction**: Handlers extract session from `Extension\u003cUserSession\u003e`
4. **Expiration**: Tokens expire based on `exp` claim (default: 1 hour for Root, configurable for others)

## Public Endpoints

The following endpoints bypass authentication:

- `/health` - Health check
- `/api/v1/users/login` - User login
- `/api/v1/app-config` - Application configuration
- `/v1/config` - Iceberg REST config
- `*/config` - Any config endpoint
- `/oauth/authorize/*` - OAuth authorization flow
- `/oauth/callback/*` - OAuth callback handling

**Location**: [auth_middleware.rs:L158-167](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/auth_middleware.rs#L158-167)

## Admin User Provisioning

### Automatic Seeding

When `PANGOLIN_SEED_ADMIN=true` or `PANGOLIN_NO_AUTH=true`:

```bash
export PANGOLIN_SEED_ADMIN=true
export PANGOLIN_ADMIN_USER=admin
export PANGOLIN_ADMIN_PASSWORD=password
```

**Location**: [main.rs:L62-127](file:///home/alexmerced/development/personal/Personal/2026/pangolin/pangolin/pangolin_api/src/main.rs#L62-127)

#### What Happens
1. Creates default tenant (`00000000-0000-0000-0000-000000000000`)
2. Hashes the admin password
3. Creates a `TenantAdmin` user with ID `00000000-0000-0000-0000-000000000000`
4. Generates a long-lived JWT token (365 days)
5. Prints configuration banner with credentials and token

\u003e [!WARNING]
\u003e **The seeded admin user is a TENANT ADMIN, not a ROOT user**. They can only authenticate via Bearer tokens, not Basic Auth, unless you also set `PANGOLIN_ROOT_USER=admin` and `PANGOLIN_ROOT_PASSWORD=password`.

## Common Pitfalls & Troubleshooting

### Issue: "Missing or invalid authorization header"

**Symptoms**: All requests return 401 with this message

**Common Causes**:

1. **Using Basic Auth for tenant users**
   ```bash
   # ❌ WRONG - Tenant users cannot use Basic Auth
   curl -u tenant_admin:password http://localhost:8080/api/v1/catalogs
   
   # ✅ CORRECT - Use Bearer token
   TOKEN=$(curl -X POST http://localhost:8080/api/v1/users/login \\
     -H "Content-Type: application/json" \\
     -d '{"username":"tenant_admin","password":"password"}' \\
     | jq -r '.token')
   curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/catalogs
   ```

2. **Root credentials not set**
   ```bash
   # ❌ WRONG - No root credentials configured
   curl -u admin:password http://localhost:8080/api/v1/tenants
   
   # ✅ CORRECT - Set root credentials first
   export PANGOLIN_ROOT_USER=admin
   export PANGOLIN_ROOT_PASSWORD=password
   # Restart API
   curl -u admin:password http://localhost:8080/api/v1/tenants
   ```

3. **Seeded admin vs Root user confusion**
   ```bash
   # When PANGOLIN_SEED_ADMIN=true creates a user, it's a TENANT ADMIN
   # They need Bearer tokens, NOT Basic Auth
   
   # To use Basic Auth, you need BOTH:
   export PANGOLIN_SEED_ADMIN=true
   export PANGOLIN_ADMIN_USER=admin
   export PANGOLIN_ADMIN_PASSWORD=password
   export PANGOLIN_ROOT_USER=admin        # ← Also needed for Basic Auth
   export PANGOLIN_ROOT_PASSWORD=password # ← Also needed for Basic Auth
   ```

### Issue: Token expired

**Symptoms**: 401 with "Invalid token: token expired"

**Solution**: Obtain a new token via `/api/v1/users/login`

### Issue: Token revoked

**Symptoms**: 401 with "Token has been revoked"

**Solution**: Token was explicitly revoked. Obtain a new token.

### Issue: Different behavior across backends

**Symptoms**: Authentication works with MemoryStore but fails with PostgresStore

**Root Cause**: Environment variables not consistently set across test runs

**Solution**: Ensure all required environment variables are set for each backend:
```bash
# Required for ALL backends if using Basic Auth
export PANGOLIN_ROOT_USER=admin
export PANGOLIN_ROOT_PASSWORD=password

# Required for persistent backends (Postgres, Mongo, SQLite)
export PANGOLIN_SEED_ADMIN=true
export PANGOLIN_ADMIN_USER=admin
export PANGOLIN_ADMIN_PASSWORD=password
```

## Environment Variables Reference

| Variable | Purpose | Default | Required For |
|----------|---------|---------|--------------|
| `PANGOLIN_ROOT_USER` | Root username for Basic Auth | (empty) | Basic Auth as Root |
| `PANGOLIN_ROOT_PASSWORD` | Root password for Basic Auth | (empty) | Basic Auth as Root |
| `PANGOLIN_SEED_ADMIN` | Auto-create tenant admin on startup | `false` | Admin provisioning |
| `PANGOLIN_ADMIN_USER` | Seeded admin username | `tenant_admin` | Admin provisioning |
| `PANGOLIN_ADMIN_PASSWORD` | Seeded admin password | `password123` | Admin provisioning |
| `PANGOLIN_JWT_SECRET` | JWT signing secret | `default_secret_for_dev` | Token verification |
| `PANGOLIN_NO_AUTH` | Disable auth (dev only) | `false` | Development |

## Security Best Practices

1. **Production Deployment**
   - Set strong `PANGOLIN_JWT_SECRET`
   - Use HTTPS for all API communication
   - Rotate Root credentials regularly
   - Never use `PANGOLIN_NO_AUTH=true` in production

2. **Token Management**
   - Set appropriate token expiration times
   - Implement token refresh mechanism
   - Revoke tokens on logout
   - Store tokens securely on client side

3. **Password Security**
   - Passwords are hashed using bcrypt (cost factor 10)
   - Never log or expose password hashes
   - Enforce strong password policies

4. **API Keys**
   - Treat API keys like passwords
   - Set expiration dates
   - Revoke unused keys
   - Audit key usage regularly

## Testing Authentication

### Integration Test Setup

For PyIceberg integration tests or other automated testing:

```bash
# Start API with Root credentials for test setup
export PANGOLIN_ROOT_USER=admin
export PANGOLIN_ROOT_PASSWORD=password
export PANGOLIN_SEED_ADMIN=true
export PANGOLIN_ADMIN_USER=admin
export PANGOLIN_ADMIN_PASSWORD=password

# Run API
cargo run --bin pangolin_api

# Tests can now use Basic Auth for setup
curl -u admin:password http://localhost:8080/api/v1/tenants

# Or obtain Bearer token for tenant operations
TOKEN=$(curl -X POST http://localhost:8080/api/v1/users/login \\
  -H "Content-Type: application/json" \\
  -d '{"username":"admin","password":"password"}' \\
  | jq -r '.token')
```

## Related Documentation

- [User Management](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/api/users.md)
- [RBAC System](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/architecture/rbac.md)
- [API Reference](file:///home/alexmerced/development/personal/Personal/2026/pangolin/docs/api/README.md)
