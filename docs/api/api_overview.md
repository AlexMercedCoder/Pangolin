# API Reference Overview

Pangolin exposes a multi-tenant REST API split into three core functional areas: The Standard Iceberg REST API, the Pangolin Management API, and the Identity/Security API.

---

## 🔐 Authentication & Identity
*Base Path: `/api/v1/`*

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `users/login` | POST | Authenticate and receive a JWT session. |
| `tokens` | POST | Generate a long-lived JWT token for a specific user. |
| `auth/revoke` | POST | Invalidate current user token. |
| `service-users` | GET/POST | Manage programmatic API identities. |
| `service-users/{id}/rotate` | POST | Rotate API key for a service user. |

---

## 📋 Standard Iceberg API (REST Catalog)
*Base Path: `/v1/`*

Pangolin is 100% compliant with the [Apache Iceberg REST Specification](https://github.com/apache/iceberg/blob/main/open-api/rest-catalog-open-api.yaml).

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `config` | GET | Get client configuration. |
| `{prefix}/namespaces` | GET/POST | Manage catalog namespaces. |
| `{prefix}/namespaces/{ns}/tables` | GET/POST | Manage tables (Standard Iceberg). |
| `{prefix}/namespaces/{ns}/tables/{table}` | GET/POST | Table metadata and schema evolution. |

> [!TIP]
> **Branching**: Use the `@` suffix in the table name (e.g., `my_table@dev`) to redirect Iceberg operations to a specific Pangolin branch.

---

## 🏗️ Management & Data Governance
*Base Path: `/api/v1/`*

### 1. Multi-Tenancy (Root Only)
| Endpoint | Method | Use Case |
| :--- | :--- | :--- |
| `tenants` | GET/POST/PUT | Onboard new organizations to the platform. |
| `config/settings` | GET/PUT | Manage global platform defaults. |

### 2. Infrastructure & Data (Tenant Admin)
| Endpoint | Method | Use Case |
| :--- | :--- | :--- |
| `warehouses` | GET/POST/PUT | Configure cloud storage containers (S3/GCS/Azure). |
| `catalogs` | GET/POST/PUT | Create local or federated Iceberg catalogs. |
| `audit` | GET/POST | Query standard or count audit logs. |
| `maintenance` | POST | Trigger snapshot expiration or orphan file removal. |

### 3. Branching & Merging
| Endpoint | Method | Use Case |
| :--- | :--- | :--- |
| `branches` | GET/POST | List or create feature/ingest branches. |
| `branches/merge` | POST | Initiate a Git-like merge between two branches. |
| `merge-operations` | GET | Track progress of active or past merges. |
| `conflicts/{id}/resolve`| POST | Apply conflict resolution strategies (Source Wins/Target Wins). |

### 4. Search & Optimization
| Endpoint | Method | Use Case |
| :--- | :--- | :--- |
| `search` | GET | Unified search across Catalogs, Namespaces, and Tables. |
| `search/assets` | GET | Optimized lookup for specific tables/views by name. |
| `validate/names` | POST | Check if an asset name is valid and available. |
| `bulk/assets/delete` | POST | Bulk deletion of assets. |


---

## 🧬 OAuth 2.0 Flow
Pangolin supports Google, GitHub, and Microsoft OAuth.

- **Initiate**: `GET /oauth/authorize/{provider}`
- **Callback**: `GET /oauth/callback/{provider}`

---

## 🚀 Quick Integration
To use the API, ensure you provide the `Authorization` header and the `X-Pangolin-Tenant` header (unless in `NO_AUTH` mode).

```bash
curl -X GET http://localhost:8080/api/v1/catalogs \
  -H "Authorization: Bearer <JWT_TOKEN>" \
  -H "X-Pangolin-Tenant: <TENANT_UUID>"
```
