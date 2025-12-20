# Service User CLI Commands

Complete reference for managing service users via the Pangolin Admin CLI.

## Overview

Service users provide API key-based authentication for programmatic access to Pangolin. They are ideal for:
- CI/CD pipelines
- Data integration tools
- Monitoring systems
- Microservices

## Commands

### create-service-user

Create a new service user with an API key.

**Syntax**:
```bash
pangolin-admin create-service-user \
  --name <name> \
  [--description <description>] \
  [--role <role>] \
  [--expires-in-days <days>]
```

**Arguments**:
- `--name` (required): Name of the service user
- `--description` (optional): Description of purpose
- `--role` (optional): Role assignment (default: `tenant-user`)
  - Valid values: `tenant-user`, `tenant-admin`, `root`
- `--expires-in-days` (optional): Expiration in days from creation

**Example**:
```bash
pangolin-admin create-service-user \
  --name "ci-pipeline" \
  --description "GitHub Actions CI/CD" \
  --role "tenant-user" \
  --expires-in-days 90
```

**Output**:
```
✅ Service user created successfully!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  IMPORTANT: Save this API key - it will not be shown again!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Service User ID: 550e8400-e29b-41d4-a716-446655440000
Name: ci-pipeline
API Key: pgl_AbCdEfGhIjKlMnOpQrStUvWxYz1234567890...
Expires At: 2026-03-18T19:00:00Z
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

### list-service-users

List all service users for the current tenant.

**Syntax**:
```bash
pangolin-admin list-service-users
```

**Example Output**:
```
+--------------------------------------+---------------+-------------+--------+------------+------------+
| ID                                   | Name          | Role        | Active | Last Used  | Expires At |
+--------------------------------------+---------------+-------------+--------+------------+------------+
| 550e8400-e29b-41d4-a716-446655440000 | ci-pipeline   | tenant-user | ✓      | 2 days ago | Never      |
| 660e8400-e29b-41d4-a716-446655440001 | data-sync     | tenant-user | ✓      | Never      | 2026-06-01 |
+--------------------------------------+---------------+-------------+--------+------------+------------+
```

---

### get-service-user

Get detailed information about a specific service user.

**Syntax**:
```bash
pangolin-admin get-service-user --id <service-user-id>
```

**Example**:
```bash
pangolin-admin get-service-user --id 550e8400-e29b-41d4-a716-446655440000
```

**Output**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Service User Details
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ID: 550e8400-e29b-41d4-a716-446655440000
Name: ci-pipeline
Description: GitHub Actions CI/CD
Role: tenant-user
Active: Yes
Created At: 2025-12-18T14:00:00Z
Last Used: 2025-12-16T10:30:00Z
Expires At: 2026-03-18T19:00:00Z
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

### update-service-user

Update service user properties.

**Syntax**:
```bash
pangolin-admin update-service-user \
  --id <service-user-id> \
  [--name <new-name>] \
  [--description <new-description>] \
  [--active <true|false>]
```

**Arguments**:
- `--id` (required): Service user ID
- `--name` (optional): New name
- `--description` (optional): New description
- `--active` (optional): Enable/disable service user

**Example - Update description**:
```bash
pangolin-admin update-service-user \
  --id 550e8400-e29b-41d4-a716-446655440000 \
  --description "Updated: GitHub Actions + GitLab CI"
```

**Example - Deactivate service user**:
```bash
pangolin-admin update-service-user \
  --id 550e8400-e29b-41d4-a716-446655440000 \
  --active false
```

---

### delete-service-user

Permanently delete a service user.

**Syntax**:
```bash
pangolin-admin delete-service-user --id <service-user-id>
```

**Example**:
```bash
pangolin-admin delete-service-user --id 550e8400-e29b-41d4-a716-446655440000
```

**Output**:
```
✅ Service user deleted successfully!
```

**Warning**: This action is permanent and immediately invalidates the API key.

---

### rotate-service-user-key

Rotate the API key for a service user.

**Syntax**:
```bash
pangolin-admin rotate-service-user-key --id <service-user-id>
```

**Example**:
```bash
pangolin-admin rotate-service-user-key --id 550e8400-e29b-41d4-a716-446655440000
```

**Output**:
```
✅ API key rotated successfully!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  IMPORTANT: Save this new API key - it will not be shown again!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Service User ID: 550e8400-e29b-41d4-a716-446655440000
Name: ci-pipeline
New API Key: pgl_NewKeyXyZ123456789...
Expires At: 2026-03-18T19:00:00Z
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Important**: The old API key is immediately invalidated.

---

## Best Practices

### Security

1. **Save API Keys Securely**
   - Store in secrets manager (AWS Secrets Manager, HashiCorp Vault, etc.)
   - Never commit to version control
   - Never log in plaintext

2. **Use Least Privilege**
   - Assign minimum required role
   - Use `tenant-user` for most cases
   - Reserve `tenant-admin` for administrative tasks

3. **Set Expiration**
   - Use `--expires-in-days` for temporary access
   - Recommended: 90 days for production, 30 days for testing

4. **Regular Rotation**
   - Rotate keys every 90 days
   - Rotate immediately if compromised
   - Update services with new key before old expires

### Monitoring

1. **Check Last Used**
   - Review `list-service-users` output
   - Identify unused service users
   - Delete inactive service users

2. **Audit Activity**
   - Monitor API access logs
   - Track service user operations
   - Alert on suspicious patterns

### Naming Conventions

Use descriptive names that indicate:
- **Purpose**: `ci-pipeline`, `data-sync`, `monitoring`
- **Environment**: `prod-ci`, `staging-etl`
- **Team**: `analytics-team-bot`, `ml-pipeline`

---

## Troubleshooting

### "Invalid or expired API key"

**Cause**: API key is incorrect, expired, or service user is inactive

**Solutions**:
1. Verify API key is correct (check for typos)
2. Check service user is active: `get-service-user --id <id>`
3. Check expiration date hasn't passed
4. Rotate key if needed: `rotate-service-user-key --id <id>`

### "Only admins can create service users"

**Cause**: Insufficient permissions

**Solutions**:
1. Ensure you're logged in as Tenant Admin or Root
2. Check your JWT token is valid
3. Re-login if token expired

### "Unknown variant TenantUser"

**Cause**: Role format is incorrect or case-sensitive.

**Solution**: Use the exact case expected by the API (e.g., `TenantUser`, `TenantAdmin`, `Root`).
- ✅ `--role "TenantUser"`
- ❌ `--role "tenant-user"`

---

## Related Documentation

- [Service Users API](../service_users.md)
- [Authentication](../authentication.md)
- [RBAC](../features/rbac.md)
- [CLI Overview](./admin.md)
