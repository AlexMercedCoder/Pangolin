# RBAC & UI Authentication

Pangolin implements a Role-Based Access Control (RBAC) system to secure the API and UI.

## Roles

- **Root**: Superuser with full access to all tenants and system configuration. Authenticated via Basic Auth (Env Vars).
- **Admin**: Tenant-level administrator. Can manage users and warehouses within a tenant.
- **User**: Standard user. Can read/write data based on permissions.

## Authentication Methods

### 1. Root User (Basic Auth)
The Root User is defined via environment variables:
- `PANGOLIN_ROOT_USER`
- `PANGOLIN_ROOT_PASSWORD`

Requests must include the header: `Authorization: Basic <base64(user:pass)>`.

### 2. JWT Authentication (Bearer Token)
Standard users authenticate via JWTs signed with `PANGOLIN_JWT_SECRET`.
The JWT claims must include:
- `sub`: User ID
- `roles`: List of roles (e.g., `["Admin"]`)
- `tenant_id`: (Optional) Tenant ID for context

## UI Authentication

The Management UI (`pangolin_ui`) supports login via the `/login` page.
- Currently supports Root User login.
- Credentials are verified against the API and stored locally.
- Protected routes redirect to `/login` if no valid session is found.
