# Configuration

Pangolin is primarily configured via Environment Variables. 

## Runtime Configuration
The application loads configuration at startup. Currently, the following behaviors are configurable:

- **Storage Backend**: Controlled by `PANGOLIN_STORAGE_TYPE` (e.g., `s3`, `memory`).
- **Logging**: Controlled by `RUST_LOG`.

## Storage Configuration

### S3 / MinIO
To use S3-compatible storage, set `PANGOLIN_STORAGE_TYPE=s3` and configure the following environment variables:

- `AWS_ACCESS_KEY_ID`: Access key.
- `AWS_SECRET_ACCESS_KEY`: Secret key.
- `AWS_REGION`: Region (e.g., `us-east-1`).
- `AWS_ENDPOINT_URL`: (Optional) Custom endpoint for MinIO or other S3-compatible services.

### Memory
Set `PANGOLIN_STORAGE_TYPE=memory` for an ephemeral in-memory store (default). No further configuration is required.

## Iceberg Client Configuration
Clients connecting to Pangolin should use the `/v1/config` endpoint to discover defaults and overrides.

Example response from `GET /v1/config`:
```json
{
  "defaults": {},
  "overrides": {}
}
```
