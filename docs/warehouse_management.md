# Warehouse Management

Pangolin uses warehouses to store credentials and storage configuration. Warehouses are internal to Pangolin and are referenced by catalogs.

## Concepts
- **Warehouse**: Stores storage credentials and configuration (S3, Azure, GCS, etc.)
- **Catalog**: References a warehouse and specifies a storage location for that catalog
- **Credential Vending**: Warehouses can be configured to use STS (dynamic credentials) or static credentials

## Warehouse Configuration

### use_sts Flag
- **`true`**: Pangolin will vend temporary STS credentials to clients (recommended for production)
- **`false`**: Pangolin will pass through static credentials from the warehouse configuration

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
    "use_sts": true,
    "storage_config": {
      "type": "s3",
      "bucket": "my-bucket",
      "region": "us-east-1",
      "role_arn": "arn:aws:iam::123456789012:role/PangolinRole"
    }
  }
]
```

### Create Warehouse
`POST /api/v1/warehouses`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`

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

**Body (without STS - static credentials):**
```json
{
  "name": "dev_warehouse",
  "use_sts": false,
  "storage_config": {
    "type": "s3",
    "bucket": "dev-bucket",
    "region": "us-east-1",
    "access_key_id": "AKIAIOSFODNN7EXAMPLE",
    "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
  }
}
```

### Get Warehouse
`GET /api/v1/warehouses/{name}`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`

## Catalog Association

After creating a warehouse, you can create catalogs that reference it:

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

## Best Practices

1. **Use STS in Production**: Set `use_sts: true` for production warehouses to leverage temporary credentials
2. **Static Credentials for Development**: Use `use_sts: false` with static credentials for local development
3. **Separate Warehouses by Environment**: Create different warehouses for dev, staging, and production
4. **Scope Storage Locations**: Use the catalog's `storage_location` to organize data within a warehouse
