# CLI Warehouse Management Guide

## Overview

The Pangolin Admin CLI provides comprehensive warehouse management capabilities for S3, Azure ADLS, and Google GCS storage backends.

## Creating Warehouses

### S3/MinIO Warehouses

Create an S3-compatible warehouse (works with AWS S3 and MinIO):

```bash
pangolin-admin create-warehouse my-s3-warehouse \
  --type s3 \
  --bucket my-bucket \
  --access-key AKIAIOSFODNN7EXAMPLE \
  --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
  --region us-east-1 \
  --endpoint http://localhost:9000  # Optional, for MinIO
```

**Interactive Mode** (prompts for credentials):
```bash
pangolin-admin create-warehouse my-s3-warehouse --type s3
```

**Properties Set**:
- `s3.bucket` - S3 bucket name
- `s3.access-key-id` - AWS access key ID
- `s3.secret-access-key` - AWS secret access key
- `s3.region` - AWS region
- `s3.endpoint` - Custom endpoint (for MinIO)

### Azure ADLS Warehouses

Create an Azure Data Lake Storage warehouse:

```bash
pangolin-admin create-warehouse my-azure-warehouse --type azure
```

**Interactive Prompts**:
```
Azure Storage Account Name: mystorageaccount
Azure Storage Account Key: ******
Azure Container Name: mycontainer
Use SAS token instead of account key? [y/N]: n
```

**Properties Set**:
- `adls.account-name` - Azure storage account name
- `adls.account-key` - Azure storage account key
- `azure.container` - Azure container name
- `adls.sas-token` - SAS token (optional, if using SAS auth)

### Google GCS Warehouses

Create a Google Cloud Storage warehouse:

```bash
pangolin-admin create-warehouse my-gcs-warehouse --type gcs
```

**Interactive Prompts**:
```
GCP Project ID: my-gcp-project
GCS Bucket Name: my-gcs-bucket
Service Account JSON File Path: /path/to/service-account.json
```

**Properties Set**:
- `gcs.project-id` - GCP project ID
- `gcs.bucket` - GCS bucket name
- `gcs.service-account-file` - Path to service account JSON file

## Listing Warehouses

View all warehouses in the current tenant:

```bash
pangolin-admin list-warehouses
```

**Output**:
```
+--------------------+------+
| Name               | Type |
+--------------------+------+
| my-s3-warehouse    |      |
| my-azure-warehouse |      |
+--------------------+------+
```

## Updating Warehouses

Update warehouse name and/or storage configuration:

```bash
pangolin-admin update-warehouse --id <warehouse-uuid>
```

**Interactive Flow**:
```
=== Update Warehouse ===
Current name: my-s3-warehouse

What would you like to update?
1. Name only
2. Storage configuration
3. Both
Choice [1]: 2

=== Update Storage Configuration ===
Enter new storage configuration properties (key=value).
Press Enter with empty input to finish.
Example: s3.access-key-id=AKIA...

Property (or press Enter to finish): s3.access-key-id=AKIANEWKEY123
  ✓ Added: s3.access-key-id = AKIANEWKEY123
Property (or press Enter to finish): s3.secret-access-key=newsecret
  ✓ Added: s3.secret-access-key = newsecret
Property (or press Enter to finish): 

✅ Warehouse updated successfully!
```

### Update Name Only

```bash
pangolin-admin update-warehouse --id <uuid> --name new-warehouse-name
```

### Update Storage Configuration

Common use cases:

**Rotate S3 Credentials**:
```
Property: s3.access-key-id=AKIANEWKEY
Property: s3.secret-access-key=newsecretkey
```

**Change Azure Account Key**:
```
Property: adls.account-key=newaccountkey
```

**Update GCS Service Account**:
```
Property: gcs.service-account-file=/new/path/to/sa.json
```

## Deleting Warehouses

Delete a warehouse by name:

```bash
pangolin-admin delete-warehouse my-warehouse-name
```

> **Warning**: Deleting a warehouse does not delete the underlying storage bucket or data. It only removes the warehouse configuration from Pangolin.

## Property Reference

### S3/MinIO Properties

| Property | Required | Description | Example |
|----------|----------|-------------|---------|
| `s3.bucket` | Yes | S3 bucket name | `my-data-bucket` |
| `s3.access-key-id` | Yes | AWS access key ID | `AKIAIOSFODNN7EXAMPLE` |
| `s3.secret-access-key` | Yes | AWS secret access key | `wJalrXUtnFEMI/K7MDENG...` |
| `s3.region` | Yes | AWS region | `us-east-1` |
| `s3.endpoint` | No | Custom S3 endpoint | `http://localhost:9000` |

