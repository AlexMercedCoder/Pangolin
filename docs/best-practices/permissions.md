# Permissions Management Best Practices

## Overview

This guide covers best practices for managing permissions in Pangolin using Role-Based Access Control (RBAC).

## Permission Model

### Roles Hierarchy

```
Root (System Administrator)
  ├── Full system access
  ├── Tenant management
  └── Cross-tenant operations

Tenant Admin
  ├── Full tenant access
  ├── User management
  ├── Warehouse/catalog creation
  └── Permission grants within tenant

Tenant User
  ├── Catalog access (as granted)
  ├── Table operations (as granted)
  └── Self-service discovery

Service User
  ├── API-only access
  ├── Programmatic operations
  └── Limited to granted permissions
```

### Permission Scopes

**Catalog-Level**
```bash
# Grant access to entire catalog
pangolin-admin grant-permission \
  --user analyst@company.com \
  --catalog production \
  --role READ
```

**Namespace-Level**
```bash
# Grant access to specific namespace
pangolin-admin grant-permission \
  --user analyst@company.com \
  --catalog production \
  --namespace sales \
  --role READ_WRITE
```

**Asset-Level** (via tags)
```bash
# Grant access to tagged assets
pangolin-admin grant-permission \
  --user analyst@company.com \
  --catalog production \
  --tag "department:sales" \
  --role READ
```

## Best Practices

### 1. Principle of Least Privilege

**Start Minimal**
```bash
# ❌ Don't grant broad access initially
pangolin-admin grant-permission --user new-user --catalog "*" --role ADMIN

# ✅ Grant specific access as needed
pangolin-admin grant-permission --user new-user --catalog analytics --namespace reports --role READ
```

**Expand Gradually**
- Start with READ access
- Upgrade to READ_WRITE when needed
- ADMIN role only for catalog owners

### 2. Role-Based Assignment

**Define Organizational Roles**
```yaml
# Example role definitions
roles:
  data_analyst:
    catalogs:
      - analytics: READ
      - reports: READ_WRITE
    namespaces:
      - sales.*: READ
      
  data_engineer:
    catalogs:
      - raw_data: READ_WRITE
      - processed: READ_WRITE
    namespaces:
      - etl.*: ADMIN
      
  data_scientist:
    catalogs:
      - ml_features: READ_WRITE
      - experiments: ADMIN
```

**Implement via Scripts**
```bash
#!/bin/bash
# Grant permissions for data analyst role

USERS=("alice@company.com" "bob@company.com")
for user in "${USERS[@]}"; do
  pangolin-admin grant-permission --user "$user" --catalog analytics --role READ
  pangolin-admin grant-permission --user "$user" --catalog reports --role READ_WRITE
done
```

### 3. Tag-Based Access Control

**Organize by Tags**
```python
# Tag assets by department
{
  "table": "sales.customers",
  "tags": {
    "department": "sales",
    "sensitivity": "high",
    "region": "us-east"
  }
}
```

**Grant by Tags**
```bash
# Sales team gets access to sales data
pangolin-admin grant-permission \
  --user sales-team@company.com \
  --catalog production \
  --tag "department:sales" \
  --role READ_WRITE

# Regional access
pangolin-admin grant-permission \
  --user eu-analyst@company.com \
  --catalog production \
  --tag "region:eu" \
  --role READ
```

### 4. Service User Permissions

**Dedicated Service Users**
```bash
# ETL pipeline - write to staging, read from raw
pangolin-admin create-service-user --name etl-pipeline
pangolin-admin grant-permission --service-user etl-pipeline --catalog raw_data --role READ
pangolin-admin grant-permission --service-user etl-pipeline --catalog staging --role READ_WRITE

# BI tool - read-only access
pangolin-admin create-service-user --name tableau-connector
pangolin-admin grant-permission --service-user tableau-connector --catalog analytics --role READ
```

