# Security & Credential Vending (Multi-Cloud)

Pangolin provides mechanisms to securely vend temporary credentials to clients for S3, Azure ADLS Gen2, and Google Cloud Storage, enabling direct data access while maintaining security.

## Overview

Instead of sharing long-term cloud credentials with clients (e.g., Spark jobs, Dremio, Trino), Pangolin acts as a trusted intermediary. It authenticates the client and then issues temporary, scoped credentials for specific storage resources.

**Benefits**:
- ✅ No long-term credentials in client configurations
- ✅ Automatic credential rotation (STS for S3, OAuth2 for Azure/GCP)
- ✅ Scoped access to specific table locations
- ✅ Centralized audit trail of data access
- ✅ Support for cross-account/cross-cloud access
- ✅ **Multi-cloud support:** S3, Azure ADLS Gen2, Google Cloud Storage

---

## Configuration

### Prerequisites

1. **AWS Credentials**: Pangolin needs AWS credentials with permissions to:
   - Call `sts:AssumeRole` (for STS vending)
   - Access S3 buckets (for static credential vending)
   - Generate presigned URLs

2. **IAM Role** (for STS vending): Create an IAM role that Pangolin can assume with S3 access permissions.

### Environment Variables

```bash
# AWS Configuration
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=...

# For STS Credential Vending
PANGOLIN_STS_ROLE_ARN=arn:aws:iam::123456789012:role/PangolinDataAccess
PANGOLIN_STS_SESSION_DURATION=3600  # 1 hour (default)

# For MinIO or S3-compatible storage
AWS_ENDPOINT_URL=http://minio:9000
AWS_ALLOW_HTTP=true
```

### Warehouse Configuration

When creating a warehouse, set `use_sts` to enable credential vending:

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production_warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "s3",
      "bucket": "my-data-bucket",
      "region": "us-east-1",
      "role_arn": "arn:aws:iam::123456789012:role/PangolinDataAccess"
    }
  }'
```

**Configuration Options**:
- `use_sts: true` - Vend temporary STS credentials to clients
- `use_sts: false` - Pass through static credentials from environment variables

### Azure ADLS Gen2 Configuration

**OAuth2 Mode (Recommended):**
```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure_warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "azure",
      "account_name": "mystorageaccount",
      "container": "data",
      "tenant_id": "azure-tenant-id",
      "client_id": "azure-client-id",
      "client_secret": "azure-client-secret"
    }
  }'
```

**Account Key Mode:**
```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure_warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "azure",
      "account_name": "mystorageaccount",
      "container": "data",
      "account_key": "your-account-key"
    }
  }'
```

### Google Cloud Storage Configuration

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gcp_warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "gcs",
      "project_id": "my-gcp-project",
      "bucket": "my-data-bucket",
      "service_account_key": "{...json key...}"
    }
  }'
```

---

## Features

### 1. Table Credential Vending

Get temporary AWS credentials (Access Key, Secret Key, Session Token) scoped to a specific table's location.

**Endpoint:** `POST /v1/{prefix}/namespaces/{namespace}/tables/{table}/credentials`

**Request:**
```bash
curl -X POST http://localhost:8080/v1/analytics/namespaces/sales/tables/transactions/credentials \
  -H "Authorization: Bearer <token>" \
  -H "X-Pangolin-Tenant: <tenant-id>"
```

**Response:**
```json
{
  "access_key_id": "ASIA...",
  "secret_access_key": "...",
  "session_token": "...",
  "expiration": "2023-10-27T10:00:00Z"
}
```

**Use Case**: Spark jobs, Trino queries, or any client that needs direct S3 access.

---

### 2. Presigned URLs

Get a presigned URL to download a specific file (e.g., a metadata file or data file) without needing AWS credentials.

**Endpoint:** `GET /v1/{prefix}/namespaces/{namespace}/tables/{table}/presign?location=s3://bucket/key`

**Request:**
```bash
curl "http://localhost:8080/v1/analytics/namespaces/sales/tables/transactions/presign?location=s3://my-bucket/data/file.parquet" \
  -H "Authorization: Bearer <token>" \
  -H "X-Pangolin-Tenant: <tenant-id>"
```

**Response:**
```json
{
  "url": "https://bucket.s3.amazonaws.com/key?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=..."
}
```

**Use Case**: Web applications, data preview tools, or clients that can't handle AWS credentials.

---

## PyIceberg Integration

PyIceberg automatically uses Pangolin's credential vending when configured correctly for all supported cloud providers.

### Automatic Credential Vending (S3)

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "token": "your-jwt-token",
        # No S3 credentials needed - Pangolin vends them automatically!
    }
)

# PyIceberg will request credentials from Pangolin for each table access
table = catalog.load_table("sales.transactions")
df = table.scan().to_pandas()  # Pangolin vends S3 credentials automatically
```

### Automatic Credential Vending (Azure)

```python
catalog = load_catalog(
    "pangolin_azure",
    **{
        "uri": "http://localhost:8080",
        "prefix": "azure_catalog",
        "token": "your-jwt-token",
        # No Azure credentials needed - Pangolin vends them automatically!
        # Pangolin provides: adls.token, adls.account-name, adls.container
    }
)

table = catalog.load_table("sales.transactions")
df = table.scan().to_pandas()  # Pangolin vends Azure credentials automatically
```

### Automatic Credential Vending (GCP)

```python
catalog = load_catalog(
    "pangolin_gcp",
    **{
        "uri": "http://localhost:8080",
        "prefix": "gcp_catalog",
        "token": "your-jwt-token",
        # No GCP credentials needed - Pangolin vends them automatically!
        # Pangolin provides: gcp-oauth-token, gcp-project-id
    }
)

