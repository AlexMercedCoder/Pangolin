# 🔐 Logging In to Pangolin

This guide covers how to authenticate with Pangolin across all interfaces: **UI**, **CLI**, **API**, and **Python SDK**. It specifically explains when you need a **Tenant UUID** and how to find it.

---

## 🏗️ Core Concept: Root vs. Tenant Users

Pangolin has two types of users, which determines how you log in:

| User Type | Scope | Requires Tenant UUID? | Login Context |
| :--- | :--- | :--- | :--- |
| **Root User** | Superadmin for the entire system | **NO** | Authenticates against the System Root. Can manage all tenants. |
| **Tenant User** | Scoped to a specific Organization (Tenant) | **YES** | Authenticates against a specific Tenant. Username is only unique *within* that tenant. |

---

## 🆔 The Tenant UUID

### What is it?
A **Tenant UUID** (Universally Unique Identifier) is the unique ID for an Organization/Tenant (e.g., `a1b2c3d4-...`).

### When do I need it?
*   **Root User (`admin`)**: **NEVER** use a Tenant UUID. You must log in globally (send `null` or omit the ID).
    *   *Note:* Even in **No-Auth Mode**, the `admin` account is Global. If you try to log in with the Default Tenant UUID (`0000...`), it may fail because the `admin` user does not exist *inside* that tenant.
*   **Tenant User (e.g., `alice`)**: **ALWAYS** need it.
*   **Default Tenant Admin (`tenant_admin`)**: **ALWAYS** need it (Use `00000000-0000-0000-0000-000000000000`).
    *   *Note:* In **No-Auth Mode**, this is often the user you interact with implicitly, so it's easy to confuse this with Root.

### ⚠️ Common Confusion: No-Auth Mode
You might feel like you need a UUID in No-Auth mode. This is usually because:
1.  You are interacting with the **Default Tenant** (`0000...`), which requires that UUID.
2.  Or you are using the `root`/`root` client-side bypass (which is a UI implementation detail, not a real backend user).

**Rule of Thumb:**
- If you are `admin` (System Owner) -> **No UUID**.
- If you are anyone else (including Default Admin) -> **Use UUID**.

### 🔍 How to find your Tenant UUID

#### 1. Via UI (If you are already logged in as Root/Admin)
*   Go to the **Tenants** page.
*   The UUID is listed under the **ID** column for each tenant.

#### 2. Via CLI (If you have Root access)
```bash
pangolin tenants list
```
Output:
```text
ID                                     Name          Description
------------------------------------  ------------  ------------------
00000000-0000-0000-0000-000000000000  Default       Default Tenant
a1b2-c3d4-e5f6-g7h8                   Acme Corp     Primary Tenant
...
```

#### 3. Via API
```bash
curl -H "Authorization: Bearer <root-token>" http://localhost:8080/api/v1/tenants
```

---

## 🖥️ Logging In via UI

The Login screen adapts to your needs:

### As Root User (`admin`)
1.  Enter Username: `admin` (or your configured root user).
2.  Enter Password.
3.  **Leave "Tenant-specific login" unchecked.**
4.  Click **Sign In**.

### As Tenant User
1.  Enter Username.
2.  Enter Password.
3.  Check **"Tenant-specific login"**.
4.  Enter your **Tenant UUID**.
5.  Click **Sign In**.

---

## 💻 Logging In via CLI

### As Root User
```bash
# No tenant-id flag needed
pangolin login --username admin --password password123
```

### As Tenant User
```bash
# Must provide tenant-id
pangolin login --username alice --password secret --tenant-id <UUID>
```

---

## 🔌 Logging In via API

The API endpoint: `POST /api/v1/users/login`

### As Root User
Send `tenant-id` as `null` (or omit it entirely if supported by your client).

```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password123",
    "tenant-id": null
  }'
```

### As Tenant User
Send `tenant-id` with the UUID.

```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "secret",
    "tenant-id": "a1b2c3d4-..."
  }'
```

---

## 🐍 Logging In via Python SDK (`pypangolin`)

### As Root User
```python
from pangolin import PangolinClient

client = PangolinClient(base_url="http://localhost:8080")
client.authenticate("admin", "password123") # No tenant_id arg
```

### As Tenant User
```python
from pangolin import PangolinClient

client = PangolinClient(base_url="http://localhost:8080")
client.authenticate("alice", "secret", tenant_id="a1b2c3d4-...")
```
