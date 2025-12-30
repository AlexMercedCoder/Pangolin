# Users Reference

Users are identities that can log in and interact with Pangolin resources. Users (except Root) are scoped to a specific Tenant.

## 1. API

**Base Endpoint**: `/api/v1/users`

### List Users
*   **Method**: `GET`
*   **Path**: `/api/v1/users`
*   **Auth**: Tenant Admin Token (or Root with Tenant Context)
*   **Headers**: `X-Pangolin-Tenant: <tenant-uuid>` (if using Root token)

```bash
curl -X GET http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer <token>"
```

### Create User
*   **Method**: `POST`
*   **Path**: `/api/v1/users`
*   **Body**:
    ```json
    {
      "username": "alice",
      "email": "alice@example.com",
      "password": "password123",
      "role": "tenant-user",
      "tenant_id": "<optional-uuid>" 
    }
    ```
    > **Note**: `role` must be `tenant-user`, `tenant-admin`, or `root`.

```bash
curl -X POST http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "email": "alice@ex.com", "password": "pw", "role": "tenant-user"}'
```

---

## 2. CLI

Ensure you have selected a tenant first using `pangolin-admin use <tenant>`.

### List Users
```bash
pangolin-admin list-users
```

### Create User
```bash
# Create a standard user
pangolin-admin create-user alice \
  --email alice@example.com \
  --role tenant-user \
  --password "securePass123"

# Create a tenant admin
pangolin-admin create-user bob-admin \
  --email bob@example.com \
  --role tenant-admin
```

### Delete User
```bash
pangolin-admin delete-user alice
```

---

## 3. Python SDK (`pypangolin`)

### List Users
```python
users = client.users.list()
for u in users:
    print(u.username, u.role)
```

### Create User
```python
# Create standard user
user = client.users.create(
    username="alice",
    email="alice@example.com",
    role="tenant-user", # Case-sensitive: tenant-user or tenant-admin
    password="password123"
)
print(f"Created user {user.id}")
```

---

## 4. UI

1.  **Log in** as a **Tenant Admin** (or Root switched to a tenant).
2.  Navigate to **Users** in the sidebar.
3.  **List**: You will see a table of users in the current tenant.
4.  **Create**: Click **"Create User"**.
    *   Fill in Username, Email, Password.
    *   Select Role: **Tenant User** or **Tenant Admin**.
    *   Click **Create**.
5.  **Manage**: Click the "Edit" icon to update details or "Trash" icon to delete.
