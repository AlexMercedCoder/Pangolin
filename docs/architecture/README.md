# Pangolin Architecture

## Overview

Pangolin is a multi-tenant Apache Iceberg REST Catalog with advanced features including federated catalogs, credential vending, fine-grained permissions, and Git-like branching.

## Pangolin Architecture Documentation

This directory contains technical documentation for the Pangolin architecture, core data models, and primary abstraction layers.

## Documentation Index

### 1. [Architecture Overview](./architecture.md)
High-level system design, core components, security logic, and request flow. Start here to understand how Pangolin works.

### 2. [Data Models](./models.md)
Detailed reference for all core structs and enums across the `model`, `user`, `permission`, `business_metadata`, and `audit` domains.

### 3. [CatalogStore Trait](./catalog-store-trait.md)
Reference for the primary storage abstraction layer, covering multi-tenant isolation and metadata lifecycle operations.

### 4. [Signer Trait](./signer-trait.md)
Documentation of the cloud credential vending logic, used to provide temporary access to S3, GCS, and Azure storage.

## Target Audience
This documentation is intended for:
- **Core Contributors**: Developers extending the backend or adding new storage engines.
- **Security Auditors**: Engineers reviewing tenant isolation and credential vending patterns.
- **Enterprise Integrators**: Teams deploying Pangolin in custom cloud environments.

### Quick Links

- [API Documentation](../api/README.md)
- [Utilities](../utilities/README.md)
- [Planning Documents](../../planning/)

## System Components

### 1. API Layer (`pangolin_api`)

HTTP REST API implementing the Apache Iceberg REST Catalog specification plus Pangolin-specific extensions.

**Key Features**:
- Iceberg REST Catalog endpoints
- Multi-tenant isolation
- JWT authentication
- Fine-grained permissions
- OAuth integration
- Swagger UI documentation

**Handlers**:
- Tenant management
- Warehouse management
- Catalog management (local + federated)
- Namespace & table operations
- User & permission management
- Branch, tag, and merge operations
- Business metadata & discovery
- Service user management

### 2. Core Layer (`pangolin_core`)

Shared data models and business logic.

**Modules**:
- `model.rs` - Core Iceberg and infrastructure models
- `user.rs` - User authentication models
- `permission.rs` - RBAC models
- `business_metadata.rs` - Discovery and metadata models
- `audit.rs` - Audit logging models

### 3. Storage Layer (`pangolin_store`)

Pluggable storage backend abstraction.

**Trait**: `CatalogStore`
- Defines all storage operations
- Async interface
- Multi-tenant aware
- Error handling

**Implementations**:
- **MemoryStore** - In-memory (testing/development)
- **PostgresStore** - PostgreSQL (production)
- **MongoStore** - MongoDB (alternative)
- **SqliteStore** - SQLite (embedded)

**Credential Vending**:
- **Signer Trait** - Abstraction for credential generation
- AWS S3 (STS role assumption)
- Azure Blob (SAS tokens)
- GCP Storage (service account impersonation)

### 4. CLI Tools

- **`pangolin_cli_admin`** - Administrative CLI for tenant/user management
- **`pangolin_cli_catalog`** - Catalog operations CLI

## Key Architectural Patterns

### Multi-Tenancy

Complete tenant isolation at all layers:
```
Tenant → Warehouses → Catalogs → Namespaces → Tables
```

Every operation requires a `tenant_id` for isolation.

### Trait-Based Abstraction

Core abstractions use Rust traits:
- `CatalogStore` - Storage operations
- `Signer` - Credential vending
- Enables multiple implementations
- Facilitates testing with mocks

### Async/Await

All I/O operations are async:
- Non-blocking database queries
- Concurrent request handling
- Efficient resource utilization

### Error Handling

Consistent error handling with `anyhow::Result`:
- Propagates errors up the stack
- Provides context with `.context()`
- Converts to HTTP status codes in handlers

## Data Flow

### Table Read Operation

