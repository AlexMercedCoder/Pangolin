# Warehouses Reference

Warehouses define the underlying object storage (S3, GCS, Azure, etc.) where catalogs store data.

## 1. API

**Base Endpoint**: `/api/v1/warehouses`

### List Warehouses
*   **Method**: `GET`
*   **Path**: `/api/v1/warehouses`
*   **Auth**: Tenant Admin Token

```bash
curl -X GET http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer <token>"
```

### Create Warehouse
*   **Method**: `POST`
*   **Path**: `/api/v1/warehouses`
*   **Body**:
    ```json
    {
      "name": "s3-warehouse",
      "storage_config": {
        "s3.bucket": "my-bucket",
        "s3.region": "us-east-1",
        "s3.access-key-id": "AKIA...",
        "s3.secret-access-key": "secret..."
      },
      "vending_strategy": {
        "AwsStatic": {
          "access_key_id": "AKIA...",
          "secret_access_key": "secret..."
        }
      }
    }
    ```

```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '...'
```

### Delete Warehouse
*   **Method**: `DELETE`
*   **Path**: `/api/v1/warehouses/{name}`

---

## 2. CLI

### List Warehouses
```bash
pangolin-admin list-warehouses
```

### Create Warehouse (S3)
```bash
pangolin-admin create-warehouse s3-warehouse \
  --type s3 \
  --bucket my-data-bucket \
  --region us-east-1 \
  --access-key "AKIA..." \
  --secret-key "secret..."
```

### Delete Warehouse
```bash
pangolin-admin delete-warehouse s3-warehouse
```

---

## 3. Python SDK (`pypangolin`)

### List Warehouses
```python
warehouses = client.warehouses.list()
for w in warehouses:
    print(w.name)
```

### Create Warehouse (S3)
```python
wh = client.warehouses.create_s3(
    name="s3-prod",
    bucket="my-prod-bucket",
    region="us-east-1",
    access_key="AKIA...",
    secret_key="secret...",
    vending_strategy="AwsStatic" 
)
print(f"Created warehouse: {wh.name}")
```

---

## 4. UI

1.  **Log in** as a **Tenant Admin**.
2.  Navigate to **Warehouses** in the sidebar.
3.  **List**: Validates current warehouses.
4.  **Create**: Click **"Create Warehouse"**.
    *   **Name**: Unique identifier.
    *   **Type**: Select S3 (MinIO provided by default config).
    *   **Credentials**: Enter keys if not using IAM roles.
    *   Click **Save**.
5.  **Manage**: Edit or delete warehouses.
