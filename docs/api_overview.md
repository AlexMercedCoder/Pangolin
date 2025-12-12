# API Overview

Pangolin exposes two sets of APIs: the standard Iceberg REST API and the extended Pangolin API.

## Iceberg REST API
Compliant with the Apache Iceberg REST Catalog Specification.

### Catalog Configuration
- `GET /v1/config`: Get catalog configuration.

### Table Management
- `POST /v1/{prefix}/namespaces/{namespace}/tables`: Create a table.
- `GET /v1/{prefix}/namespaces/{namespace}/tables/{table}`: Load a table.
- `POST /v1/{prefix}/namespaces/{namespace}/tables/{table}`: Update/Commit to a table.
- `DELETE /v1/{prefix}/namespaces/{namespace}/tables/{table}`: Delete a table.
- `POST /v1/{prefix}/tables/rename`: Rename a table.
- `POST /v1/{prefix}/namespaces/{namespace}/tables/{table}/metrics`: Report metrics.

### Namespace Management
- `GET /v1/{prefix}/namespaces`: List namespaces.
- `POST /v1/{prefix}/namespaces`: Create a namespace.
- `DELETE /v1/{prefix}/namespaces/{namespace}`: Delete a namespace.
- `POST /v1/{prefix}/namespaces/{namespace}/properties`: Update namespace properties.

**Branching Support**:
Append `@branchName` to the table name or namespace to target a specific branch.
- Example: `GET .../tables/my_table@dev`

## Pangolin Extended API
Additional endpoints for features not covered by the Iceberg spec.

### Tenant Management
- `POST /api/v1/tenants`: Create a new tenant.
- `GET /api/v1/tenants`: List all tenants.
- `GET /api/v1/tenants/:id`: Get a specific tenant.

### Warehouse Management
- `POST /api/v1/warehouses`: Create a new warehouse.
- `GET /api/v1/warehouses`: List all warehouses.
- `GET /api/v1/warehouses/:name`: Get a specific warehouse.

### Asset Management (Views)
- `POST /v1/{prefix}/namespaces/{namespace}/views`: Create a new view.
- `GET /v1/{prefix}/namespaces/{namespace}/views/{view}`: Get a specific view.

### Branch Management
- `GET /api/v1/branches`: List branches for a catalog.
- `POST /api/v1/branches`: Create a new branch.
- `GET /api/v1/branches/:name`: Get branch details.
- `POST /api/v1/branches/merge`: Merge a source branch into a target branch.
