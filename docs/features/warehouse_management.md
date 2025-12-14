# Warehouse Management

## Overview

Warehouses in Pangolin define storage configurations and credential management strategies for your Iceberg tables. Each warehouse represents a storage backend (S3, Azure Blob, GCS) and controls how clients access data.

**Key Concepts**:
- **Warehouse**: Storage configuration and credential vending settings
- **Catalog**: References a warehouse and defines a storage location
- **Credential Vending**: Automatic provisioning of temporary credentials to clients

---

## Creating a Warehouse

### Basic Warehouse (Static Credentials)

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "dev_warehouse",
    "use_sts": false,
    "storage_config": {
      "type": "s3",
      "bucket": "my-dev-bucket",
      "region": "us-east-1"
    }
  }'
```

**Configuration**:
- `use_sts: false` - Clients use static credentials from their environment
- Suitable for development and testing

### Production Warehouse (STS Credential Vending)

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production_warehouse",
    "use_sts": true,
    "storage_config": {
      "type": "s3",
      "bucket": "my-prod-bucket",
      "region": "us-east-1",
      "role_arn": "arn:aws:iam::123456789012:role/PangolinDataAccess"
    }
  }'
```

**Configuration**:
- `use_sts: true` - Pangolin vends temporary STS credentials to clients
- Recommended for production environments
- Requires IAM role with S3 access permissions

---

## Storage Backend Configuration

### AWS S3

```json
{
  "type": "s3",
  "bucket": "my-bucket",
  "region": "us-east-1",
  "role_arn": "arn:aws:iam::123456789012:role/DataAccess"
}
```

### Azure Blob Storage

```json
{
  "type": "azure",
  "account_name": "mystorageaccount",
  "container": "data"
}
```

### Google Cloud Storage

```json
{
  "type": "gcs",
  "bucket": "my-gcs-bucket",
  "project_id": "my-project"
}
```

### MinIO (S3-Compatible)

```json
{
  "type": "s3",
  "bucket": "minio-bucket",
  "endpoint": "http://minio:9000",
  "allow_http": true
}
```

---

## API Endpoints

### List Warehouses
`GET /api/v1/warehouses`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`

### Create Warehouse
`POST /api/v1/warehouses`

**Body (with STS):**
```json
{
  "name": "main_warehouse",
  "use_sts": true,
  "storage_config": {
    "type": "s3",
    "bucket": "my-bucket",
    "region": "us-east-1",
    "role_arn": "arn:aws:iam::123456789012:role/PangolinRole"
  }
}
```

### Get Warehouse
`GET /api/v1/warehouses/{name}`

---

## Catalog Association

After creating a warehouse, create catalogs that reference it:

```bash
curl -X POST http://localhost:8080/api/v1/catalogs \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "analytics",
    "warehouse_name": "main_warehouse",
    "storage_location": "s3://my-bucket/analytics"
  }'
```

---

## Best Practices

1. **Use STS in Production**: Set `use_sts: true` for production warehouses
2. **Static Credentials for Development**: Use `use_sts: false` for local development
3. **Separate Warehouses by Environment**: Create different warehouses for dev, staging, production
4. **Scope Storage Locations**: Use catalog's `storage_location` to organize data

---

## Related Documentation

- [Security & Credential Vending](./security_vending.md) - Detailed credential vending guide
- [AWS S3 Storage](../storage/storage_s3.md) - S3 backend configuration
- [Client Configuration](../getting-started/client_configuration.md) - PyIceberg, Spark, Trino setup
