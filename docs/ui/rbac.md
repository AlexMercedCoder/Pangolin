# Role-Based Access Control (RBAC)

Pangolin implements a comprehensive RBAC system to manage access to catalogs, namespaces, and assets. The system is built on a 3-tier permission model combined with granular, dynamic roles.

For a high-level overview of the permission system, see [Permissions System](../permissions.md) and [Authentication](../authentication.md).

This document details the **User Interface** aspects of managing Roles and Users in Pangolin.

## 3-Tier Permission Model

The core authorization hierarchy consists of three main user roles:

1.  **Root User**:
    *   **Scope**: Global System Administrator.
    *   **Capabilities**: Full access to all tenants, system configuration, and user management.
    *   **Limitation**: Defined at system initialization (or first user).

2.  **Tenant Admin**:
    *   **Scope**: Specific Tenant.
    *   **Capabilities**: Full access to the tenant's resources (warehouses, catalogs), user management within the tenant, and role assignment.
    *   **Limitation**: Cannot access other tenants' data.

3.  **Tenant User**:
    *   **Scope**: Specific Tenant.
    *   **Capabilities**: Constrained by assigned permissions and dynamic roles.
    *   **Default**: Basic read access ( configurable).

## Dynamic Roles & Permissions

Beyond the 3-tier model, Pangolin supports dynamic roles for fine-grained access control.

### Concepts

*   **Role**: A named collection of permissions (e.g., "Data Engineer", "Viewer").
*   **Permission**: A specific grant of an action on a scope.
*   **Action**: What can be done (e.g., `READ`, `WRITE`, `CREATE`, `DELETE`, `LIST`).
*   **Scope**: Where it applies.
    *   `Catalog`: Entire catalog.
    *   `Namespace`: Specific namespace.
    *   `Asset`: Specific table or view.
    *   `Tag`: All assets with a specific tag (e.g., "PII").

### Permissions Matrix

| Action | Description |
| :--- | :--- |
| `READ` | Can read data and metadata. |
| `WRITE` | Can insert, update, or overwrite data. |
| `DELETE` | Can delete assets or data. |
| `CREATE` | Can create new assets or namespaces. |
| `UPDATE` | Can update metadata or schema. |
| `LIST` | Can list assets (discovery). |
| `ALL` | Full control. |

## API Reference

### Role Management

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/api/v1/roles` | Create a new role definition. |
| `GET` | `/api/v1/roles` | List all roles for the tenant. |
| `GET` | `/api/v1/roles/{id}` | Get details of a specific role. |
| `PUT` | `/api/v1/roles/{id}` | Update role details. |
| `DELETE` | `/api/v1/roles/{id}` | Delete a role definition. |

**Create Role Body**:
```json
{
  "name": "Data Engineer",
  "description": "Access to write and manage tables",
  "tenant-id": "uuid-..."
}
```

### Permission Management

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/api/v1/permissions` | Grant a direct permission to a user. |
| `DELETE` | `/api/v1/permissions/{id}` | Revoke a permission. |
| `GET` | `/api/v1/permissions/user/{id}` | List effective permissions for a user. |

**Grant Permission Body**:
```json
{
  "user-id": "uuid-...",
  "scope": {
    "type": "namespace",
    "catalog-id": "uuid-...",
    "namespace": "marketing.prod"
  },
  "actions": ["read", "write"]
}
```

### Role Assignment

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/api/v1/users/{id}/roles` | Assign a role to a user. |
| `DELETE` | `/api/v1/users/{id}/roles/{role-id}` | Remove a role from a user. |

**Assign Role Body**:
```json
{
  "role-id": "uuid-..."
}
```

## Implementation Notes

*   **Storage**: Roles and permissions are stored in the backend (Memory, S3, etc.) and cached for performance.
*   **Precedence**: `Root` > `TenantAdmin` > Specific Permissions. Explicit `DENY` (future) will override `ALLOW`.
*   **Audit**: All permission changes are logged to the audit trail.
