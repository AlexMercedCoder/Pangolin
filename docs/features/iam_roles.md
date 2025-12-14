# Using IAM Roles with Pangolin Warehouses

## Overview

Pangolin supports role-based authentication for cloud storage, allowing you to avoid storing static credentials. Instead, Pangolin can assume IAM roles to vend temporary credentials to clients.

This document explains how to configure warehouses to use IAM roles for AWS S3, Azure ADLS Gen2, and Google Cloud Storage.

## Benefits of Role-Based Authentication

✅ **Enhanced Security**: No static credentials stored in Pangolin  
✅ **Automatic Rotation**: Temporary credentials expire automatically  
✅ **Fine-Grained Access**: Use cloud IAM policies for precise control  
✅ **Audit Trail**: Cloud provider logs all access via the role  
✅ **Compliance**: Meets security requirements for credential management  

---

## AWS S3 with IAM Roles

### Prerequisites

1. **IAM Role**: Create an IAM role that Pangolin can assume
2. **Trust Policy**: Configure the role to trust the Pangolin service account/instance
3. **Permissions**: Attach policies granting S3 access to the role

### Step 1: Create IAM Role

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::YOUR_ACCOUNT_ID:role/PangolinServiceRole"
      },
      "Action": "sts:AssumeRole",
      "Condition": {}
    }
  ]
}
```

### Step 2: Attach S3 Permissions

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
        "arn:aws:s3:::your-data-bucket/*",
        "arn:aws:s3:::your-data-bucket"
      ]
    }
  ]
}
```

### Step 3: Create Warehouse with STS

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "s3-iam-warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "s3",
      "bucket": "your-data-bucket",
      "region": "us-east-1",
      "role_arn": "arn:aws:iam::123456789012:role/PangolinDataAccessRole"
    }
  }'
```

### Configuration Options

| Field | Required | Description |
|-------|----------|-------------|
| `use_sts` | Yes | Set to `true` to enable STS credential vending |
| `storage_config.type` | Yes | Must be `"s3"` |
| `storage_config.bucket` | Yes | S3 bucket name |
| `storage_config.region` | Yes | AWS region (e.g., `us-east-1`) |
| `storage_config.role_arn` | Yes | ARN of the IAM role to assume |
| `storage_config.endpoint` | No | Custom S3 endpoint (for MinIO, etc.) |
| `storage_config.external_id` | No | External ID for AssumeRole (optional security measure) |
| `storage_config.session_duration` | No | Session duration in seconds (default: 3600) |

### How It Works

1. Client requests credentials via `/api/v1/catalogs/{catalog}/namespaces/{ns}/tables/{table}/credentials`
2. Pangolin calls AWS STS `AssumeRole` with the configured `role_arn`
3. AWS returns temporary credentials (access key, secret key, session token)
4. Pangolin vends these credentials to the client
5. Client uses temporary credentials to access S3
6. Credentials expire after the session duration (default 1 hour)

---

## Azure ADLS Gen2 with Managed Identity / Service Principal

### Prerequisites

1. **Managed Identity** or **Service Principal**: Create in Azure AD
2. **RBAC Assignment**: Grant "Storage Blob Data Contributor" role to the identity
3. **Pangolin Configuration**: Configure Pangolin to use the identity

### Step 1: Create Managed Identity

```bash
# Create managed identity
az identity create \
  --name pangolin-data-access \
  --resource-group your-resource-group

# Get the identity's client ID
az identity show \
  --name pangolin-data-access \
  --resource-group your-resource-group \
  --query clientId -o tsv
```

### Step 2: Grant Storage Access

```bash
# Assign Storage Blob Data Contributor role
az role assignment create \
  --assignee <MANAGED_IDENTITY_CLIENT_ID> \
  --role "Storage Blob Data Contributor" \
  --scope /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/<RESOURCE_GROUP>/providers/Microsoft.Storage/storageAccounts/<STORAGE_ACCOUNT>
