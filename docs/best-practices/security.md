# Security Best Practices

## Overview

This guide covers security best practices for Pangolin deployments, from authentication to data protection.

## Authentication & Authorization

### JWT Token Security

**Strong Secrets**
```bash
# Generate cryptographically secure secret (256-bit)
openssl rand -base64 32

# Store in secrets manager, never in code
export PANGOLIN_JWT_SECRET=$(aws secretsmanager get-secret-value ...)
```

**Token Expiration**
```bash
# Set appropriate expiration (24 hours recommended)
PANGOLIN_JWT_EXPIRATION_HOURS=24

# For service users, use longer expiration with rotation
SERVICE_TOKEN_EXPIRATION_HOURS=720  # 30 days
```

**Token Rotation**
- Rotate JWT secrets quarterly
- Implement graceful rotation (support old + new for overlap period)
- Force re-authentication after secret rotation

### Multi-Factor Authentication

**For Admin Users**
- Require MFA for root and tenant-admin roles
- Use time-based one-time passwords (TOTP)
- Implement backup codes for account recovery

**Implementation via OAuth**
```bash
# Configure OAuth with MFA-enabled provider
PANGOLIN_OAUTH_PROVIDER=okta
PANGOLIN_OAUTH_REQUIRE_MFA=true
```

## Role-Based Access Control (RBAC)

### Principle of Least Privilege

**User Roles Hierarchy**
```
Root (System Admin)
  ↓
Tenant Admin
  ↓
Tenant User
  ↓
Service User (API-only)
```

**Permission Assignment**
```bash
# Grant minimum necessary permissions
pangolin-admin grant-permission \
  --user data-analyst \
  --catalog analytics \
  --namespace sales \
  --role READ

# Avoid wildcard grants in production
❌ --namespace "*"  # Too broad
✅ --namespace "sales"  # Specific
```

### Service Users

**API Key Management**
```bash
# Create service user with specific permissions
pangolin-admin create-service-user \
  --name etl-pipeline \
  --description "Nightly ETL job"

# Rotate keys regularly (90 days)
pangolin-admin rotate-service-user-key --id <uuid>

# Revoke compromised keys immediately
pangolin-admin delete-service-user --id <uuid>
```

**Best Practices**
- One service user per application/pipeline
- Document service user purpose
- Monitor service user activity
- Implement key rotation automation

## Network Security

### TLS/HTTPS

**Enforce HTTPS**
```nginx
# Nginx configuration
server {
    listen 80;
    server_name pangolin.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name pangolin.example.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    location / {
        proxy_pass http://pangolin-api:8080;
    }
}
```

### Firewall Rules

**API Server**
```bash
# Allow only necessary ports
- 443 (HTTPS) from load balancer
- 8080 (API) from internal network only
- Block all other inbound traffic
```

**Database**
```bash
# Restrict to API servers only
- 5432 (PostgreSQL) from API server IPs only
- 27017 (MongoDB) from API server IPs only
- Block public internet access
```

### VPC/Network Isolation

```
┌─────────────────────────────────┐
│  Public Subnet                  │
│  ┌──────────────┐               │
│  │ Load Balancer│               │
│  └──────┬───────┘               │
└─────────┼─────────────────────────┘
          │
┌─────────┼─────────────────────────┐
│  Private Subnet                  │
│  ┌──────▼───────┐                │
│  │  API Servers │                │
│  └──────┬───────┘                │
│         │                        │
│  ┌──────▼───────┐                │
│  │  PostgreSQL  │                │
│  └──────────────┘                │
└──────────────────────────────────┘
```

## Data Protection

### Encryption at Rest

**Database Encryption**
```bash
# PostgreSQL
# Enable transparent data encryption (TDE)
# Use encrypted EBS volumes for RDS

# MongoDB
# Enable encryption at rest
mongod --enableEncryption --encryptionKeyFile /path/to/keyfile
```

**Object Storage Encryption**
```bash
# S3 Server-Side Encryption
aws s3api put-bucket-encryption \
  --bucket my-pangolin-bucket \
  --server-side-encryption-configuration '{
    "Rules": [{
      "ApplyServerSideEncryptionByDefault": {
        "SSEAlgorithm": "AES256"
      }
    }]
  }'

# Or use KMS for key management
"SSEAlgorithm": "aws:kms",
"KMSMasterKeyID": "arn:aws:kms:..."
```

### Encryption in Transit

**Database Connections**
```bash
# PostgreSQL with SSL
DATABASE_URL=postgresql://user:pass@host:5432/pangolin?sslmode=require

# MongoDB with TLS
mongodb://user:pass@host:27017/pangolin?tls=true
```

**Object Storage**
- Always use HTTPS endpoints
- Verify SSL certificates
- Use signed URLs for temporary access

## Credential Management

### Secrets Storage

**DO NOT**
- ❌ Store secrets in environment files
- ❌ Commit secrets to version control
- ❌ Log secrets in application logs
- ❌ Pass secrets via command-line arguments

**DO**
- ✅ Use secrets manager (AWS Secrets Manager, Vault, etc.)
- ✅ Rotate secrets regularly
- ✅ Audit secret access
- ✅ Use IAM roles when possible