### Azure ADLS Properties

| Property | Required | Description | Example |
|----------|----------|-------------|---------|
| `adls.account-name` | Yes | Azure storage account name | `mystorageaccount` |
| `adls.account-key` | Yes* | Azure storage account key | `base64encodedkey...` |
| `azure.container` | Yes | Azure container name | `mycontainer` |
| `adls.sas-token` | No | SAS token (alternative to account key) | `?sv=2020-08-04&ss=...` |

\* Required unless using SAS token

### Google GCS Properties

| Property | Required | Description | Example |
|----------|----------|-------------|---------|
| `gcs.project-id` | Yes | GCP project ID | `my-gcp-project` |
| `gcs.bucket` | Yes | GCS bucket name | `my-gcs-bucket` |
| `gcs.service-account-file` | Yes | Path to service account JSON | `/path/to/sa.json` |

## Best Practices

### Security

1. **Never commit credentials** to version control
2. **Use environment variables** for non-interactive scripts:
   ```bash
   export S3_ACCESS_KEY="AKIA..."
   export S3_SECRET_KEY="..."
   pangolin-admin create-warehouse my-warehouse --type s3 \
     --bucket my-bucket \
     --access-key "$S3_ACCESS_KEY" \
     --secret-key "$S3_SECRET_KEY" \
     --region us-east-1
   ```
3. **Rotate credentials regularly** using the update command
4. **Use STS/temporary credentials** when possible (Azure SAS, AWS STS)

### Naming Conventions

- Use descriptive warehouse names: `production-s3-us-east`, `dev-azure-westus`
- Include environment and region in the name
- Use lowercase with hyphens for consistency

### Multi-Cloud Strategy

When using multiple cloud providers:

1. **Create separate warehouses** for each cloud provider
2. **Use consistent naming**: `prod-s3-warehouse`, `prod-azure-warehouse`
3. **Document which catalogs use which warehouses**
4. **Test credential vending** with PyIceberg before production use

## Troubleshooting

### "Root user cannot create warehouses"

**Problem**: Trying to create warehouse as root user  
**Solution**: Login as a Tenant Admin:
```bash
pangolin-admin login --username tenant-admin --password <password>
```

### "Failed to create warehouse: 422 Unprocessable Entity"

**Problem**: Invalid property names or missing required fields  
**Solution**: Ensure you're using correct property names (e.g., `s3.bucket` not `bucket`)

### "Warehouse created but PyIceberg can't connect"

**Problem**: Incorrect credentials or property names  
**Solution**: 
1. Verify credentials are correct
2. Check property names match PyIceberg conventions
3. Update warehouse with correct properties:
   ```bash
   pangolin-admin update-warehouse --id <uuid>
   ```

## Examples

### Complete S3 Warehouse Setup

```bash
# 1. Login as tenant admin
pangolin-admin login --username admin --password mypassword

# 2. Create S3 warehouse
pangolin-admin create-warehouse production-s3 \
  --type s3 \
  --bucket prod-data-bucket \
  --access-key AKIAIOSFODNN7EXAMPLE \
  --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
  --region us-east-1

# 3. Create catalog using the warehouse
pangolin-admin create-catalog prod-catalog --warehouse production-s3

# 4. Verify
pangolin-admin list-warehouses
pangolin-admin list-catalogs
```

### Rotate S3 Credentials

```bash
# Get warehouse ID
pangolin-admin list-warehouses

# Update credentials
pangolin-admin update-warehouse --id <warehouse-uuid>
# Choose option 2 (Storage configuration)
# Enter new credentials:
#   s3.access-key-id=AKIANEWKEY
#   s3.secret-access-key=newsecret
```

### Migrate from S3 to Azure

```bash
# 1. Create new Azure warehouse
pangolin-admin create-warehouse production-azure --type azure
# Follow prompts for Azure credentials

# 2. Create new catalog with Azure warehouse
pangolin-admin create-catalog prod-catalog-azure --warehouse production-azure

# 3. Migrate data (external process)
# 4. Update applications to use new catalog
# 5. Decommission old S3 warehouse when ready
```

## Related Documentation

- [Storage & Connectivity Architecture](../architecture/storage_and_connectivity.md) - Detailed multi-cloud architecture
- [PyIceberg Integration Guide](../../planning/pyiceberg_testing_guide.md) - Testing warehouses with PyIceberg
- [Authentication Guide](../architecture/authentication.md) - CLI authentication methods

## See Also

- `pangolin-admin create-catalog` - Create catalogs using warehouses
- `pangolin-admin list-catalogs` - View catalogs and their warehouses
- `pangolin-admin help` - View all available commands
