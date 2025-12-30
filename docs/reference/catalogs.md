# Catalogs Reference

Catalogs are Iceberg REST catalogs. Pangolin supports two types:
*   **Local**: Managed purely by Pangolin, stored in a Warehouse.
*   **Federated**: Proxies to another Iceberg catalog (e.g., Dremio, Snowflake, Unity, Glue).

## 1. API

### Local Catalogs
*   **Endpoint**: `/api/v1/catalogs`
*   **Method**: `POST`
*   **Body**:
    ```json
    {
      "name": "analytics",
      "warehouse_name": "s3-prod",
      "catalog_type": "Local"
    }
    ```

### Federated Catalogs
*   **Endpoint**: `/api/v1/federated-catalogs`
*   **Method**: `POST`
*   **Body**:
    ```json
    {
      "name": "snowflake-mirror",
      "config": {
        "uri": "https://...",
        "warehouse": "optional-warehouse-ref",
        "credential": "optional-credential",
        "properties": {}
      }
    }
    ```

---

## 2. CLI

### Create Local Catalog
```bash
pangolin-admin create-catalog analytics --warehouse s3-prod
```

### Create Federated Catalog
```bash
pangolin-admin create-federated-catalog snowflake-mirror \
  --storage-location "s3://bucket/prefix" \
  -P uri="https://snowflake..." \
  -P token="sess:..."
```

### Sync Federated Catalog
Triggers a metadata discovery from the remote catalog.
```bash
pangolin-admin sync-federated-catalog snowflake-mirror
```

---

## 3. Python SDK (`pypangolin`)

### Create Local Catalog
```python
cat = client.catalogs.create(
    name="analytics",
    warehouse="s3-prod",
    type="Local"
)
```

### Create Federated Catalog
```python
fed_cat = client.federated_catalogs.create(
    name="snowflake-mirror",
    uri="https://...",
    properties={"warehouse": "my-wh"}
)
```

### Sync
```python
client.federated_catalogs.sync("snowflake-mirror")
```

---

## 4. UI

1.  **Log in** as a **Tenant Admin**.
2.  Navigate to **Catalogs** in the sidebar.
3.  **Create**: Click **"Create Catalog"**.
    *   **Name**: Identifier.
    *   **Type**: Choose **Local** (Standard) or **Federated**.
    *   **Details**:
        *   Local: Select a **Warehouse**.
        *   Federated: Enter URI and credentials.
    *   Click **Save**.
4.  **Browse**: Click on a catalog name to open the **Data Explorer**.