### Credential Vending

**S3 Temporary Credentials**
```python
# Vend short-lived STS credentials
credentials = sts.assume_role(
    RoleArn='arn:aws:iam::account:role/pangolin-data-access',
    DurationSeconds=3600  # 1 hour
)

# Return to client for direct S3 access
return {
    'access_key_id': credentials['AccessKeyId'],
    'secret_access_key': credentials['SecretAccessKey'],
    'session_token': credentials['SessionToken'],
    'expiration': credentials['Expiration']
}
```

**Best Practices**
- Minimum credential lifetime (1-12 hours)
- Scope credentials to specific resources
- Monitor credential usage
- Revoke on suspicious activity

## Audit Logging

### Comprehensive Logging

**Log All Security Events**
```json
{
  "event": "authentication_failure",
  "timestamp": "2025-12-24T20:00:00Z",
  "user": "john.doe",
  "ip_address": "192.168.1.100",
  "reason": "invalid_password",
  "attempts": 3
}
```

**Critical Events to Log**
- Authentication (success/failure)
- Authorization decisions
- Permission changes
- User creation/deletion
- Service user key rotation
- Data access (read/write)
- Configuration changes

### Log Retention

**Compliance Requirements**
- Security logs: 1-7 years (depending on industry)
- Audit logs: 90 days minimum
- Access logs: 30 days minimum

**Storage**
- Use immutable storage (S3 Object Lock, WORM)
- Encrypt logs at rest
- Implement log integrity verification

### Monitoring & Alerting

**Security Alerts**
```yaml
alerts:
  - name: Multiple Failed Logins
    condition: failed_login_count > 5 in 5 minutes
    action: notify_security_team
    
  - name: Privilege Escalation
    condition: role_change to tenant-admin or root
    action: notify_security_team
    
  - name: Unusual Access Pattern
    condition: access_from_new_country
    action: require_mfa_verification
    
  - name: Service User Key Rotation Overdue
    condition: key_age > 90 days
    action: notify_admin
```

## Vulnerability Management

### Dependency Scanning

```bash
# Scan Rust dependencies
cargo audit

# Scan Docker images
docker scan your-registry/pangolin-api:latest

# Scan infrastructure as code
checkov -d ./terraform
```

### Patch Management

**Update Schedule**
- Critical security patches: Within 24 hours
- High severity: Within 7 days
- Medium severity: Within 30 days
- Low severity: Next maintenance window

**Testing Process**
1. Test in development environment
2. Deploy to staging
3. Run security tests
4. Deploy to production with rollback plan

## Compliance

### Data Residency

**Geographic Restrictions**
```bash
# Deploy in specific regions
AWS_REGION=eu-west-1  # EU data stays in EU
AZURE_LOCATION=westeurope

# Configure cross-region replication carefully
# Ensure compliance with GDPR, CCPA, etc.
```

### Data Classification

**Sensitivity Levels**
```python
# Tag assets with sensitivity
{
  "table": "customers.personal_data",
  "classification": "PII",
  "retention_policy": "7_years",
  "encryption_required": true,
  "access_logging": "mandatory"
}
```

### Access Reviews

**Quarterly Reviews**
```bash
# Generate access report
pangolin-admin list-permissions --format csv > access_review_Q4_2025.csv

# Review and revoke unnecessary permissions
pangolin-admin revoke-permission --user inactive-user --catalog analytics
```

## Incident Response

### Security Incident Playbook

**1. Detection**
- Monitor alerts
- Review audit logs
- User reports

**2. Containment**
```bash
# Immediately revoke compromised credentials
pangolin-admin revoke-token-by-id --id <token-id>

# Disable compromised user
pangolin-admin update-user --id <user-id> --active false

# Rotate affected secrets
pangolin-admin rotate-service-user-key --id <service-user-id>
```

**3. Investigation**
```bash
# Review audit logs
pangolin-admin list-audit-events \
  --user <compromised-user> \
  --start-date 2025-12-01 \
  --end-date 2025-12-24

# Check for unauthorized access
pangolin-admin list-audit-events \
  --action "READ" \
  --resource-type "TABLE" \
  --user <compromised-user>
```

**4. Recovery**
- Restore from clean backup if needed
- Reset all affected credentials
- Apply security patches

**5. Post-Incident**
- Document incident
- Update security procedures
- Conduct team training

## Security Checklist

### Deployment
- [ ] HTTPS/TLS enabled
- [ ] Strong JWT secrets configured
- [ ] Secrets stored in secrets manager
- [ ] Database encryption enabled
- [ ] Firewall rules configured
- [ ] VPC/network isolation implemented

### Operations
- [ ] Audit logging enabled
- [ ] Security monitoring configured
- [ ] Alerts set up
- [ ] Backup encryption enabled
- [ ] Incident response plan documented

### Maintenance
- [ ] Quarterly access reviews
- [ ] Regular security patches
- [ ] Dependency scanning
- [ ] Penetration testing (annual)
- [ ] Secret rotation (quarterly)

## Additional Resources

- [Permissions Management Best Practices](./permissions.md)
- [Audit Logging Guide](../features/audit_logs.md)
- [Deployment Security](./deployment.md)
