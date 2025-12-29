# Authentication Modes & Guide

Pangolin supports two distinct operational modes: **Auth Mode** (Production) and **No-Auth Mode** (Development). This guide details how to configure, access, and manage identity in both scenarios.

---

## 1. Auth Mode vs. No-Auth Mode

| Feature | **Auth Mode** (Default) | **No-Auth Mode** |
| :--- | :--- | :--- |
| **Use Case** | Production, Multi-Tenancy, Public Internet | Local Dev, CI/CD, Rapid Prototyping |
| **Security** | Enforced via JWT Tokens | Disabled (Open Access) |
| **Identity** | Users must log in; Identity is verified | Requests default to `admin`; Password checks bypassed |
| **Setup** | Requires `PANGOLIN_JWT_SECRET` | Requires `PANGOLIN_NO_AUTH=true` |
| **Tenant Context** | Strict isolation; Users belong to 1 Tenant | Default Tenant used automatically if unspecified |

---

## 2. Configuration (Environment Variables)

### Root User Credentials
The **Root User** is the system superadmin. These credentials are used to bootstrap the system (create the first tenant).

**Auth Mode**:
```bash
# Secure credentials for production
export PANGOLIN_ROOT_USER="super_admin"
export PANGOLIN_ROOT_PASSWORD="Start123!ComplexPassword"
export PANGOLIN_JWT_SECRET="<random-32-char-string>"
```

**No-Auth Mode**:
```bash
# Credentials are ignored, but variables can still be set
export PANGOLIN_NO_AUTH="true"
```

---

## 3. UI Login Nuances

The Management UI adapts its login form based on the detected mode.

### Auth Mode UI
*   **Standard Login**: Requires **Username**, **Password**, and **Tenant ID**.
*   **Root Login**: Users must toggle the **"Login as System Root"** switch (or use the dedicated link). This restricts the form to just **Username** and **Password** (as Root has no tenant).
*   **Behavior**: Invalid credentials return a 401 error. Session expires when the JWT expires.

### No-Auth Mode UI
*   **Auto-Detection**: The UI detects `NO_AUTH` state on load.
*   **Simplified Form**: Password validation is skipped on the backend.
*   **"Easy Login"**: You can often just type a username (e.g., `admin`) and hit Enter to simulate logging in as that user for testing RBAC visibility.
*   **Root Access**: Root login works with *any* password, provided the username matches `PANGOLIN_ROOT_USER`.

---

## 4. Root User Workflows

The Root User's primary job is **System bootstrapping**. They generally do **not** query data.

### Workflow A: Logging In as Root

**Via API (cURL)**:
```bash
# Auth Mode: Must provide correct password & null tenant-id
curl -X POST http://localhost:8080/api/v1/users/login \
  -d '{"username":"super_admin", "password":"...", "tenant-id":null}'

# No-Auth Mode: Password can be anything
curl -X POST http://localhost:8080/api/v1/users/login \
  -d '{"username":"any", "password":"any", "tenant-id":null}'
```

### Workflow B: Creating a Tenant & Admin (Bootstrapping)

Only the Root user can create tenants.

**1. Create Tenant**:
```bash
# Requires Root Token
POST /api/v1/tenants
{
  "name": "acme-corp"
}
# Response: Returns { "id": "uuid-for-acme", ... }
```

**2. Create Tenant Admin**:
```bash
# Requires Root Token
# Note: You are creating a user *inside* the new tenant
POST /api/v1/users
{
  "username": "acme_admin",
  "password": "temporary_password",
  "tenant_id": "uuid-for-acme",  # Critical: Links user to tenant
  "role": "TenantAdmin"
}
```

---

## 5. Token Generation Guide

Authentication tokens (JWTs) act as your API keys for all operations.

### Generating a Token (Generic)

**Endpoint**: `POST /api/v1/tokens` (Requires Admin privileges) or `POST /api/v1/users/login` (Self-service).

**Payload**:
```json
{
  "username": "data_engineer",
  "password": "secure_password",
  "tenant_id": "uuid-for-acme"
}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1Ni...",
  "expires_in": 86400,
  "tenant_id": "uuid-for-acme"
}
```

### Using the Token
Pass it in the HTTP Header:
```http
Authorization: Bearer eyJhbGciOiJIUzI1Ni...
```

---

## 6. Permissions Matrix: Who can touch what?

Pangolin employs a strict hierarchy. Here is what each user type can access.

| Resource | **Root User** | **Tenant Admin** | **Tenant User** |
| :--- | :--- | :--- | :--- |
| **Manage Tenants** | ✅ **Create/Delete** | ❌ No Access | ❌ No Access |
| **Manage Users** | ✅ (Global) | ✅ (Own Tenant Only) | ❌ No Access |
| **Create Warehouses** | ❌ (Out of Scope) | ✅ **Full Control** | ❌ No Access |
| **Create Catalogs** | ❌ (Out of Scope) | ✅ **Full Control** | ❌ No Access |
| **Manage Roles** | ❌ (Out of Scope) | ✅ **Full Control** | ❌ No Access |
| **Query Data** | ❌ **No Access*** | ✅ **Full Access** | ⚠️ **By Permission Only** |
| **View Audit Logs** | ✅ (System Wide) | ✅ (Own Tenant Only) | ❌ No Access |

> ***Note on Root Data Access**: By design, the Root user cannot query data (tables/views) directly. They must authenticate as a Tenant Admin or specific User to modify actual data. This enforcement ensures "System Administrators" cannot casually browse sensitive data "Business Data".

### Tenant User Permissions (RBAC)
A standard **Tenant User** starts with **ZERO** permissions. They cannot see Catalogs or query Tables until granted a Role:

*   **Reader Role**: Can `SELECT` from tables.
*   **Writer Role**: Can `INSERT`/`UPDATE`/`DELETE`.
*   **Discovery Role**: Can search for assets but not query contents.

See [Permissions Management](../best-practices/permissions.md) for how to assign these granular rights.
