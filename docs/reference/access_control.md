# Access Control Reference (RBAC & TBAC/ABAC)

Pangolin uses a multi-layered security model:
1.  **Tenant Isolation**: Strict separation of resources between tenants.
2.  **Role-Based Access Control (RBAC)**: Permissions assigned to users via roles.
3.  **Tag-Based Access Control (TBAC/ABAC)**: Permissions granted on assets based on their tags (business metadata).

## 1. Concepts

*   **Tenant Isolation**: Users are confined to their tenant. Cross-tenant access is impossible except for the Root user.
*   **Roles**:
    *   `root`: Global superuser.
    *   `tenant-admin`: Admin for a specific tenant.
    *   `tenant-user`: Regular user, requires explicit permissions.
*   **Permissions**: 
    *   **Scopes**: `Catalog`, `Namespace`, `Asset`, `Tag`.
    *   **Actions**: `Read`, `Write`, `Delete`, `Admin`.

## 2. API

**Base Endpoint**: `/api/v1/permissions`

### Grant Permission (Standard RBAC)
*   **Method**: `POST`
*   **Body**:
    ```json
    {
      "user_id": "uuid-of-user",
      "scope": "Catalog",
      "resource": "analytics",
      "action": "Read"
    }
    ```

### Grant Permission (Tag-Based / ABAC)
Grant access to *any* asset that checks the "PII" tag.
*   **Body**:
    ```json
    {
      "user_id": "uuid-of-compliance-officer",
      "scope": "Tag",
      "resource": "PII", 
      "action": "Read"
    }
    ```
    *Note: For Tag scope, the `resource` field is the Tag Name.*

### Revoke Permission
*   **Method**: `DELETE`
*   **Path**: `/api/v1/permissions/{permission_id}`

### List Permissions
*   **Method**: `GET`
*   **Path**: `/api/v1/permissions`
*   **Params**: `user={uuid}`, `role={role_name}`

---

## 3. CLI

### Grant Permission
```bash
# Grant Read on 'analytics' catalog (RBAC)
pangolin-admin grant-permission \
  --username alice \
  --action Read \
  --resource analytics

# Grant Read on all assets tagged 'Public' (TBAC)
pangolin-admin grant-permission \
  --username alice \
  --action Read \
  --scope Tag \
  --resource Public
```

### Revoke Permission
```bash
pangolin-admin revoke-permission --id <permission-uuid>
```

### List Permissions
```bash
# List all permissions
pangolin-admin list-permissions

# Filter by user
pangolin-admin list-permissions --user alice
```

---

## 4. Python SDK (`pypangolin`)

### Grant
```python
# RBAC
client.permissions.grant(
    user_id="uuid...",
    scope="Catalog",
    resource="analytics", 
    action="Read"
)

# TBAC (Tag-Based)
client.permissions.grant(
    user_id="uuid...",
    scope="Tag",
    resource="Confidential", 
    action="Read"
)
```

### List
```python
perms = client.permissions.list(user_id="uuid...")
for p in perms:
    print(f"{p.scope}: {p.resource} -> {p.action}")
```

### Revoke
```python
client.permissions.revoke(permission_id="uuid...")
```

---

## 5. UI

1.  **Log in** as a **Tenant Admin**.
2.  Navigate to **Permissions**.
3.  **Grant**: Click **"Grant Permission"**.
    *   Select **User** or **Role**.
    *   Select **Scope**: Choose **Tag** for ABAC/TBAC strategies.
    *   Enter **Resource**: The name of the tag (e.g., `PII`, `Financial`).
    *   Select **Action**.
    *   Click **Grant**.
