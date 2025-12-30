# Auditing Reference

Pangolin provides comprehensive audit logs for governance.

## 1. API

**Base Endpoint**: `/api/v1/audit`

### List Audit Events
*   **Method**: `GET`
*   **Path**: `/api/v1/audit`
*   **Parameters**:
    *   `user_id`: Filter by actor.
    *   `action`: Filter by action (e.g., `create_table`).
    *   `resource_type`: Filter by type (e.g., `Catalog`).
    *   `result`: `Success` or `Failure`.
    *   `limit`, `offset`: Pagination.

```bash
curl "http://localhost:8080/api/v1/audit?action=create_table&limit=10" \
  -H "Authorization: Bearer <token>"
```

### Get Event Details
*   **Method**: `GET`
*   **Path**: `/api/v1/audit/{event_id}`

---

## 2. CLI

### List Events
```bash
# List recent
pangolin-admin list-audit-events --limit 20

# Filter by user
pangolin-admin list-audit-events --user-id <uuid>

# Filter by failure
pangolin-admin list-audit-events --result Failure
```

### Get Event
```bash
pangolin-admin get-audit-event --id <uuid>
```

---

## 3. Python SDK (`pypangolin`)

### List Events
```python
events = client.audit.list(
    limit=50,
    action="delete_table",
    result="Success"
)
for e in events:
    print(e.timestamp, e.username, e.action, e.resource_name)
```

### Get Event
```python
event = client.audit.get("event-uuid")
print(event.metadata)
```

---

## 4. UI

1.  **Log in** as **Tenant Admin**.
2.  Navigate to **Audit Logs** (usually in the **Governance** or **Admin** section).
3.  **Browse**: View the chronological list of events.
4.  **Filter**: Use the filter bar to search by Actor, Action, or Status.
5.  **Details**: Click on an event row to see full JSON metadata (IP address, User Agent, request details).
