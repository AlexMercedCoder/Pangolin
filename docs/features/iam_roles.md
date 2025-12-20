# Using Cloud IAM Roles with Pangolin

## Overview

Pangolin supports role-based authentication for cloud storage, allowing you to avoid storing static credentials. Instead of distributing long-lived keys, Pangolin can assume IAM roles (AWS) or use Managed Identities (Azure/GCP) to vend temporary credentials to clients like Spark, Dremio, and PyIceberg.

## Vending Strategies

When configuring a **Warehouse**, you select a `vending_strategy` that defines how credentials are provisioned.

### 1. AwsSts (AWS Security Token Service)
Pangolin assumes a specified IAM Role and vends a temporary `AccessKey`, `SecretKey`, and `SessionToken`.
- **Best for**: Production environments on AWS.
- **Config**: Requires `role_arn` and optional `external_id`.

### 2. AwsStatic
Pangolin vends the static credentials configured in the warehouse.
- **Best for**: Centralizing credentials so they don't have to be shared with every developer or tool individually.
- **Config**: Uses the `access_key` and `secret_key` from the warehouse config.

### 3. AzureSas
Pangolin generates a Shared Access Signature (SAS) token for the specific storage container.
- **Best for**: Azure Blob Storage users.

### 4. GcpDownscoped
Pangolin vends a downscoped OAuth2 token with access restricted to specific buckets/prefixes.
- **Best for**: Google Cloud Storage users.

---

## Configuration Example

Creating a warehouse with an AWS STS strategy using the Admin CLI:

```bash
pangolin-admin create-warehouse --name "prod-s3" --type "s3"
# Then update with strategy (or use the interactive wizard)
```

**Using the API directly**:

```json
{
  "name": "s3-iam-warehouse",
  "storage_config": {
    "s3.bucket": "acme-data",
    "s3.region": "us-east-1"
  },
  "vending_strategy": {
    "type": "AwsSts",
    "role_arn": "arn:aws:iam::123456789012:role/PangolinDataAccess"
  }
}
```

---

## Current Status & Support

| Strategy | Status | Notes |
| :--- | :--- | :--- |
| **AwsStatic** | ✅ Implemented | Full support for S3/MinIO |
| **AwsSts** | 📝 Planned | STS integration in progress |
| **AzureSas** | 📝 Planned | Awaiting Azure SDK integration |
| **GcpDownscoped** | 📝 Planned | Awaiting GCP SDK integration |

> [!IMPORTANT]
> For providers/strategies marked as **Planned**, clients must provide their own storage credentials (e.g., via environment variables or Spark properties) even if the warehouse is configured in Pangolin.

---

## Best Practices

1. **Principle of Least Privilege**: Ensure the IAM role assumed by Pangolin only has access to the specific buckets/prefixes used by the warehouse.
2. **Short Session Durations**: Prefer shorter TTLs for vended credentials (e.g., 1 hour) to minimize the impact of token leakage.
3. **Use External IDs**: When configuring cross-account `AwsSts` roles, always use an `external_id` to prevent the "confused deputy" problem.

## Related Documentation
- [Warehouse Management](warehouse_management.md)
- [Security & Credential Vending](security_vending.md)
- [Architecture: Signer Trait](../architecture/signer-trait.md)
