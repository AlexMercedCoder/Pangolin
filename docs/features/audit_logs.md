# Audit Logging

Pangolin provides a comprehensive, multi-tenant audit logging system. Every action—from login attempts to table schema evolutions—is recorded with full context, providing an immutable trail for security, compliance, and debugging.

---

## 🔒 Multi-Tenant Isolation

Audit logs are strictly partitioned by **Tenant ID**.
- **Root Admin**: Can view logs for the platform itself (tenant creation, global settings).
- **Tenant Admin**: Can view all logs belonging to their specific organization.
- **Tenant User**: Only sees logs for their own individual actions.

> [!NOTE]
> Tenant isolation is enforced at the storage driver level. It is physically impossible for a User in Tenant A to query logs for Tenant B.

---

## 🛠️ Usage Guides

### 1. Via CLI (`pangolin-admin`)
The admin tool provides powerful filtering and counting capabilities for security reviews.

**List Recent Events:**
```bash
# List last 50 events for a specific user
pangolin-admin list-audit-events --user-id <uuid> --limit 50

# Filter by failed actions
pangolin-admin list-audit-events --result failure
```

**Count Events:**
```bash
# Get total count of table creations
pangolin-admin count-audit-events --action create_table
```

### 2. Via Management UI
1. Navigate to **Administration -> Audit Logs**.
2. Use the **Filter Sidebar** to narrow down by User, Action, or Time Range.
3. Click on any row to open the **Event Detail Modal**, which displays the raw JSON metadata.

### 3. Via REST API
Integrate Pangolin logs into your own SIEM (Splunk, Datadog) using the audit endpoints.

**Fetch Logs:**
```bash
GET /api/v1/audit?action=commit_table&limit=100
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

---

## 📂 Event Data Model

Every log entry follows a standardized structure:

| Field | Description |
| :--- | :--- |
| `id` | Unique UUID for the log entry. |
| `user_id` | UUID of the actor. |
| `username` | Readable name of the actor. |
| `action` | The operation (e.g., `create_table`, `merge_branch`). |
| `resource` | Path to the affected object (e.g., `catalog/ns/table`). |
| `result` | `success` or `failure`. |
| `timestamp` | UTC ISO8601 string. |
| `metadata` | Action-specific JSON (e.g., old/new schema ID). |

---

## 🧬 Logged Actions Detail

| Category | Actions |
| :--- | :--- |
| **Catalog** | `create_catalog`, `delete_catalog`, `update_catalog` |
| **Branching** | `create_branch`, `delete_branch`, `merge_branch` |
| **Security** | `login`, `grant_permission`, `rotate_api_key` |
| **Maintenance** | `expire_snapshots`, `remove_orphan_files` |
| **Data (Iceberg)** | `create_table`, `drop_table`, `commit_table` |

---

## 🚦 Best Practices
- **Regular Audits**: Schedule a weekly review of `failure` results to identify potential misconfigurations or attack patterns.
- **Webhook Integration**: Export high-severity actions (like `drop_table` or `revoke_permission`) to your alerting system.
- **Retention**: Configure `PANGOLIN_AUDIT_RETENTION_DAYS` (default 90) to match your legal requirements.
