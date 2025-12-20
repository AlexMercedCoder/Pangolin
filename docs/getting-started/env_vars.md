# Environment Variables

Pangolin supports the following environment variables for configuration:

## Core API Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PORT` | The port the API server will listen on. | `8080` | No |
| `RUST_LOG` | Log level (`error`, `warn`, `info`, `debug`, `trace`). | `info` | No |

## Backend Storage (Metadata Persistence)

Pangolin determines the storage backend based on the following variables (in order of priority):

1. **`DATABASE_URL`**: If present, the driver is inferred from the scheme:
   - `postgresql://` or `postgres://` -> **PostgreSQL**
   - `mongodb://` or `mongodb+srv://` -> **MongoDB**
   - `sqlite://` or a path ending in `.db` -> **SQLite**
2. **`PANGOLIN_STORAGE_TYPE`**: Fallback if `DATABASE_URL` is missing. Options: `memory` (default), `postgres`, `mongo`, `sqlite`.

### Specific Backend Vars:
| Variable | Description | Default |
|----------|-------------|---------|
| `MONGO_DB_NAME` | Database name to use when using MongoDB. | `pangolin` |

---

## Authentication & Security

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PANGOLIN_NO_AUTH` | **Evaluation Mode**. Disables JWT/Key checks and auto-provision a default session. MUST be `"true"` to enable. | `false` | No |
| `PANGOLIN_JWT_SECRET` | Secret key for signing/verifying JWT tokens. Should be a long random string. | `default_secret` | For JWT Auth |
| `PANGOLIN_SEED_ADMIN` | If `"true"`, auto-provision a Tenant Admin on startup (even if `NO_AUTH` is false). | `false` | No |
| `PANGOLIN_ADMIN_USER` | Username for auto-provisioned Admin (used with `NO_AUTH` or `SEED_ADMIN`). | `tenant_admin` | No |
| `PANGOLIN_ADMIN_PASSWORD`| Password for auto-provisioned Admin. | `password123` | No |
| `PANGOLIN_ROOT_USER` | Initial Root user for multi-tenant management bootstrapping. | `admin` | No |
| `PANGOLIN_ROOT_PASSWORD` | Password for the initial Root user. | `password` | No |

---

## OAuth 2.0 Configuration
Used for User/Provider authentication flows.

| Variable | Description |
|----------|-------------|
| `OAUTH_GOOGLE_CLIENT_ID` | Google OAuth Client ID |
| `OAUTH_GOOGLE_CLIENT_SECRET` | Google OAuth Secret |
| `OAUTH_MICROSOFT_CLIENT_ID` | Microsoft OAuth Client ID |
| `OAUTH_MICROSOFT_CLIENT_SECRET` | Microsoft OAuth Secret |
| `OAUTH_GITHUB_CLIENT_ID` | GitHub OAuth Client ID |
| `OAUTH_GITHUB_CLIENT_SECRET` | GitHub OAuth Secret |

---

## Default Storage (fallback for Local Catalog)
These variables configure the default S3-compatible storage if no specific warehouse is configured.

| Variable | Description | Default |
|----------|-------------|---------|
| `PANGOLIN_S3_BUCKET` | Default S3 bucket name. | `pangolin` |
| `PANGOLIN_S3_PREFIX` | Default prefix for data files. | `data` |
| `AWS_ACCESS_KEY_ID` | AWS/MinIO access key. | - |
| `AWS_SECRET_ACCESS_KEY` | AWS/MinIO secret key. | - |
| `AWS_REGION` | AWS region. | `us-east-1` |
| `AWS_ENDPOINT_URL` | S3 endpoint override (required for MinIO). | - |
| `AWS_ALLOW_HTTP` | Set to `true` to allow non-HTTPS connections. | `false` |

---

## Warehouse Providers (Per-Warehouse Config)
Most storage settings (Azure Account Key, GCP Key, AWS STS Role) are configured **per-warehouse** via the API/CLI and stored in the metadata backend, rather than environment variables.

### Azure Blob Storage
Configured in Warehouse `storage_config`:
- `AZURE_STORAGE_ACCOUNT_NAME`
- `AZURE_STORAGE_ACCOUNT_KEY`
- `AZURE_STORAGE_CONTAINER`

### Google Cloud Storage
Configured in Warehouse `storage_config`:
- `GOOGLE_SERVICE_ACCOUNT_PATH`
- `GCS_BUCKET`

---

## Security Best Practices

1. **Production Mode**: Ensure `PANGOLIN_NO_AUTH` is NOT set or set to `false`.
2. **JWT Secret**: Use a 32+ character random string in production.
3. **Database Security**: Prefer `DATABASE_URL` with encrypted connections for production backends.
4. **Bootstrapping**: Use `PANGOLIN_ROOT_USER/PASSWORD` only for the first login to create your real Tenant Admins, then consider rotating or disabling it.

---

**Related Documentation**:
- [Architecture Overview](../architecture/architecture.md)
- [Authentication Guide](../authentication.md)
- [Deployment Guide](./deployment.md)
