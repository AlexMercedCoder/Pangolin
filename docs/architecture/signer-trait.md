# Signer Trait

## Overview

The `Signer` trait provides an abstraction for generating pre-signed URLs for cloud storage access. It enables secure, temporary access to S3-compatible storage without exposing long-term credentials.

**Location**: `pangolin_store/src/signer.rs`

## Purpose

The `Signer` trait provides:
- **Credential Vending**: Generate temporary, scoped credentials for S3 access
- **Pre-signed URLs**: Create time-limited URLs for direct object access
- **Multi-cloud Support**: Abstract interface works with AWS S3, Azure Blob, GCP Storage
- **Security**: Implements least-privilege access with expiration

## Implementations

- **SignerImpl** (`pangolin_store/src/signer.rs`) - AWS S3 signer using AWS SDK
- **AzureSigner** (`pangolin_store/src/azure_signer.rs`) - Azure Blob Storage signer
- **GcpSigner** (`pangolin_store/src/gcp_signer.rs`) - Google Cloud Storage signer

## Trait Definition

```rust
#[async_trait]
pub trait Signer: Send + Sync {
    async fn generate_presigned_url(
        &self,
        bucket: &str,
        key: &str,
        expiration_secs: u64,
    ) -> Result<String>;
    
    async fn vend_credentials(
        &self,
        warehouse_config: &std::collections::HashMap<String, String>,
        prefix: &str,
    ) -> Result<VendedCredentials>;
}
```

## Methods

### `generate_presigned_url`

```rust
async fn generate_presigned_url(
    &self,
    bucket: &str,
    key: &str,
    expiration_secs: u64,
) -> Result<String>;
```

Generates a pre-signed URL for direct object access.

**Parameters**:
- `bucket`: S3 bucket name
- `key`: Object key/path within the bucket
- `expiration_secs`: URL validity duration in seconds

**Returns**: `Result<String>` - Pre-signed URL

**Usage**:
```rust
let url = signer.generate_presigned_url(
    "my-bucket",
    "path/to/metadata.json",
    3600, // 1 hour
).await?;
```

**Use Cases**:
- Providing read access to table metadata files
- Allowing direct downloads of data files
- Temporary access for external tools

### `vend_credentials`

```rust
async fn vend_credentials(
    &self,
    warehouse_config: &std::collections::HashMap<String, String>,
    prefix: &str,
) -> Result<VendedCredentials>;
```

Generates temporary, scoped credentials for S3 access.

**Parameters**:
- `warehouse_config`: Warehouse configuration containing storage settings
- `prefix`: S3 prefix to scope credentials to (e.g., "warehouse1/catalog1/")

**Returns**: `Result<VendedCredentials>` - Temporary credentials

**VendedCredentials Structure**:
```rust
pub struct VendedCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub expiration: Option<DateTime<Utc>>,
}
```

**Usage**:
```rust
let creds = signer.vend_credentials(
    &warehouse.storage_config,
    "warehouse1/catalog1/",
).await?;

// Return to client for direct S3 access
```

**Use Cases**:
- PyIceberg client credential vending
- Spark/Trino engine access
- Direct write operations from compute engines

## Credential Vending Flow

```
1. Client requests table metadata
2. API validates permissions
3. Signer generates scoped credentials
4. Credentials returned via X-Iceberg-Access-Delegation header
5. Client uses credentials for direct S3 access
```

## Security Considerations

### Scope Limitation
Credentials should be scoped to the minimum necessary prefix:
```rust
// Good - scoped to specific table
let prefix = format!("{}/{}/{}/", warehouse, catalog, namespace);

// Bad - too broad
let prefix = "/";
```

### Expiration
Set appropriate expiration times:
```rust
// Short-lived for metadata reads
let expiration = 3600; // 1 hour

// Longer for large data operations
let expiration = 14400; // 4 hours
```

### Audit Logging
Log all credential vending operations:
```rust
store.log_audit_event(tenant_id, AuditLogEntry {
    action: "vend_credentials".to_string(),
    resource: format!("{}/{}", catalog, table),
    // ...
}).await?;
```

## AWS S3 Implementation

The default `SignerImpl` uses AWS STS to assume roles:

```rust
pub struct SignerImpl {
    s3_client: aws_sdk_s3::Client,
    sts_client: aws_sdk_sts::Client,
}
```

**Configuration**:
```toml
[warehouse.storage_config]
s3.endpoint = "https://s3.amazonaws.com"
s3.region = "us-east-1"
s3.role-arn = "arn:aws:iam::123456789012:role/IcebergAccess"
```

## Azure Blob Implementation

Azure uses SAS tokens for temporary access:

```rust
pub struct AzureSigner {
    account_name: String,
    account_key: String,
}
```

**Configuration**:
```toml
[warehouse.storage_config]
azure.account-name = "mystorageaccount"
azure.account-key = "..."
```

## GCP Storage Implementation

GCP uses service account impersonation:

```rust
pub struct GcpSigner {
    service_account_email: String,
    private_key: String,
}
```

**Configuration**:
```toml
[warehouse.storage_config]
gcs.project-id = "my-project"
gcs.service-account = "iceberg@my-project.iam.gserviceaccount.com"
```

## Error Handling

All methods return `Result<T>` where errors can include:
- **CredentialError**: Failed to generate credentials
- **ConfigurationError**: Invalid warehouse configuration
- **PermissionError**: Insufficient permissions to assume role
- **NetworkError**: Failed to communicate with cloud provider

## Testing

Mock signer for testing:

```rust
pub struct MockSigner;

#[async_trait]
impl Signer for MockSigner {
    async fn generate_presigned_url(
        &self,
        bucket: &str,
        key: &str,
        _expiration_secs: u64,
    ) -> Result<String> {
        Ok(format!("https://{}.s3.amazonaws.com/{}", bucket, key))
    }
    
    async fn vend_credentials(
        &self,
        _warehouse_config: &HashMap<String, String>,
        _prefix: &str,
    ) -> Result<VendedCredentials> {
        Ok(VendedCredentials {
            access_key_id: "MOCK_KEY".to_string(),
            secret_access_key: "MOCK_SECRET".to_string(),
            session_token: Some("MOCK_TOKEN".to_string()),
            expiration: Some(Utc::now() + chrono::Duration::hours(1)),
        })
    }
}
```

## Best Practices

1. **Minimal Scope**: Always scope credentials to the narrowest prefix possible
2. **Short Expiration**: Use short expiration times (1-4 hours)
3. **Audit Everything**: Log all credential vending operations
4. **Validate Config**: Verify warehouse configuration before vending
5. **Handle Errors**: Gracefully handle cloud provider errors

## Integration with CatalogStore

The `CatalogStore` trait extends `Signer`, making signing capabilities available to all store implementations:

```rust
#[async_trait]
pub trait CatalogStore: Send + Sync + Signer {
    // ... catalog operations ...
}
```

This allows handlers to vend credentials directly:

```rust
let creds = store.vend_credentials(&warehouse.storage_config, &prefix).await?;
```

## See Also

- [CatalogStore Trait](./catalog-store-trait.md)
- [Credential Vending Guide](../guides/credential-vending.md)
- [Storage Configuration](../configuration/storage.md)
