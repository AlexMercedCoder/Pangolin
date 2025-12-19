# API Overview

Pangolin provides a rich set of APIs for catalog management, branching, merging, and authentication.

## Authentication

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/users/login` | POST | Authenticate and receive a JWT token. |
| `/api/v1/users/logout` | POST | Invalidate current session (optional). |
| `/api/v1/users/me` | GET | Get current user information. |
| `/api/v1/tokens` | POST | Generate a long-lived JWT token for a specific tenant/user. |

## Iceberg REST API

Pangolin implements the standard Apache Iceberg REST Catalog API.

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/v1/config` | GET | Get Iceberg client configuration. |
| `/v1/{tenant}/config` | GET | Get tenant-specific Iceberg client configuration. |
| `/v1/{prefix}/namespaces` | GET/POST | List and create namespaces. |
| `/v1/{prefix}/namespaces/{namespace}/tables` | GET/POST | List and create tables. |
| `/v1/{prefix}/namespaces/{namespace}/tables/{table}` | GET/POST/DELETE | Manage table metadata and snapshots. |

**Note**: Branching is supported via the `table@branch` syntax (e.g., `GET .../tables/my_table@dev`).

## Pangolin Extended APIs

### Branch Operations

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/branches` | GET/POST | List all branches or create a new branch. |
| `/api/v1/branches/merge` | POST | Initiate a merge from a source branch to a target branch. |
| `/api/v1/branches/{name}/commits` | GET | List commit history for a specific branch. |

### Tag Management

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/tags` | GET/POST | List all tags or create a new tag. |
| `/api/v1/tags/{name}` | GET/DELETE | View or delete a specific tag. |

### Merge Operations (Conflict Resolution)

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/catalogs/{catalog}/merge-operations` | GET | List merge operations for a catalog. |
| `/api/v1/merge-operations/{id}` | GET | Get merge operation details and status. |
| `/api/v1/merge-operations/{id}/conflicts` | GET | List conflicts for a merge operation. |
| `/api/v1/conflicts/{id}/resolve` | POST | Resolve a specific conflict with a strategy. |
| `/api/v1/merge-operations/{id}/complete` | POST | Complete a merge after resolving all conflicts. |
| `/api/v1/merge-operations/{id}/abort` | POST | Abort a pending merge operation. |

### Management CRUD

| Entity | Endpoints | Methods |
| :--- | :--- | :--- |
| **Tenants** | `/api/v1/tenants` | GET, POST, PUT, DELETE |
| **Warehouses** | `/api/v1/warehouses` | GET, POST, PUT, DELETE |
| **Catalogs** | `/api/v1/catalogs` | GET, POST, PUT, DELETE |
| **Federated Catalogs** | `/api/v1/federated-catalogs` | GET, POST, DELETE, TEST |
| **Users** | `/api/v1/users` | GET, POST, PUT, DELETE |
| **Service Users**| `/api/v1/service-users` | GET, POST, PUT, DELETE, ROTATE |
| **Roles** | `/api/v1/roles` | GET, POST, PUT, DELETE |
| **Permissions** | `/api/v1/permissions` | GET, POST, DELETE |

### Token Management

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/tokens` | POST | Generate a long-lived JWT token for scripts/automation. |
| `/api/v1/auth/revoke` | POST | Revoke current user's token. |
| `/api/v1/auth/revoke/{token_id}` | POST | Revoke a specific token (admin only). |
| `/api/v1/auth/cleanup-tokens` | POST | Clean up expired tokens (admin only). |
| `/api/v1/users/{user_id}/tokens` | GET | List all tokens for a specific user (admin only). |
| `/api/v1/tokens/{token_id}` | DELETE | Delete a specific token by ID (admin only). |

### System Configuration

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/config/settings` | GET | Get system configuration settings (admin only). |
| `/api/v1/config/settings` | PUT | Update system configuration settings (admin only). |

### Federated Catalog Operations

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/federated-catalogs/{name}/sync` | POST | Trigger immediate metadata sync for federated catalog. |
| `/api/v1/federated-catalogs/{name}/stats` | GET | Get sync statistics and status for federated catalog. |

### Data Explorer

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/catalogs/{prefix}/namespaces/tree` | GET | Get hierarchical namespace tree structure for a catalog. |

### OAuth

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/oauth/authorize/{provider}` | GET | Initiate OAuth flow (Google, GitHub, etc.). |
| `/oauth/callback/{provider}` | GET | OAuth callback handler. |

## Auditing

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/v1/audit-logs` | GET | Retrieve audit logs for the current tenant. |

## Other APIs

- **Credential Vending**: `/api/v1/credentials` (GET)
- **S3 Presigning**: `/api/v1/presign` (POST)
- **Business Metadata**: `/api/v1/metadata/search` (POST)
- **Access Requests**: `/api/v1/access-requests` (GET/POST)
- **App Config**: `/api/v1/app-config` (GET)
