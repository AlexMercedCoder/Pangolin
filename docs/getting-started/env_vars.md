# Environment Variables

Pangolin is configured via environment variables. This guide lists all available options.

## 🚀 Core API Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | The port the API server will listen on. | `8080` |
| `RUST_LOG` | Log level (`error`, `warn`, `info`, `debug`, `trace`). | `info` |

## 💾 Metadata Persistence

Pangolin stores its own metadata (tenants, users, catalogs) in a backend database.

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | Connection string for Postgres, MongoDB, or SQLite. | (None) |
| `PANGOLIN_STORAGE_TYPE` | Storage driver if `DATABASE_URL` is missing. (`memory`, `postgres`, `mongo`, `sqlite`). | `memory` |
| `MONGO_DB_NAME` | Database name (when using MongoDB). | `pangolin` |

> [!NOTE]
> If `DATABASE_URL` is provided, the driver is automatically inferred from the URI scheme (e.g., `postgresql://`, `mongodb://`, `sqlite://`).

## 🛡️ Authentication & Security

| Variable | Description | Default |
|----------|-------------|---------|
| `PANGOLIN_NO_AUTH` | **Evaluation Mode**. Auto-provisions a default tenant and admin user. Set to `"true"` to enable. | `false` |
| `PANGOLIN_JWT_SECRET` | Secret key for JWT signing. **MUST** be changed in production. | `default_secret` |
| `PANGOLIN_SEED_ADMIN` | Auto-provision a Tenant Admin even if `NO_AUTH` is false. Set to `"true"`. | `false` |
| `PANGOLIN_ADMIN_USER` | Username for seed admin. | `tenant_admin` |
| `PANGOLIN_ADMIN_PASSWORD` | Password for seed admin. | `password123` |
| `PANGOLIN_ROOT_USER` | Initial Root user for multi-tenant bootstrapping. | `admin` |
| `PANGOLIN_ROOT_PASSWORD` | Password for Root user. | `password` |

## 🌐 OAuth 2.0 (External Providers)

Required for enabling "Login with Google/GitHub/Microsoft" in the UI.

| Variable | Provider |
|----------|----------|
| `OAUTH_GOOGLE_CLIENT_ID` / `_SECRET` | Google |
| `OAUTH_MICROSOFT_CLIENT_ID` / `_SECRET` | Microsoft |
| `OAUTH_GITHUB_CLIENT_ID` / `_SECRET` | GitHub |

## ☁️ Cloud Provider Features

When building with cloud features (`--features aws-sts`, etc.), these standard variables are used by the underlying SDKs for the **Signer** logic.

| Variable | Description |
|----------|-------------|
| `AWS_ACCESS_KEY_ID` | AWS Credentials for STS / Signer. |
| `AWS_SECRET_ACCESS_KEY` | AWS Credentials for STS / Signer. |
| `AWS_REGION` | Default AWS region. |
| `AWS_ENDPOINT_URL` | Override for MinIO or custom S3 backends. |

## 🚦 Security Checklist

1. **Disable NO_AUTH**: Ensure `PANGOLIN_NO_AUTH` is unset or `false` in production.
2. **Rotate JWT Secret**: Use a 32+ character random string.
3. **Strong Root Password**: Change the default `admin/password` immediately.
4. **Use DATABASE_URL**: Avoid the `memory` store for any non-trivial use case.