**Document Service Users**
```markdown
# Service User Registry

| Name | Purpose | Permissions | Owner | Last Rotated |
|------|---------|-------------|-------|--------------|
| etl-pipeline | Nightly ETL | raw_data:READ, staging:RW | Data Eng | 2025-12-01 |
| tableau-connector | BI Dashboards | analytics:READ | Analytics | 2025-11-15 |
```

### 5. Temporary Access

**Time-Limited Grants**
```bash
# Grant temporary access for incident investigation
pangolin-admin grant-permission \
  --user incident-responder@company.com \
  --catalog production \
  --role READ \
  --expires-in 24h

# Revoke after investigation
pangolin-admin revoke-permission \
  --user incident-responder@company.com \
  --catalog production
```

### 6. Access Request Workflow

**User Self-Service**
```bash
# User requests access
pangolin-user request-access \
  --catalog analytics \
  --namespace sales \
  --role READ \
  --reason "Q4 sales analysis project"

# Admin reviews and approves
pangolin-admin list-access-requests
pangolin-admin update-access-request --id <uuid> --status approved

# System automatically grants permission
```

## Permission Patterns

### Development → Staging → Production

**Environment Isolation**
```bash
# Developers: Full access to dev, read-only to staging
pangolin-admin grant-permission --user dev-team --catalog dev --role ADMIN
pangolin-admin grant-permission --user dev-team --catalog staging --role READ

# QA: Read-write to staging, read-only to production
pangolin-admin grant-permission --user qa-team --catalog staging --role READ_WRITE
pangolin-admin grant-permission --user qa-team --catalog production --role READ

# Production: Limited access, strict controls
pangolin-admin grant-permission --user prod-operators --catalog production --role READ_WRITE
```

### Data Sensitivity Levels

**Classification-Based Access**
```bash
# Public data - broad access
pangolin-admin grant-permission --tag "sensitivity:public" --role READ

# Internal data - employees only
pangolin-admin grant-permission --tag "sensitivity:internal" --group employees --role READ

# Confidential - specific teams
pangolin-admin grant-permission --tag "sensitivity:confidential" --group finance --role READ

# Restricted - need-to-know basis
pangolin-admin grant-permission --tag "sensitivity:restricted" --user authorized-user --role READ
```

## Monitoring & Auditing

### Regular Access Reviews

**Quarterly Review Process**
```bash
# 1. Export current permissions
pangolin-admin list-permissions --format csv > permissions_Q4_2025.csv

# 2. Review with managers
# 3. Revoke unnecessary access
pangolin-admin revoke-permission --user inactive-user --catalog analytics

# 4. Document review
echo "Q4 2025 access review completed" >> access_review_log.txt
```

### Permission Change Auditing

**Track All Changes**
```sql
-- Query audit logs for permission changes
SELECT 
    timestamp,
    actor,
    action,
    resource,
    details
FROM audit_logs
WHERE action IN ('GRANT_PERMISSION', 'REVOKE_PERMISSION')
ORDER BY timestamp DESC
LIMIT 100;
```

**Alert on Sensitive Changes**
```yaml
alerts:
  - name: Admin Permission Granted
    condition: action = 'GRANT_PERMISSION' AND role = 'ADMIN'
    notify: security-team@company.com
    
  - name: Production Access Granted
    condition: action = 'GRANT_PERMISSION' AND catalog = 'production'
    notify: data-governance@company.com
```

### Unused Permission Detection

**Identify Inactive Access**
```sql
-- Find users with permissions but no recent activity
SELECT 
    p.user_id,
    p.catalog,
    p.role,
    MAX(a.timestamp) as last_access
FROM permissions p
LEFT JOIN audit_logs a ON p.user_id = a.user_id AND p.catalog = a.catalog
GROUP BY p.user_id, p.catalog, p.role
HAVING MAX(a.timestamp) < NOW() - INTERVAL '90 days'
OR MAX(a.timestamp) IS NULL;
```

## Common Scenarios