```
1. Client → API: GET /v1/{prefix}/namespaces/{ns}/tables/{table}
2. API → Auth: Validate JWT token
3. API → Authz: Check READ permission
4. API → Store: Load table metadata
5. Store → S3: Read metadata file
6. API → Signer: Vend credentials
7. API → Client: Return metadata + credentials
8. Client → S3: Direct data access
```

### Table Write Operation

```
1. Client → API: POST /v1/{prefix}/namespaces/{ns}/tables/{table}
2. API → Auth: Validate JWT token
3. API → Authz: Check WRITE permission
4. API → Store: Update metadata location
5. Store → S3: Write new metadata file
6. Store → DB: Update pointer
7. API → Audit: Log write operation
8. API → Client: Return updated metadata
```

## Security Architecture

### Authentication

- **JWT Tokens**: Stateless authentication
- **Service Users**: API key-based auth for services
- **OAuth**: Google, GitHub, Microsoft integration

### Authorization

- **RBAC**: Role-based access control
- **Scoped Permissions**: Tenant/Warehouse/Catalog/Namespace/Asset levels
- **Actions**: Read, Write, Delete, Admin, ManageDiscovery

### Credential Vending

- **Scoped Credentials**: Limited to specific S3 prefixes
- **Time-Limited**: 1-4 hour expiration
- **Audit Logged**: All vending operations tracked

## Scalability

### Horizontal Scaling

- Stateless API servers
- Load balancer distribution
- Shared database backend

### Database Scaling

- Read replicas for queries
- Connection pooling
- Indexed lookups

### Storage Scaling

- S3/Azure/GCS for unlimited data storage
- Metadata in database
- Data files in object storage

## Extensibility

### Adding New Storage Backends

1. Implement `CatalogStore` trait
2. Implement `Signer` trait
3. Add to `pangolin_store/src/lib.rs`
4. Update configuration

### Adding New Endpoints

1. Create handler function
2. Add `#[utoipa::path]` annotation
3. Register in router
4. Add to `openapi.rs`
5. Regenerate OpenAPI spec

### Adding New Models

1. Define struct in `pangolin_core`
2. Add `#[derive(Serialize, Deserialize, ToSchema)]`
3. Update database schema
4. Implement CRUD operations in `CatalogStore`

## Performance Considerations

### Caching

- In-memory caching for frequently accessed metadata
- TTL-based invalidation
- Tenant-aware cache keys

### Connection Pooling

- Database connection pool (default: 10 connections)
- Configurable pool size
- Connection timeout handling

### Batch Operations

- Bulk permission grants
- Batch metadata updates
- Transaction support where available

## Monitoring & Observability

### Logging

- Structured logging with `tracing`
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- Request ID correlation

### Metrics

- Request count
- Response times
- Error rates
- Database query performance

### Audit Trail

- All security-sensitive operations logged
- Immutable audit log
- Queryable via API

## Deployment Architecture

### Development

```
┌─────────────┐
│   API       │
│ (in-memory) │
└─────────────┘
```

### Production

```
┌──────────────┐     ┌──────────────┐
│  Load        │────▶│  API Server  │
│  Balancer    │     │  (N instances)│
└──────────────┘     └──────┬───────┘
                            │
                     ┌──────▼───────┐
                     │  PostgreSQL  │
                     │   Database   │
                     └──────────────┘
                            │
                     ┌──────▼───────┐
                     │   S3/Azure/  │
                     │     GCS      │
                     └──────────────┘
```

## Technology Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Async Runtime**: Tokio
- **Databases**: PostgreSQL, MongoDB, SQLite
- **Object Storage**: AWS S3, Azure Blob, GCP Storage
- **Authentication**: JWT, OAuth 2.0
- **API Documentation**: OpenAPI 3.0 (utoipa)
- **Serialization**: serde (JSON)

## See Also

- [Data Models](./models.md)
- [CatalogStore Trait](./catalog-store-trait.md)
- [Signer Trait](./signer-trait.md)
- [API Documentation](../api/README.md)
- [Deployment Guide](../deployment/README.md)
