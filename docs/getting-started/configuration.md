# Configuration Overview

Pangolin follow a "Configuration-over-Code" philosophy, allowing for flexible deployments across different infrastructures.

## Runtime Configuration

Most settings are managed via **Environment Variables** at startup. This includes everything from the port number to the metadata persistence backend.

- For a complete list of variables, see **[Environment Variables](./env_vars.md)**.
- For deployment-specific patterns, see **[Deployment Guide](./deployment.md)**.

## Client Configuration Discovery

Pangolin implements the Iceberg REST specification for **Config Discovery**. Clients (like Spark or PyIceberg) can query the `/v1/config` endpoint to receive default properties and overrides.

This is particularly useful for:
1. **S3 Endpoint Overrides**: Telling clients to use a local MinIO instance.
2. **IO Implementation**: Specifying the storage driver (e.g., `S3FileIO`).
3. **Warehouse Context**: Providing default warehouse identifiers.

### Example Discovery Call

```bash
GET http://localhost:8080/v1/config?warehouse=main
```

**Response:**
```json
{
  "defaults": {
    "s3.endpoint": "http://minio:9000",
    "s3.path-style-access": "true"
  },
  "overrides": {
    "io-impl": "org.apache.iceberg.aws.s3.S3FileIO"
  }
}
```

## Storage Management

While core system configuration uses environment variables, **Storage Connectors** (Warehouses) are configured dynamically via the Admin API/CLI. This allows you to add or modify storage locations without restarting the Pangolin service.

- To learn how to configure storage, see **[Warehouse Management](../features/warehouse_management.md)**.
