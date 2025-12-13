# API Documentation

REST API documentation for Pangolin catalog.

## Overview

Pangolin implements the [Apache Iceberg REST Catalog specification](https://github.com/apache/iceberg/blob/main/open-api/rest-catalog-open-api.yaml) with additional extensions for Git-like branching and multi-tenancy.

## Core API

### [API Overview](api_overview.md)
Complete REST API reference covering:
- Namespace operations
- Table operations
- Catalog configuration
- Maintenance endpoints

### [Authentication](authentication.md)
Authentication methods:
- Bearer token authentication
- NO_AUTH mode (development)
- Custom headers (X-Pangolin-Tenant)

## API Categories

### Iceberg REST Catalog (Standard)

**Namespace Operations**:
- `GET /v1/{prefix}/namespaces` - List namespaces
- `POST /v1/{prefix}/namespaces` - Create namespace
- `DELETE /v1/{prefix}/namespaces/{namespace}` - Delete namespace

**Table Operations**:
- `GET /v1/{prefix}/namespaces/{namespace}/tables` - List tables
- `POST /v1/{prefix}/namespaces/{namespace}/tables` - Create table
- `GET /v1/{prefix}/namespaces/{namespace}/tables/{table}` - Load table
- `POST /v1/{prefix}/namespaces/{namespace}/tables/{table}` - Update table
- `DELETE /v1/{prefix}/namespaces/{namespace}/tables/{table}` - Delete table

**Credential Vending**:
- `GET /v1/{prefix}/namespaces/{namespace}/tables/{table}/credentials` - Get credentials

### Pangolin Extensions

**Branch Management**:
- `GET /api/v1/branches` - List branches
- `POST /api/v1/branches` - Create branch
- `POST /api/v1/branches/merge` - Merge branches

**Tag Management**:
- `GET /api/v1/tags` - List tags
- `POST /api/v1/tags` - Create tag
- `DELETE /api/v1/tags/{name}` - Delete tag

**Warehouse Management**:
- `GET /api/v1/warehouses` - List warehouses
- `POST /api/v1/warehouses` - Create warehouse
- `GET /api/v1/warehouses/{name}` - Get warehouse

**Catalog Management**:
- `GET /api/v1/catalogs` - List catalogs
- `POST /api/v1/catalogs` - Create catalog
- `GET /api/v1/catalogs/{name}` - Get catalog

## Contents

| Document | Description |
|----------|-------------|
| [api_overview.md](api_overview.md) | Complete REST API reference |
| [authentication.md](authentication.md) | Authentication methods and setup |

## Quick Examples

### Create Namespace
```bash
curl -X POST http://localhost:8080/v1/analytics/namespaces \
  -H "Content-Type: application/json" \
  -d '{"namespace": ["my_namespace"]}'
```

### Create Table
```bash
curl -X POST http://localhost:8080/v1/analytics/namespaces/my_ns/tables \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_table",
    "schema": {...}
  }'
```

### Create Warehouse
```bash
curl -X POST http://localhost:8080/api/v1/warehouses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_warehouse",
    "storage_config": {
      "type": "s3",
      "bucket": "my-bucket",
      "access_key_id": "...",
      "secret_access_key": "..."
    }
  }'
```

## See Also

- [Getting Started](../getting-started/) - Setup and configuration
- [Features](../features/) - Advanced features
- [Storage](../storage/) - Storage configuration
