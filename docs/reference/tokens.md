# Tokens Reference

Pangolin uses **JSON Web Tokens (JWT)** for user authentication.

## 1. API

**Base Endpoint**: `/api/v1/tokens`

### Create Token (Login)
*   **Method**: `POST`
*   **Path**: `/api/v1/users/login`
*   **Body**:
    ```json
    {
      "username": "alice",
      "password": "...",
      "tenant_id": "optional-uuid"
    }
    ```

### Revoke Current Token (Logout)
*   **Method**: `POST`
*   **Path**: `/api/v1/tokens/revoke`
*   **Auth**: Bearer Token
*   **Body**: `{}`

### Revoke Specific Token
*   **Method**: `POST`
*   **Path**: `/api/v1/tokens/revoke/{token_id}`
*   **Auth**: Admin

### List User Tokens
*   **Method**: `GET`
*   **Path**: `/api/v1/users/{user_id}/tokens`

---

## 2. CLI

### List User Tokens
```bash
pangolin-admin list-user-tokens --user-id <uuid>
```

### Revoke Token
```bash
# Revoke your current session
pangolin-admin revoke-token

# Revoke a specific token by ID
pangolin-admin revoke-token-by-id --id <token-uuid>
```

---

## 3. Python SDK (`pypangolin`)

### List Tokens
```python
tokens = client.tokens.list_user_tokens("user-uuid")
for t in tokens:
    print(t.token_id, t.is_valid)
```

### Revoke Token
```python
client.tokens.revoke_by_id("token-uuid")
```

---

## 4. UI

1.  **Log in** as **Tenant Admin**.
2.  Navigate to **Users**.
3.  Select a specific user (click name or "Edit").
4.  Navigate to the **Sessions/Tokens** tab.
5.  **List**: You will see active sessions.
6.  **Revoke**: Click **Revoke** next to a session to invalidate it immediately.
