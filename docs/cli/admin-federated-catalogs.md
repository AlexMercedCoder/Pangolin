# Federated Catalogs (Admin CLI)

Manage federated catalogs that proxy requests to other Iceberg REST catalogs.

## Create Federated Catalog

Create a federated catalog that proxies to a remote Iceberg REST catalog:

```bash
pangolin-admin create-federated-catalog fed_catalog_b \
  -P uri=http://remote-server:8080/v1/catalog_b \
  --storage-location s3://my-bucket/fed_catalog_b \
  -P token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... \
  --timeout 30
```

**Parameters:**
- `name` (required): Name of the federated catalog
- `--storage-location` (required): S3/Azure/GCS path (required even for federated catalogs)
- `-P` / `--property` (optional): Key-value properties for configuration (e.g., `-P key=value`)
- `--timeout` (optional): Timeout in seconds (default: 30)

### Common Properties:
- `uri`: Base URL of the remote catalog (Required)
- `token`: Bearer token for authentication
- `credential`: Client credential (id:secret) for OAuth/Basic Auth
- `warehouse`: Warehouse name to use

### Authentication Examples

#### No Authentication
```bash
pangolin-admin create-federated-catalog public_catalog \
  -P uri=http://public-server:8080/v1/catalog \
  --storage-location s3://my-bucket/public_catalog
```

#### Bearer Token
```bash
pangolin-admin create-federated-catalog secure_catalog \
  -P uri=http://secure-server:8080/v1/catalog \
  --storage-location s3://my-bucket/secure_catalog \
  -P token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Basic Auth / Client Credentials
```bash
pangolin-admin create-federated-catalog basic_catalog \
  -P uri=http://basic-server:8080/v1/catalog \
  --storage-location s3://my-bucket/basic_catalog \
  -P credential=admin:secret123
```

#### API Key
```bash
pangolin-admin create-federated-catalog api_catalog \
  -P uri=http://api-server:8080/v1/catalog \
  --storage-location s3://my-bucket/api_catalog \
  -P x-api-key=abc123def456
```

## List Federated Catalogs

List all federated catalogs in the current tenant:

```bash
pangolin-admin list-federated-catalogs
```

**Example Output:**
```
| Name           | URI                                         | Properties   |
|----------------|---------------------------------------------|--------------|
| fed_catalog_b  | http://remote:8080/v1/catalog_b             | 3 props      |
| fed_catalog_c  | http://other:8080/v1/catalog_c              | 2 props      |
```

## Test Federated Catalog

Test connectivity and authentication to a federated catalog:

```bash
pangolin-admin test-federated-catalog fed_catalog_b
```

**Example Output:**
```
Testing connection to federated catalog 'fed_catalog_b'...

1. Checking catalog configuration... ✅
2. Testing namespace listing... ✅

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Federated catalog 'fed_catalog_b' is working correctly!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Namespaces accessible: ['db_a', 'db_b']
```

### Delete Federated Catalog
Delete a federated catalog.

**Syntax**:
```bash
pangolin-admin delete-federated-catalog <name>
```

**Example**:
```bash
pangolin-admin delete-federated-catalog fed_catalog_b
```

## Operations

### Sync Federated Catalog
Trigger a manual synchronization of metadata from the remote catalog.

**Syntax**:
```bash
pangolin-admin sync-federated-catalog <name>
```

**Example**:
```bash
pangolin-admin sync-federated-catalog fed_catalog_b
```

### Get Federated Stats
View synchronization statistics and health metrics for a federated catalog.

**Syntax**:
```bash
pangolin-admin get-federated-stats <name>
```

**Example**:
```bash
pangolin-admin get-federated-stats fed_catalog_b
```

## Use Cases

### Cross-Tenant Data Sharing

Tenant A can access Tenant B's data through a federated catalog:

1. Tenant B creates a catalog and grants access to Tenant A
2. Tenant B generates a token for Tenant A
3. Tenant A creates a federated catalog pointing to Tenant B's catalog
4. Tenant A's users can now query Tenant B's data

### Multi-Region Data Access

Access data from different regions through federated catalogs:

```bash
# Create federated catalog for US region
pangolin-admin create-federated-catalog us_catalog \
  -P uri=http://us-server:8080/v1/catalog \
  --storage-location s3://my-bucket/us_catalog \
  -P token=$US_TOKEN

# Create federated catalog for EU region
pangolin-admin create-federated-catalog eu_catalog \
  -P uri=http://eu-server:8080/v1/catalog \
  --storage-location s3://my-bucket/eu_catalog \
  -P token=$EU_TOKEN
```

### Hybrid Cloud Setup

Federate catalogs across different cloud providers:

```bash
# AWS catalog
pangolin-admin create-federated-catalog aws_catalog \
  -P uri=http://aws-pangolin:8080/v1/catalog \
  --storage-location s3://aws-bucket/catalog \
  -P token=$AWS_TOKEN

# Azure catalog
pangolin-admin create-federated-catalog azure_catalog \
  -P uri=http://azure-pangolin:8080/v1/catalog \
  --storage-location abfs://azure-container/catalog \
  -P token=$AZURE_TOKEN
```

## Troubleshooting

### Connection Timeout

If the test fails with a timeout:

```bash
```bash
# Increase timeout
pangolin-admin create-federated-catalog slow_catalog \
  -P uri=http://slow-server:8080/v1/catalog \
  --storage-location s3://my-bucket/slow_catalog \
  --timeout 60
```

### Authentication Errors

If you get 401/403 errors:
1. Verify the token is still valid
2. Check that the token has the correct permissions
3. Ensure the auth type matches the remote server's requirements

### Network Issues

If you get connection refused:
1. Verify the base URL is correct
2. Check network connectivity to the remote server
3. Ensure the remote server is running and accessible
