# Asset Management

Beyond standard Iceberg tables, Pangolin supports managing other asset types like Views, Materialized Views, Functions, and Procedures.

## Supported Assets
- **Iceberg Table**: Standard Iceberg table.
- **View**: A logical view definition.
- **Materialized View**: A pre-computed view.
- **Function**: A user-defined function.
- **Procedure**: A stored procedure.

## API Endpoints

### Create View
`POST /v1/{prefix}/namespaces/{namespace}/views`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`

**Body:**
```json
{
  "name": "my_view",
  "location": "s3://warehouse/views/my_view",
  "properties": {
    "sql": "SELECT * FROM source_table"
  }
}
```

### Get View
`GET /v1/{prefix}/namespaces/{namespace}/views/{view}`

**Headers:**
- `Authorization`: `Bearer <token>`
- `X-Pangolin-Tenant`: `<Tenant-ID>`

**Response:**
```json
{
  "name": "my_view",
  "location": "s3://warehouse/views/my_view",
  "properties": {
    "sql": "SELECT * FROM source_table"
  }
}
```

### Rename Asset
`POST /v1/{prefix}/tables/rename`

Works for both Tables and Views.

**Body:**
```json
{
  "source": {
    "namespace": ["ns1"],
    "name": "old_name"
  },
  "destination": {
    "namespace": ["ns1"],
    "name": "new_name"
  }
}
```

### Update Asset
`POST /v1/{prefix}/namespaces/{namespace}/tables/{table}` (Tables)
`POST /v1/{prefix}/namespaces/{namespace}/views/{view}` (Views)

### Delete Asset
`DELETE /v1/{prefix}/namespaces/{namespace}/tables/{asset_name}`

> [!TIP]
> While the Iceberg REST specification often separates Table and View endpoints for creation and retrieval, deletion is typically performed via the generic resource path. Pangolin's `delete_table` handler internally handles any asset type tracked by the catalog.

### Rename Asset
`POST /v1/{prefix}/tables/rename`

Works for both Tables and Views.
