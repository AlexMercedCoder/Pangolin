# Audit Logs

Pangolin provides a structured audit logging system to track critical actions performed on the catalog.

## Data Model

## Overview

Pangolin automatically tracks all catalog operations in audit logs, providing a complete trail of who did what, when, and where. This is essential for compliance, security monitoring, and troubleshooting.

**Key Features**:
- Automatic logging of all catalog operations
- User attribution (who performed the action)
- Timestamp tracking (when it happened)
- Resource identification (what was affected)
- Tenant isolation (logs are tenant-specific)
- Queryable via REST API

---

## Logged Events

### Catalog Operations
- `create_catalog` - Catalog created
- `delete_catalog` - Catalog deleted
- `update_catalog` - Catalog modified

### Namespace Operations
- `create_namespace` - Namespace created
- `delete_namespace` - Namespace deleted
- `update_namespace` - Namespace properties updated

### Table Operations
- `create_table` - Table created
- `drop_table` - Table dropped
- `rename_table` - Table renamed
- `update_table` - Table schema or properties updated
- `commit_table` - Table commit (data write)

### Branch Operations
- `create_branch` - Branch created
- `delete_branch` - Branch deleted
- `merge_branch` - Branch merged
- `initiate_merge` - Merge operation started
- `complete_merge` - Merge operation completed
- `abort_merge` - Merge operation aborted

### User Management
- `create_user` - User created
- `delete_user` - User deleted
- `update_user` - User modified
- `login` - User logged in
- `logout` - User logged out

### Service User Operations
- `create_service_user` - Service user created
- `delete_service_user` - Service user deleted
- `rotate_api_key` - API key rotated

### Permission Operations
- `grant_permission` - Permission granted
- `revoke_permission` - Permission revoked
- `assign_role` - Role assigned to user

### Federated Catalog Operations
- `create_federated_catalog` - Federated catalog created
- `delete_federated_catalog` - Federated catalog deleted
- `test_federated_connection` - Connection test performed

---

## Querying Audit Logs

### List All Audit Logs

```bash
GET /api/v1/audit
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

**Response**:
```json
[
  {
    "id": "uuid",
    "tenant_id": "uuid",
    "user_id": "uuid",
    "username": "data_engineer",
    "action": "create_table",
    "resource": "analytics/sales/transactions",
    "timestamp": "2024-01-15T10:30:00Z",
    "metadata": {
      "catalog": "analytics",
      "namespace": "sales",
      "table": "transactions"
    }
  }
]
```

### Filter by User

```bash
GET /api/v1/audit?user_id=<user-uuid>
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

### Filter by Action

```bash
GET /api/v1/audit?action=create_table
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

### Filter by Time Range

```bash
GET /api/v1/audit?start_time=2024-01-01T00:00:00Z&end_time=2024-01-31T23:59:59Z
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

### Filter by Resource

```bash
GET /api/v1/audit?resource=analytics/sales/transactions
Authorization: Bearer <token>
X-Pangolin-Tenant: <tenant-id>
```

---

## Audit Log Structure

```json
{
  "id": "uuid",
  "tenant_id": "uuid",
  "user_id": "uuid",
  "username": "data_engineer",
  "action": "create_table",
  "resource": "analytics/sales/transactions",
  "timestamp": "2024-01-15T10:30:00Z",
  "ip_address": "192.168.1.100",
  "user_agent": "PyIceberg/0.5.0",
  "metadata": {
    "catalog": "analytics",
    "namespace": "sales",
    "table": "transactions",
    "schema_id": 1,
    "partition_spec_id": 0
  },
  "result": "success"
}
```

**Fields**:
- `id`: Unique log entry ID
- `tenant_id`: Tenant that owns the resource
- `user_id`: User who performed the action
- `username`: Username for readability
- `action`: Operation performed
- `resource`: Resource affected (catalog/namespace/table)
- `timestamp`: When the action occurred (UTC)
- `ip_address`: Client IP address (if available)
- `user_agent`: Client user agent
- `metadata`: Additional context (varies by action)
- `result`: `success` or `failure`

