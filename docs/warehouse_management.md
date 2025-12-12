# Warehouse Management

Pangolin supports managing multiple warehouses within a tenant. A warehouse represents a physical storage location (e.g., an S3 bucket and prefix) where data files are stored.

## Concepts
- **Warehouse**: A logical entity mapping to a storage location.
- **Storage Config**: Configuration details for the underlying storage (e.g., S3 bucket, region, credentials).

## API Endpoints

### List Warehouses
`GET /api/v1/warehouses`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`

**Response:**
```json
[
  {
    "id": "uuid",
    "name": "main_warehouse",
    "tenant_id": "uuid",
    "storage_config": {
      "type": "s3",
      "bucket": "my-bucket",
      "prefix": "warehouse/main"
    }
  }
]
```

### Create Warehouse
`POST /api/v1/warehouses`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`

**Body:**
```json
{
  "name": "analytics_warehouse",
  "storage_config": {
    "type": "s3",
    "bucket": "analytics-bucket",
    "prefix": "data"
  }
}
```

### Get Warehouse
`GET /api/v1/warehouses/{name}`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`
