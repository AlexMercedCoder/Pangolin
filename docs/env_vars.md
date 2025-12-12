# Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (e.g., `info`, `debug`) | `info` |
| `PANGOLIN_STORAGE_TYPE` | Storage backend (`memory`, `s3`, `postgres`, `mongo`) | `memory` |
| `DATABASE_URL` | Connection string for Postgres or MongoDB | - |
| `PANGOLIN_S3_BUCKET` | S3 Bucket name | `pangolin` |
| `PANGOLIN_S3_PREFIX` | S3 Prefix for data | `data` |
| `AWS_ACCESS_KEY_ID` | S3 Access Key | - |
| `AWS_SECRET_ACCESS_KEY` | S3 Secret Key | - |
| `AWS_REGION` | S3 Region | `us-east-1` |
| `AWS_ENDPOINT_URL` | S3 Endpoint URL (for MinIO) | - |
| `AWS_ALLOW_HTTP` | Allow HTTP for S3 (true/false) | `false` |
| `PANGOLIN_ROOT_USER` | Username for Root operations (e.g., creating tenants) | - |
| `PANGOLIN_ROOT_PASSWORD` | Password for Root operations | - |
| `PANGOLIN_JWT_SECRET` | Secret key for signing JWTs | - |