---

## Use Cases

### Security Monitoring

**Detect Unauthorized Access Attempts**:
```bash
GET /api/v1/audit?result=failure&action=login
```

**Track Sensitive Operations**:
```bash
GET /api/v1/audit?action=drop_table
```

### Compliance

**Generate Access Reports**:
```bash
GET /api/v1/audit?start_time=2024-01-01&end_time=2024-01-31
```

**Track Data Modifications**:
```bash
GET /api/v1/audit?action=commit_table&resource=analytics/pii/customers
```

### Troubleshooting

**Investigate Table Issues**:
```bash
GET /api/v1/audit?resource=analytics/sales/transactions&start_time=2024-01-15T00:00:00Z
```

**Track User Activity**:
```bash
GET /api/v1/audit?user_id=<uuid>&start_time=2024-01-15T00:00:00Z
```

---

## Retention Policy

**Default Retention**: 90 days

**Configuration**:
```bash
PANGOLIN_AUDIT_RETENTION_DAYS=90
```

**Archival**: Audit logs older than retention period are automatically archived or deleted based on configuration.

---

## Integration Examples

### Export to SIEM

```python
import requests
import json

# Fetch audit logs
response = requests.get(
    "http://localhost:8080/api/v1/audit",
    headers={
        "Authorization": f"Bearer {token}",
        "X-Pangolin-Tenant": tenant_id
    },
    params={
        "start_time": "2024-01-01T00:00:00Z",
        "end_time": "2024-01-31T23:59:59Z"
    }
)

logs = response.json()

# Send to SIEM (example: Splunk HEC)
for log in logs:
    requests.post(
        "https://splunk:8088/services/collector",
        headers={"Authorization": "Splunk <token>"},
        json={"event": log}
    )
```

### Compliance Report

```python
import pandas as pd

# Fetch audit logs
response = requests.get(
    "http://localhost:8080/api/v1/audit",
    headers={
        "Authorization": f"Bearer {token}",
        "X-Pangolin-Tenant": tenant_id
    },
    params={"start_time": "2024-01-01", "end_time": "2024-01-31"}
)

logs = response.json()

# Convert to DataFrame
df = pd.DataFrame(logs)

# Generate report
report = df.groupby(['action', 'username']).size().reset_index(name='count')
report.to_csv('audit_report_january_2024.csv', index=False)
```

---

## Best Practices

### 1. **Regular Review**
Review audit logs weekly for:
- Unusual access patterns
- Failed authentication attempts
- Unexpected data modifications

### 2. **Automated Alerts**
Set up alerts for critical events:
- Failed login attempts (potential brute force)
- Table drops in production
- Permission changes

### 3. **Retention Compliance**
Configure retention based on compliance requirements:
- GDPR: Minimum 6 months
- HIPAA: Minimum 6 years
- SOX: Minimum 7 years

### 4. **Export for Long-Term Storage**
Regularly export audit logs to:
- S3 for archival
- Data warehouse for analysis
- SIEM for security monitoring

### 5. **Protect Audit Logs**
- Restrict access to audit logs (TenantAdmin only)
- Ensure audit logs are immutable
- Encrypt audit logs at rest and in transit

---

## Troubleshooting

### Audit logs not appearing

**Cause**: Audit logging disabled or storage issue.

**Solution**:
1. Verify `PANGOLIN_ENABLE_AUDIT_LOGS=true`
2. Check storage backend is accessible
3. Review server logs for errors

### Missing user information

**Cause**: Service user or unauthenticated request.

**Solution**: Service users are logged with their service user ID and name.

### High volume of logs

**Cause**: High-frequency operations (e.g., bulk data loads).

**Solution**:
1. Filter by specific actions or resources
2. Use time range filters
3. Consider sampling for analysis

---

## Related Documentation

- [RBAC](./rbac.md) - Role-based access control
- [Service Users](../service_users.md) - Service user management
- [Authentication](../authentication.md) - User authentication.