```

### Step 3: Create Warehouse with OAuth2

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "azure-iam-warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "azure",
      "container": "your-container",
      "account_name": "yourstorageaccount",
      "tenant_id": "your-azure-tenant-id",
      "client_id": "your-managed-identity-client-id",
      "client_secret": "your-service-principal-secret"
    }
  }'
```

### Configuration Options

| Field | Required | Description |
|-------|----------|-------------|
| `use_sts` | Yes | Set to `true` to enable OAuth2 token vending |
| `storage_config.type` | Yes | Must be `"azure"` |
| `storage_config.container` | Yes | Azure container name |
| `storage_config.account_name` | Yes | Storage account name |
| `storage_config.tenant_id` | Yes | Azure AD tenant ID |
| `storage_config.client_id` | Yes | Managed identity or service principal client ID |
| `storage_config.client_secret` | No | Service principal secret (not needed for managed identity) |
| `storage_config.endpoint` | No | Custom endpoint (for Azurite emulator) |

### How It Works

1. Client requests credentials
2. Pangolin requests OAuth2 token from Azure AD using the configured identity
3. Azure AD returns an access token
4. Pangolin vends the token to the client
5. Client uses the token to access ADLS Gen2
6. Token expires after ~1 hour (Azure default)

---

## Google Cloud Storage with Service Account Impersonation

### Prerequisites

1. **Service Account**: Create a service account for data access
2. **IAM Bindings**: Grant storage permissions to the service account
3. **Impersonation**: Allow Pangolin's service account to impersonate the data access account

### Step 1: Create Service Account

```bash
# Create service account
gcloud iam service-accounts create pangolin-data-access \
  --display-name="Pangolin Data Access"

# Grant storage permissions
gcloud projects add-iam-policy-binding YOUR_PROJECT_ID \
  --member="serviceAccount:pangolin-data-access@YOUR_PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/storage.objectAdmin"
```

### Step 2: Enable Impersonation

```bash
# Allow Pangolin's service account to impersonate the data access account
gcloud iam service-accounts add-iam-policy-binding \
  pangolin-data-access@YOUR_PROJECT_ID.iam.gserviceaccount.com \
  --member="serviceAccount:pangolin-service@YOUR_PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountTokenCreator"
```

### Step 3: Create Warehouse with OAuth2

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gcs-iam-warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "gcs",
      "bucket": "your-data-bucket",
      "project_id": "your-project-id",
      "service_account_email": "pangolin-data-access@your-project-id.iam.gserviceaccount.com"
    }
  }'
```

### Configuration Options

| Field | Required | Description |
|-------|----------|-------------|
| `use_sts` | Yes | Set to `true` to enable OAuth2 token vending |
| `storage_config.type` | Yes | Must be `"gcs"` |
| `storage_config.bucket` | Yes | GCS bucket name |
| `storage_config.project_id` | Yes | GCP project ID |
| `storage_config.service_account_email` | Yes | Service account to impersonate |
| `storage_config.endpoint` | No | Custom endpoint (for GCS emulator) |

### How It Works

1. Client requests credentials
2. Pangolin calls GCP IAM `generateAccessToken` to impersonate the service account
3. GCP returns an OAuth2 access token
4. Pangolin vends the token to the client
5. Client uses the token to access GCS
6. Token expires after ~1 hour

---

## Comparison: Static Credentials vs IAM Roles

| Aspect | Static Credentials | IAM Roles |
|--------|-------------------|-----------|
| **Security** | ⚠️ Long-lived credentials | ✅ Temporary credentials |
| **Rotation** | ❌ Manual rotation required | ✅ Automatic expiration |
| **Audit** | ⚠️ Limited traceability | ✅ Full cloud provider audit logs |
| **Setup** | ✅ Simple | ⚠️ Requires IAM configuration |
| **Complexity** | ✅ Low | ⚠️ Medium |
| **Best For** | Development, testing | Production, compliance |

---

## Current Implementation Status

> **Note**: As of the current version, Pangolin has placeholders for STS/OAuth2 token generation. The infrastructure is in place, but actual token generation requires:
>
> 1. **AWS**: Integration with AWS SDK for `sts:AssumeRole`
> 2. **Azure**: Integration with Azure SDK for OAuth2 token acquisition
> 3. **GCS**: Integration with GCP SDK for service account impersonation
>
> **Workaround**: For production use, you can:
> - Use static credentials with `use_sts: false`
> - Implement a custom credential vending service
> - Contribute STS implementation to Pangolin!

### Implementation Complexity

Adding full STS support is **moderately complex** and requires:

1. **Dependencies** (~1-2 hours):
   - Add AWS SDK (`aws-sdk-sts`)
   - Add Azure SDK (`azure_identity`, `azure_storage_blobs`)
   - Add GCP SDK (`google-cloud-storage`, `google-auth`)

2. **Code Changes** (~4-6 hours):
   - Implement `assume_role()` for AWS
   - Implement `get_oauth_token()` for Azure
   - Implement `impersonate_service_account()` for GCP
   - Add error handling and retries
   - Add credential caching (optional but recommended)

3. **Testing** (~2-3 hours):
   - Unit tests for each cloud provider
   - Integration tests with real cloud accounts
   - Test credential expiration and renewal

**Total Estimate**: 7-11 hours for full implementation

---

## Examples

### PyIceberg with IAM Role Warehouse

```python
from pyiceberg.catalog import load_catalog