### Onboarding New User

```bash
#!/bin/bash
# Onboarding script for data analyst

USER_EMAIL=$1
MANAGER_EMAIL=$2

# 1. Create user
pangolin-admin create-user \
  --username "$USER_EMAIL" \
  --email "$USER_EMAIL" \
  --role tenant-user

# 2. Grant standard analyst permissions
pangolin-admin grant-permission --user "$USER_EMAIL" --catalog analytics --role READ
pangolin-admin grant-permission --user "$USER_EMAIL" --catalog sandbox --role READ_WRITE

# 3. Notify manager
echo "User $USER_EMAIL onboarded with standard analyst permissions" | \
  mail -s "New User Onboarded" "$MANAGER_EMAIL"
```

### Offboarding User

```bash
#!/bin/bash
# Offboarding script

USER_EMAIL=$1

# 1. Disable user
pangolin-admin update-user --email "$USER_EMAIL" --active false

# 2. Revoke all tokens
pangolin-admin list-user-tokens --user "$USER_EMAIL" | \
  xargs -I {} pangolin-admin delete-token --id {}

# 3. Document for audit
echo "$(date): User $USER_EMAIL offboarded" >> offboarding_log.txt
```

### Project-Based Access

```bash
# Grant access for project duration
PROJECT_TEAM=("alice@company.com" "bob@company.com" "charlie@company.com")
PROJECT_CATALOG="customer_churn_analysis"

for user in "${PROJECT_TEAM[@]}"; do
  pangolin-admin grant-permission \
    --user "$user" \
    --catalog "$PROJECT_CATALOG" \
    --role READ_WRITE \
    --expires-in 90d
done

# Set reminder to review after 90 days
```

## Troubleshooting

### Permission Denied Issues

**Debug Process**
```bash
# 1. Check user's current permissions
pangolin-admin list-permissions --user user@company.com

# 2. Check catalog/namespace exists
pangolin-admin list-catalogs

# 3. Verify user is active
pangolin-admin list-users --filter "email=user@company.com"

# 4. Check audit logs for recent denials
pangolin-admin list-audit-events --user user@company.com --action ACCESS_DENIED
```

### Over-Permissioned Users

**Identify and Remediate**
```bash
# Find users with ADMIN role
pangolin-admin list-permissions --role ADMIN

# Review necessity
# Downgrade if not needed
pangolin-admin revoke-permission --user user@company.com --catalog analytics --role ADMIN
pangolin-admin grant-permission --user user@company.com --catalog analytics --role READ_WRITE
```

## Compliance

### GDPR/Privacy Considerations

**Data Access Logging**
- Log all access to PII-tagged data
- Retain logs for compliance period
- Provide access reports on request

**Right to Access**
```bash
# Generate user's data access report
pangolin-admin list-audit-events \
  --user user@company.com \
  --action READ \
  --tag "pii:true" \
  --start-date 2025-01-01
```

### SOC 2 / ISO 27001

**Access Control Requirements**
- [ ] Documented permission model
- [ ] Regular access reviews (quarterly)
- [ ] Segregation of duties
- [ ] Audit trail of all changes
- [ ] Automated provisioning/deprovisioning

## Checklist

### Initial Setup
- [ ] Define organizational roles
- [ ] Document permission model
- [ ] Create service users for applications
- [ ] Set up access request workflow
- [ ] Configure audit logging

### Ongoing Operations
- [ ] Quarterly access reviews
- [ ] Monitor permission changes
- [ ] Review service user keys
- [ ] Audit unused permissions
- [ ] Update documentation

### Security
- [ ] Principle of least privilege enforced
- [ ] No wildcard permissions in production
- [ ] Service users have minimal permissions
- [ ] Sensitive data access logged
- [ ] Compliance requirements met

## Additional Resources

- [Security Best Practices](./security.md)
- [RBAC Documentation](../features/rbac.md)
- [Audit Logging](../features/audit_logs.md)