table = catalog.load_table("sales.transactions")
df = table.scan().to_pandas()  # Pangolin vends GCP credentials automatically
```

**How it works**:
1. PyIceberg requests table metadata from Pangolin
2. Pangolin includes temporary cloud credentials in the response (based on warehouse type)
3. PyIceberg uses these credentials to read data files from cloud storage
4. Credentials expire after the configured duration (default: 1 hour)

### Client-Provided Credentials

If you prefer to manage credentials yourself:

**S3:**
```python
catalog = load_catalog(
    "pangolin",
    **{
        "uri": "http://localhost:8080",
        "prefix": "analytics",
        "token": "your-jwt-token",
        "s3.access-key-id": "AKIA...",
        "s3.secret-access-key": "...",
    }
)
```

**Azure:**
```python
catalog = load_catalog(
    "pangolin_azure",
    **{
        "uri": "http://localhost:8080",
        "prefix": "azure_catalog",
        "token": "your-jwt-token",
        "adls.account-name": "mystorageaccount",
        "adls.account-key": "...",
    }
)
```

**GCP:**
```python
catalog = load_catalog(
    "pangolin_gcp",
    **{
        "uri": "http://localhost:8080",
        "prefix": "gcp_catalog",
        "token": "your-jwt-token",
        "gcp-project-id": "my-project",
        "gcs.service-account-key": "/path/to/key.json",
    }
)
```

---

## IAM Policy Examples

### Pangolin Service Role Policy

This is the policy for the IAM role that Pangolin assumes to vend credentials:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::my-data-bucket/*",
        "arn:aws:s3:::my-data-bucket"
      ]
    }
  ]
}
```

### Trust Policy for Pangolin

Allow Pangolin's AWS account/role to assume the data access role:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::YOUR-PANGOLIN-ACCOUNT:role/PangolinService"
      },
      "Action": "sts:AssumeRole"
    }
  ]
}
```

---

## Security Best Practices

### 1. **Use STS Credential Vending**
Always use `use_sts: true` in production to vend temporary credentials instead of sharing static credentials.

### 2. **Scope Credentials to Table Locations**
Pangolin vends credentials scoped to specific table locations, limiting blast radius if credentials are compromised.

### 3. **Set Short Expiration Times**
Configure `PANGOLIN_STS_SESSION_DURATION` to the minimum time needed (default: 3600 seconds / 1 hour).

### 4. **Use IAM Conditions**
Add IAM policy conditions to restrict access by IP, time, or other factors:

```json
{
  "Condition": {
    "IpAddress": {
      "aws:SourceIp": "10.0.0.0/8"
    }
  }
}
```

### 5. **Monitor Credential Usage**
Review CloudTrail logs for STS AssumeRole calls to detect unusual patterns.

### 6. **Rotate Static Credentials**
If using static credentials (`use_sts: false`), rotate them regularly.

---

## Troubleshooting

### "Access Denied" when reading data

**Cause**: Vended credentials don't have permissions for the S3 location.

**Solution**:
1. Verify the IAM role has S3 permissions for the table location
2. Check the role ARN in warehouse configuration
3. Verify Pangolin can assume the role: `aws sts assume-role --role-arn <arn> --role-session-name test`

### "Credentials expired" errors

**Cause**: STS credentials expired during a long-running query.

**Solution**:
1. Increase `PANGOLIN_STS_SESSION_DURATION` (max: 43200 seconds / 12 hours)
2. Configure your client to refresh credentials automatically
3. For very long queries, consider using static credentials

### PyIceberg not using vended credentials

**Cause**: PyIceberg may be using client-provided credentials instead.

**Solution**:
1. Remove `s3.access-key-id` and `s3.secret-access-key` from PyIceberg config
2. Verify warehouse has `use_sts: true`
3. Check Pangolin logs for credential vending requests

### "Invalid security token" errors

**Cause**: Clock skew between Pangolin server and AWS.

**Solution**:
1. Sync server time with NTP: `sudo ntpdate -s time.nist.gov`
2. Verify server timezone is set correctly
3. Check CloudTrail for timestamp-related errors

### Cross-account access not working

**Cause**: Trust policy or permissions issue.

**Solution**:
1. Verify trust policy allows Pangolin's role to assume the target role
2. Check both the trust policy and the permissions policy
3. Test with AWS CLI: `aws sts assume-role --role-arn <target-role> --role-session-name test`

---

## Advanced Configuration

### Custom Session Duration

```bash
# Set custom session duration (in seconds)
PANGOLIN_STS_SESSION_DURATION=7200  # 2 hours
```

### External ID for Cross-Account Access

For enhanced security in cross-account scenarios:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::PANGOLIN-ACCOUNT:role/PangolinService"
      },
      "Action": "sts:AssumeRole",
      "Condition": {
        "StringEquals": {
          "sts:ExternalId": "unique-external-id-12345"
        }
      }
    }
  ]
}
```

Set in Pangolin:
```bash
PANGOLIN_STS_EXTERNAL_ID=unique-external-id-12345
```

### Regional Endpoints

For better performance, use regional STS endpoints:

```bash
AWS_STS_REGIONAL_ENDPOINTS=regional
```

---

## Related Documentation

- [Warehouse Management](./warehouse_management.md) - Creating and configuring warehouses
- [Authentication](../authentication.md) - User authentication and tokens
- [Client Configuration](../getting-started/client_configuration.md) - PyIceberg, Spark, Trino setup
- [AWS S3 Storage](../warehouse/s3.md) - S3 storage backend configuration