# Catalog configured with IAM role warehouse
catalog = load_catalog(
    "pangolin",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/api/v1",
        "credential": "user:password",  # Pangolin auth
        # No storage credentials needed - vended by Pangolin!
    }
)

# Credentials are automatically vended when accessing tables
table = catalog.load_table("my_catalog.my_namespace.my_table")
df = table.scan().to_pandas()  # Uses temporary credentials from IAM role
```

### Spark with IAM Role Warehouse

```scala
spark.conf.set("spark.sql.catalog.pangolin", "org.apache.iceberg.spark.SparkCatalog")
spark.conf.set("spark.sql.catalog.pangolin.catalog-impl", "org.apache.iceberg.rest.RESTCatalog")
spark.conf.set("spark.sql.catalog.pangolin.uri", "http://localhost:8080/api/v1")
spark.conf.set("spark.sql.catalog.pangolin.credential", "user:password")
// No S3 credentials needed - vended by Pangolin!

val df = spark.table("pangolin.my_catalog.my_namespace.my_table")
df.show()
```

---

## Security Best Practices

1. **Principle of Least Privilege**: Grant only necessary permissions to IAM roles
2. **Separate Roles**: Use different roles for different catalogs/tenants
3. **External IDs**: Use external IDs for AssumeRole to prevent confused deputy attacks
4. **Short Sessions**: Keep session durations short (1 hour or less)
5. **Monitor Access**: Enable CloudTrail/Azure Monitor/GCP Audit Logs
6. **Rotate Regularly**: Even with temporary credentials, rotate the base service account credentials

---

## Troubleshooting

### "AssumeRole failed: Access Denied"
- Check the trust policy on the IAM role
- Verify Pangolin's service account has `sts:AssumeRole` permission
- Check for any SCPs (Service Control Policies) blocking the action

### "OAuth2 token acquisition failed"
- Verify the managed identity/service principal has the correct permissions
- Check that the tenant ID and client ID are correct
- Ensure the service principal secret hasn't expired (if using one)

### "Service account impersonation failed"
- Verify the service account exists
- Check that Pangolin's service account has `iam.serviceAccountTokenCreator` role
- Ensure the service account has storage permissions

---

## Related Documentation

- [Warehouse Management](warehouse_management.md)
- [S3 Storage Configuration](../storage/storage_s3.md)
- [Azure Storage Configuration](../storage/storage_azure.md)
- [GCS Storage Configuration](../storage/storage_gcs.md)
- [Security & Credential Vending](security_vending.md)
