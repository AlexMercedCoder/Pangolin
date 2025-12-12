# API Overview

Pangolin exposes two sets of APIs: the standard Iceberg REST API and the extended Pangolin API.

## Iceberg REST API
Compliant with the Apache Iceberg REST Catalog Specification.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/v1/config` | Get catalog configuration |
| `GET` | `/v1/{prefix}/namespaces` | List namespaces |
| `POST` | `/v1/{prefix}/namespaces` | Create namespace |
| `GET` | `/v1/{prefix}/namespaces/{namespace}/tables` | List tables |
| `POST` | `/v1/{prefix}/namespaces/{namespace}/tables` | Create table |
| `GET` | `/v1/{prefix}/namespaces/{namespace}/tables/{table}` | Load table |

**Branching Support**:
Append `@branchName` to the table name or namespace to target a specific branch.
- Example: `GET .../tables/my_table@dev`

## Pangolin Extended API
Additional endpoints for features not covered by the Iceberg spec.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/branches` | List all branches |
| `POST` | `/api/v1/branches` | Create a new branch |
| `GET` | `/api/v1/branches/{name}` | Get branch details |
