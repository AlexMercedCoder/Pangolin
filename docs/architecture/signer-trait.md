# Signer Trait

## Overview
The `Signer` trait provides a secure mechanism for Pangolin to vend temporary, downscoped credentials or presigned URLs to clients (like PyIceberg or Spark) for direct access to object storage.

**Source**: `pangolin_store/src/signer.rs`

---

## Credentials Enum
The `Credentials` enum encapsulates provider-specific access tokens.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Credentials {
    // AWS S3 credentials (STS or Static)
    Aws {
        access_key_id: String,
        secret_access_key: String,
        session_token: Option<String>,
        expiration: Option<DateTime<Utc>>,
    },
    // Azure SAS token
    Azure {
        sas_token: String,
        account_name: String,
        expiration: DateTime<Utc>,
    },
    // GCP OAuth2 token
    Gcp {
        access_token: String,
        expiration: DateTime<Utc>,
    },
}
```

---

## The Signer Trait
Implementations of this trait interact with cloud IAM services to generate temporary tokens.

```rust
#[async_trait]
pub trait Signer: Send + Sync {
    /// Get temporary credentials for accessing a specific table location.
    /// Implementation should scope permissions to the provided location prefix.
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials>;

    /// Generate a presigned URL for a specific file location.
    async fn presign_get(&self, location: &str) -> Result<String>;
}
```

### Key Behaviors
1.  **Downscoping**: Credentials vended via `get_table_credentials` are typically scoped to the specific S3/GCS prefix of the Iceberg table to prevent unauthorized access to other data in the bucket.
2.  **Expiration handling**: Clients are expected to refresh credentials based on the `expiration` field provided in the response.
3.  **Transport Security**: Vended credentials should only be transmitted over HTTPS.

---

## SignerImpl
A default implementation of the `Signer` trait is provided for basic setups, though full cloud integrations typically require the `aws-sts`, `azure-oauth`, or `gcp-oauth` feature flags.
- `prefix`: S3 prefix to scope credentials to (e.g., "warehouse1/catalog1/")

**Returns**: `Result<Credentials>`

**Usage**:
```rust
let creds = store.get_table_credentials(
    "s3://warehouse1/catalog1/table1"
).await?;
```

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
