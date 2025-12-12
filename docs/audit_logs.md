# Audit Logs

Pangolin provides a structured audit logging system to track critical actions performed on the catalog.

## Data Model

An `AuditLogEntry` contains:
- `id`: Unique event ID.
- `tenant_id`: The tenant where the action occurred.
- `timestamp`: UTC timestamp of the event.
- `actor`: The user or system component that performed the action.
- `action`: The type of action (e.g., `create_table`, `merge_branch`).
- `resource`: The resource affected (e.g., `default/namespace/table`).
- `details`: Optional details (e.g., S3 location).

## API Endpoints

### List Audit Events
`GET /api/v1/audit`

**Response:**
```json
[
  {
    "id": "uuid",
    "tenant_id": "uuid",
    "timestamp": "2023-10-27T10:00:00Z",
    "actor": "system",
    "action": "create_table",
    "resource": "default/default/table1",
    "details": "s3://bucket/table1"
  }
]
```

## Storage

- **MemoryStore**: Logs are printed to stdout/tracing logs.
- **S3Store**: Logs are persisted as JSON files in `s3://bucket/tenants/{tenant_id}/audit/`.
- **PostgresStore**: Logs are stored in the `audit_logs` table.
- **MongoStore**: Logs are stored in the `audit_logs` collection.
