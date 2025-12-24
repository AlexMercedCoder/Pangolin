# Admin CLI - Audit Logging

This guide covers the audit logging commands available in the Pangolin Admin CLI. These commands allow administrators to track user actions, resource changes, and security events across the platform.

## List Audit Events

List audit events with support for comprehensive filtering and pagination.

### Syntax
```bash
pangolin-admin list-audit-events [OPTIONS]
```

### Parameters
- `--user-id <UUID>` - Filter by user ID
- `--action <STRING>` - Filter by action (e.g., `create_table`, `grant_permission`)
- `--resource-type <STRING>` - Filter by resource type (e.g., `table`, `catalog`, `user`)
- `--result <STRING>` - Filter by result (`success` or `failure`)
- `--tenant-id <UUID>` - Filter by tenant ID (Root users only - see cross-tenant queries)
- `--limit <NUMBER>` - Maximum number of results to return (default: 50)

### Examples

**List the 50 most recent events:**
```bash
pangolin-admin list-audit-events
```

**Filter by action and limit results:**
```bash
pangolin-admin list-audit-events --action create_table --limit 10
```

**Filter by user and failed actions:**
```bash
pangolin-admin list-audit-events --user-id "550e8400-e29b-41d4-a716-446655440000" --result failure
```

**Filter by tenant (Root users only):**
```bash
# View audit events for a specific tenant
pangolin-admin list-audit-events --tenant-id "tenant-uuid-here" --limit 20
```

### Output
```
ID        User          Action          Resource Type   Resource Name   Result   Timestamp
ab12cd34  admin         create_table    table          sales_data      success  2025-12-18T10:30:00
ef56gh78  user1         drop_table      table          old_data        failure  2025-12-18T10:35:00
...
Showing 2 events
```

---

## Count Audit Events

Get the total count of audit events matching specific criteria.

### Syntax
```bash
pangolin-admin count-audit-events [OPTIONS]
```

### Parameters
- `--user-id <UUID>` - Filter by user ID
- `--action <STRING>` - Filter by action
- `--resource-type <STRING>` - Filter by resource type
- `--result <STRING>` - Filter by result

### Examples

**Get total event count:**
```bash
pangolin-admin count-audit-events
```

**Count failure events:**
```bash
pangolin-admin count-audit-events --result failure
```

### Output
```
Total audit events: 1542
```

---

## Get Audit Event

Retrieve full details for a specific audit event by its ID.

### Syntax
```bash
pangolin-admin get-audit-event --id <UUID>
```

### Parameters
- `--id <UUID>` - The unique identifier of the audit event (required)

### Example
```bash
pangolin-admin get-audit-event --id "ab12cd34-e29b-41d4-a716-446655440000"
```

### Output
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Audit Event Details
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ID: ab12cd34-e29b-41d4-a716-446655440000
User: admin
User ID: 550e8400-e29b-41d4-a716-446655440000
Action: create_table
Resource: sales_data (table)
Result: success
Timestamp: 2025-12-18T10:30:00.123456Z
IP Address: 192.168.1.100
User Agent: pangolin-cli/0.1.0
Metadata:
  schema: public
  format: parquet
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Use Cases

### Investigation
1. **Identify User Activity**: Filter by `user_id` to see all actions performed by a specific user.
2. **Trace Resource Changes**: Filter by `resource_type` and `resource_name` (via grep) to see history of a specific asset.
3. **Security Auditing**: key filter `result=failure` to identify potential unauthorized access attempts or system errors.

### Compliance
- Export audit logs regularly using `list-audit-events` and piping to a file.
- Verify actions match approved change requests.

## Notes
- **Tenant Isolation**: 
  - Tenant users only see audit logs for their own tenant
  - Root users can see all audit logs across all tenants
  - Use `--tenant-id` filter to query specific tenant logs as Root
- **Retention**: Audit logs are retained indefinitely by default (configurable per deployment).
- **Performance**: Pagination (`--limit`, `--offset`) is recommended for large datasets.
