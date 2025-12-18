# Federated Catalogs (Admin CLI)

Manage federated catalogs that proxy requests to other Iceberg REST catalogs.

## Create Federated Catalog

Create a federated catalog that proxies to a remote Iceberg REST catalog:

```bash
pangolin-admin create-federated-catalog fed_catalog_b \
  --base-url http://remote-server:8080/v1/catalog_b \
  --storage-location s3://my-bucket/fed_catalog_b \
  --auth-type BearerToken \
  --token eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... \
  --timeout 30
```

**Parameters:**
- `name` (required): Name of the federated catalog
- `--base-url` (required): Base URL of the remote catalog
- `--storage-location` (required): S3/Azure/GCS path (required even for federated catalogs)
- `--auth-type` (optional): Authentication type - `None`, `BasicAuth`, `BearerToken`, `ApiKey` (default: `None`)
- `--token` (conditional): Bearer token (required if `--auth-type BearerToken`)
- `--username` (conditional): Username (required if `--auth-type BasicAuth`)
- `--password` (conditional): Password (required if `--auth-type BasicAuth`)
- `--api-key` (conditional): API key (required if `--auth-type ApiKey`)
- `--timeout` (optional): Timeout in seconds (default: 30)

### Authentication Examples

#### No Authentication
```bash
pangolin-admin create-federated-catalog public_catalog \
  --base-url http://public-server:8080/v1/catalog \
  --storage-location s3://my-bucket/public_catalog \
  --auth-type None
```

#### Bearer Token
```bash
pangolin-admin create-federated-catalog secure_catalog \
  --base-url http://secure-server:8080/v1/catalog \
  --storage-location s3://my-bucket/secure_catalog \
  --auth-type BearerToken \
  --token eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Basic Auth
```bash
pangolin-admin create-federated-catalog basic_catalog \
  --base-url http://basic-server:8080/v1/catalog \
  --storage-location s3://my-bucket/basic_catalog \
  --auth-type BasicAuth \
  --username admin \
  --password secret123
```

#### API Key
```bash
pangolin-admin create-federated-catalog api_catalog \
  --base-url http://api-server:8080/v1/catalog \
  --storage-location s3://my-bucket/api_catalog \
  --auth-type ApiKey \
  --api-key abc123def456
```

## List Federated Catalogs

List all federated catalogs in the current tenant:

```bash
pangolin-admin list-federated-catalogs
```

**Example Output:**
```
| Name           | Base URL                                    | Auth Type    |
|----------------|---------------------------------------------|--------------|
| fed_catalog_b  | http://remote:8080/v1/catalog_b            | BearerToken  |
| fed_catalog_c  | http://other:8080/v1/catalog_c             | ApiKey       |
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

## Delete Federated Catalog

Delete a federated catalog (with confirmation):

```bash
pangolin-admin delete-federated-catalog fed_catalog_b
```

**Example Output:**
```
Are you sure you want to delete federated catalog 'fed_catalog_b'? (yes/no): yes
✅ Federated catalog 'fed_catalog_b' deleted successfully!
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
  --base-url http://us-server:8080/v1/catalog \
  --storage-location s3://my-bucket/us_catalog \
  --auth-type BearerToken \
  --token $US_TOKEN

# Create federated catalog for EU region
pangolin-admin create-federated-catalog eu_catalog \
  --base-url http://eu-server:8080/v1/catalog \
  --storage-location s3://my-bucket/eu_catalog \
  --auth-type BearerToken \
  --token $EU_TOKEN
```

### Hybrid Cloud Setup

Federate catalogs across different cloud providers:

```bash
# AWS catalog
pangolin-admin create-federated-catalog aws_catalog \
  --base-url http://aws-pangolin:8080/v1/catalog \
  --storage-location s3://aws-bucket/catalog \
  --auth-type BearerToken \
  --token $AWS_TOKEN

# Azure catalog
pangolin-admin create-federated-catalog azure_catalog \
  --base-url http://azure-pangolin:8080/v1/catalog \
  --storage-location abfs://azure-container/catalog \
  --auth-type BearerToken \
  --token $AZURE_TOKEN
```

## Troubleshooting

### Connection Timeout

If the test fails with a timeout:

```bash
# Increase timeout
pangolin-admin create-federated-catalog slow_catalog \
  --base-url http://slow-server:8080/v1/catalog \
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
