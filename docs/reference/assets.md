# Assets Reference

Assets represent data entities (Tables, Views) or generic files managed in catalogs.

## 1. API

**Base Endpoint**: `/api/v1/assets` or via Catalog structure.

### Get Asset by ID
*   **Method**: `GET`
*   **Path**: `/api/v1/assets/{asset_id}`

### Search Assets
*   **Method**: `GET`
*   **Path**: `/api/v1/search/assets`
*   **Params**: `q={query}`

### Register Generic Asset
*   **Method**: `POST`
*   **Path**: `/api/v1/catalogs/{catalog}/namespaces/{ns}/assets`
*   **Body**:
    ```json
    {
      "name": "raw-csv",
      "kind": "file",
      "location": "s3://bucket/path/file.csv",
      "properties": {}
    }
    ```

---

## 2. CLI

### Search
```bash
pangolin-admin search "sales"
```

### Get Details
```bash
pangolin-admin get-asset-details --id <uuid>
```

### View Catalog Tree
```bash
pangolin-admin explorer tree --catalog analytics
```

---

## 3. Python SDK (`pypangolin`)

### Search
```python
results = client.search.search(query="sales", limit=5)
for r in results:
    print(r.name, r.catalog, r.namespace)
```

### Get Asset
```python
# By ID
asset = client.search.get_asset("uuid")

# By Path
asset = client.catalogs.get("analytics").namespaces("sales").tables("transactions").get()
```

### Register Generic Asset
```python
client.catalogs.namespaces("analytics").register_asset(
    namespace="raw",
    name="data-file",
    kind="csv",
    location="s3://bucket/data.csv"
)
```

---

## 4. UI

1.  **Log in**.
2.  Navigate to **Explorer** (or **Catalogs > [Select Catalog]**).
3.  **Browse**: Use the sidebar tree to navigate namespaces.
4.  **Search**: Use the global search bar at the top.
5.  **Details**: Click an asset to view Schema, History, and Metadata.
