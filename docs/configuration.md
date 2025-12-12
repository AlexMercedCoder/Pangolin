# Configuration

Pangolin is primarily configured via Environment Variables. 

## Runtime Configuration
The application loads configuration at startup. Currently, the following behaviors are configurable:

- **Storage Backend**: Controlled by `PANGOLIN_STORAGE_TYPE` (e.g., `s3`, `memory`).
- **Logging**: Controlled by `RUST_LOG`.

## Iceberg Client Configuration
Clients connecting to Pangolin should use the `/v1/config` endpoint to discover defaults and overrides.

Example response from `GET /v1/config`:
```json
{
  "defaults": {},
  "overrides": {}
}
```
