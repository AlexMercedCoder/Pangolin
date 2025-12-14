# Environment Variables

Pangolin supports the following environment variables for configuration:

## Core Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `RUST_LOG` | Log level (`error`, `warn`, `info`, `debug`, `trace`) | `info` | No |

## Backend Storage (Metadata Persistence)

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | Connection string for backend storage | - | Yes |

**Supported Backends**:
- **PostgreSQL**: `postgresql://user:password@host:port/database`
- **MongoDB**: `mongodb://user:password@host:port/database`
- **SQLite**: `sqlite:///path/to/pangolin.db` or `sqlite::memory:`

See [Backend Storage Documentation](../backend_storage/README.md) for detailed configuration.

## Authentication & Security

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PANGOLIN_NO_AUTH` | Disable authentication (development only). Must be set to exactly `"true"` (case-insensitive) to enable. Any other value keeps auth enabled. | unset | No |
| `PANGOLIN_JWT_SECRET` | Secret key for signing JWTs (min 32 chars) | - | For JWT auth |
| `PANGOLIN_ROOT_USER` | Username for Root operations | - | For initial setup |
| `PANGOLIN_ROOT_PASSWORD` | Password for Root operations | - | For initial setup |

## OAuth 2.0 Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `OAUTH_GOOGLE_CLIENT_ID` | Google OAuth client ID | - | For Google OAuth |
| `OAUTH_GOOGLE_CLIENT_SECRET` | Google OAuth client secret | - | For Google OAuth |
| `OAUTH_MICROSOFT_CLIENT_ID` | Microsoft OAuth client ID | - | For Microsoft OAuth |
| `OAUTH_MICROSOFT_CLIENT_SECRET` | Microsoft OAuth client secret | - | For Microsoft OAuth |
| `OAUTH_GITHUB_CLIENT_ID` | GitHub OAuth client ID | - | For GitHub OAuth |
| `OAUTH_GITHUB_CLIENT_SECRET` | GitHub OAuth client secret | - | For GitHub OAuth |

## S3 Storage Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PANGOLIN_S3_BUCKET` | S3 bucket name | `pangolin` | For S3 storage |
| `PANGOLIN_S3_PREFIX` | S3 prefix for data | `data` | No |
| `AWS_ACCESS_KEY_ID` | AWS access key ID | - | For S3 storage |
| `AWS_SECRET_ACCESS_KEY` | AWS secret access key | - | For S3 storage |
| `AWS_REGION` | AWS region | `us-east-1` | No |
| `AWS_ENDPOINT_URL` | S3 endpoint URL (for MinIO) | - | For MinIO |
| `AWS_ALLOW_HTTP` | Allow HTTP for S3 (true/false) | `false` | No |

## Azure Blob Storage Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `AZURE_STORAGE_ACCOUNT_NAME` | Azure storage account name | - | For Azure storage |
| `AZURE_STORAGE_ACCOUNT_KEY` | Azure storage account key | - | For Azure storage |
| `AZURE_STORAGE_CONTAINER` | Azure blob container name | `pangolin` | No |

## Google Cloud Storage Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `GOOGLE_SERVICE_ACCOUNT_PATH` | Path to GCP service account JSON | - | For GCS storage |
| `GCS_BUCKET` | GCS bucket name | `pangolin` | For GCS storage |

## Cloud Provider Credential Vending

### AWS STS Configuration

For AWS STS AssumeRole credential vending:

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `AWS_ACCESS_KEY_ID` | AWS access key for STS API calls | - | For AWS STS |
| `AWS_SECRET_ACCESS_KEY` | AWS secret key for STS API calls | - | For AWS STS |
| `AWS_REGION` | AWS region | `us-east-1` | No |

**Note**: Warehouse-specific configuration (role ARN, external ID) is stored in the warehouse `storage_config`.

### Azure OAuth2 Configuration

Azure OAuth2 credentials are configured per-warehouse in the `storage_config`:
- `tenant_id` - Azure AD tenant ID
- `client_id` - Azure AD application (client) ID
- `client_secret` - Azure AD client secret

### GCP Service Account Configuration

GCP service account keys are configured per-warehouse in the `storage_config`:
- `service_account_key` - GCP service account JSON key
- `project_id` - GCP project ID

## Service Users (New)

Service users use API keys for authentication. No additional environment variables required - API keys are managed via the API.

## Federated Catalogs (New)

Federated catalog credentials are stored in the catalog configuration. No additional environment variables required.

## Example Configuration

### Development (Memory Store, No Auth)
```bash
export RUST_LOG=debug
export PANGOLIN_NO_AUTH=true  # Must be exactly "true" (case-insensitive)
export PANGOLIN_STORAGE_TYPE=memory
```

> **Security Note**: `PANGOLIN_NO_AUTH` must be set to exactly `"true"` (case-insensitive) to enable NO_AUTH mode. Setting it to `"false"`, `"0"`, or any other value will keep authentication enabled. This prevents accidental exposure of data.

### Production (S3 Storage, JWT Auth)
```bash
export RUST_LOG=info
export PANGOLIN_STORAGE_TYPE=s3
export PANGOLIN_S3_BUCKET=my-lakehouse-bucket
export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
export AWS_REGION=us-west-2
export PANGOLIN_JWT_SECRET=your-super-secret-jwt-key-min-32-chars
export PANGOLIN_ROOT_USER=admin
export PANGOLIN_ROOT_PASSWORD=secure-password
```

### Production (PostgreSQL, OAuth)
```bash
export RUST_LOG=info
export PANGOLIN_STORAGE_TYPE=postgres
export DATABASE_URL=postgresql://user:password@localhost:5432/pangolin
export PANGOLIN_JWT_SECRET=your-super-secret-jwt-key-min-32-chars
export OAUTH_GOOGLE_CLIENT_ID=your-google-client-id
export OAUTH_GOOGLE_CLIENT_SECRET=your-google-client-secret
```

## Security Best Practices

1. **Never commit secrets to version control**
2. **Use strong JWT secrets** (minimum 32 characters, random)
3. **Rotate credentials regularly**
4. **Use environment-specific configurations**
5. **Enable HTTPS in production**
6. **Disable `PANGOLIN_NO_AUTH` in production**

## Related Documentation

- [Configuration](./configuration.md) - Detailed configuration guide
- [Authentication Setup](../setup/authentication.md) - JWT and OAuth setup
- [Service Users](../service_users.md) - API key authentication
- [Deployment](./deployment.md) - Production deployment guide
