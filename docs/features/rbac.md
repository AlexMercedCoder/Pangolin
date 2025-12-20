# Role-Based Access Control (RBAC)

## Overview

Pangolin implements a hierarchical role-based access control system to manage permissions across tenants, catalogs, and resources.

**Key Features**:
- 3-tier role hierarchy (Root, TenantAdmin, TenantUser)
- Granular permission scopes (Catalog, Namespace, Asset, Tag)
- Service user support for programmatic access
- Tenant isolation

---

## User Roles

### Root
- **Scope**: Global (all tenants)
- **Permissions**: Full system access
- **Can**:
  - Create/delete tenants
  - Manage all users across all tenants
  - Access all catalogs and resources
  - Configure system-wide settings

**Use Case**: System administrators

### TenantAdmin
- **Scope**: Single tenant
- **Permissions**: Full access within their tenant
- **Can**:
  - Create/delete users in their tenant
  - Create/delete catalogs, warehouses
  - Manage permissions for their tenant
  - Create service users
  - Create federated catalogs

**Cannot**:
  - Access other tenants
  - Create new tenants
  - Modify system settings

**Use Case**: Team leads, data platform administrators

### TenantUser
- **Scope**: Single tenant with granular permissions
- **Permissions**: Based on assigned permissions
- **Can**:
  - Read/write to authorized catalogs
  - Create tables in authorized namespaces
  - Query authorized tables

**Cannot**:
  - Create users or service users
  - Modify tenant settings
  - Access unauthorized resources

**Use Case**: Data engineers, data scientists, analysts

---

## Permission Scopes

### Catalog-Level
Permissions apply to an entire catalog and all its namespaces/tables.

**Example**: Grant user read access to `analytics` catalog
```json
{
  "user_id": "uuid",
  "scope": "Catalog",
  "resource": "analytics",
  "action": "Read"
}
```

### Namespace-Level
Permissions apply to a specific namespace and its tables.

**Example**: Grant user write access to `sales.transactions` namespace
```json
{
  "user_id": "uuid",
  "scope": "Namespace",
  "resource": "analytics/sales",
  "action": "Write"
}
```

### Asset-Level
Permissions apply to a specific table or view.

**Example**: Grant user read access to specific table
```json
{
  "user_id": "uuid",
  "scope": "Asset",
  "resource": "analytics/sales/transactions",
  "action": "Read"
}
```

---

## Permission Actions

- **Read**: View metadata, query data
- **Write**: Create tables, insert/update data
- **Delete**: Drop tables, delete data
- **Admin**: Full control (create, read, update, delete)

---

## Managing Users

### Create User

```bash
POST /api/v1/users
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "username": "data_engineer",
  "email": "engineer@example.com",
  "password": "secure-password",
  "role": "TenantUser"
}
```

### Assign Role to User

```bash
POST /api/v1/users/{user_id}/roles
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "role_id": "role-uuid"
}
```

### Grant Permission (to User or Role)

```bash
POST /api/v1/permissions
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "user_id": "optional-user-uuid",
  "role_id": "optional-role-uuid",
  "scope": "Catalog",
  "resource": "analytics",
  "action": "Read"
}
```

### Revoke Permission

```bash
DELETE /api/v1/permissions/{permission_id}
Authorization: Bearer <admin-token>
```

---

## Service Users

Service users are programmatic identities with API key authentication. See [Service Users](../service_users.md) for details.

**Key Points**:
- Service users inherit the same RBAC system
- Can be assigned TenantAdmin or TenantUser roles
- Authenticate via `X-API-Key` header
- Support expiration and rotation

---

## Best Practices

### 1. **Principle of Least Privilege**
Grant minimum necessary permissions:
- Use TenantUser role by default
- Grant catalog-level access only when needed
- Prefer namespace or asset-level permissions

### 2. **Separate Service Users by Purpose**
Create dedicated service users for each use case:
```
- ci_pipeline_user (Write to staging catalog)
- analytics_reader (Read from analytics catalog)
- etl_processor (Write to raw, read from staging)
```

### 3. **Regular Permission Audits**
Review user permissions periodically:
```bash
GET /api/v1/users/{user_id}/permissions
```

### 4. **Use Groups (Future)**
When available, use groups to manage permissions for teams.

### 5. **Monitor Access**
Review audit logs for unauthorized access attempts:
```bash
GET /api/v1/audit?user_id={user_id}
```

---

## Permission Inheritance

Permissions follow a hierarchical model:

```
Root
  └─ TenantAdmin (Tenant A)
      └─ TenantUser (Catalog: analytics)
          └─ Namespace: sales
              └─ Asset: transactions
```

**Rules**:
- Root has access to everything
- TenantAdmin has access to all resources in their tenant
- TenantUser permissions are explicitly granted
- More specific permissions override general ones

---

## Common Scenarios

### Scenario 1: Data Engineer with Full Catalog Access

```bash
# Create user
POST /api/v1/users
{
  "username": "data_engineer",
  "role": "TenantUser"
}

# Grant catalog-level write access
POST /api/v1/permissions
{
  "user_id": "uuid",
  "scope": "Catalog",
  "resource": "analytics",
  "action": "Write"
}
```

### Scenario 2: Analyst with Read-Only Access

```bash
# Create user
POST /api/v1/users
{
  "username": "analyst",
  "role": "TenantUser"
}

# Grant namespace-level read access
POST /api/v1/permissions
{
  "user_id": "uuid",
  "scope": "Namespace",
  "resource": "analytics/sales",
  "action": "Read"
}
```

### Scenario 3: CI/CD Pipeline

```bash
# Create service user
POST /api/v1/service-users
{
  "name": "ci_pipeline",
  "role": "TenantUser",
  "expires_in_days": 90
}

# Grant write access to staging catalog
POST /api/v1/permissions
{
  "user_id": "service_user_uuid",
  "scope": "Catalog",
  "resource": "staging",
  "action": "Write"
}
```

---

## Troubleshooting

### "Forbidden" errors

**Cause**: User lacks required permissions.

**Solution**:
1. Check user's role: `GET /api/v1/users/{user_id}`
2. List user's permissions: `GET /api/v1/users/{user_id}/permissions`
3. Grant missing permission

### Permission not taking effect

**Cause**: Permission cache or session issue.

**Solution**:
1. Re-authenticate to get new JWT token
2. Verify permission was created: `GET /api/v1/permissions`

---

## Related Documentation

- [Service Users](../service_users.md) - API key authentication
- [Authentication](../authentication.md) - User authentication
- [Audit Logs](./audit_logs.md) - Access monitoring
- [Permissions System](../permissions.md) - Permission details
