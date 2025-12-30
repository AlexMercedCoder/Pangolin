# Service Users Reference

Service Users are headless accounts for machine-to-machine interaction (CI/CD, ETL, etc.). They authenticate via **API Keys**.

## 1. API

**Base Endpoint**: `/api/v1/service-users`

### Create Service User
*   **Method**: `POST`
*   **Body**:
    ```json
    {
      "name": "ci-bot",
      "role": "tenant-user",
      "expires_in_days": 90
    }
    ```
*   **Response**: Returns the **API Key** (only once).

### Rotate Key
*   **Method**: `POST`
*   **Path**: `/api/v1/service-users/{id}/rotate`

---

## 2. CLI

### Create
```bash
pangolin-admin create-service-user \
  --name "ci-bot" \
  --role "tenant-user" \
  --expires-in-days 365
```
**Output**:
```
API Key: pgl_AbCd...
(Save this key!)
```

### List
```bash
pangolin-admin list-service-users
```

### Rotate Key
```bash
pangolin-admin rotate-service-user-key --id <uuid>
```

---

## 3. Python SDK (`pypangolin`)

### Create
```python
su = client.service_users.create(
    name="airflow-bot",
    role="tenant-user",
    expires_in_days=30
)
print(f"ID: {su.id}")
print(f"Key: {su.api_key}") # Only available here
```

### Rotate
```python
new_creds = client.service_users.rotate_key("uuid...")
print(f"New Key: {new_creds.api_key}")
```

---

## 4. UI

1.  **Log in** as a **Tenant Admin**.
2.  Navigate to **Service Users** (often a tab within **Users** or a separate sidebar item).
3.  **Create**: Click **"Create Service User"**.
    *   Enter Name and Role.
    *   Set Expiration.
    *   **COPY THE KEY**: A modal will show the key. Copy it immediately.
4.  **Rotate**: Select a service user -> **Rotate Key**. A new key is generated and shown.
