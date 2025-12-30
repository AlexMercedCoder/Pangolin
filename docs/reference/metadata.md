# Business Metadata Reference

Metadata allows attaching custom key-value pairs to assets for governance and discovery.

## 1. API

**Base Endpoint**: `/api/v1/metadata`

### Set Metadata
*   **Method**: `PUT`
*   **Path**: `/api/v1/metadata`
*   **Body**:
    ```json
    {
      "entity_type": "Asset",
      "entity_id": "uuid...",
      "key": "owner",
      "value": "data-team"
    }
    ```

### Get Metadata
*   **Method**: `GET`
*   **Path**: `/api/v1/metadata/{entity_type}/{entity_id}`

### Delete Metadata
*   **Method**: `DELETE`
*   **Path**: `/api/v1/metadata/{entity_type}/{entity_id}/{key}`

---

## 2. CLI

### Set Metadata
```bash
pangolin-admin set-metadata \
  --entity-type Asset \
  --entity-id <uuid> \
  --key owner \
  --value data-team
```

### Get Metadata
```bash
pangolin-admin get-metadata \
  --entity-type Asset \
  --entity-id <uuid>
```

---

## 3. Python SDK (`pypangolin`)

### Set
```python
client.metadata.set(
    entity_type="Asset",
    entity_id="uuid...",
    properties={"owner": "data-team", "sla": "24h"}
)
```

### Get
```python
meta = client.metadata.get("Asset", "uuid...")
print(meta)
```

---

## 4. UI

1.  Navigate to an **Asset** in the **Explorer**.
2.  Select the **Metadata** tab.
3.  **Add**: Click "Add Property", enter key/value, Save.
4.  **Edit/Delete**: Use the icons next to existing properties.
