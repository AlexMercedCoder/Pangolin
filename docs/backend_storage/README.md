# Backend Storage Options

Pangolin supports multiple backend storage options for persisting catalog metadata. Choose the backend that best fits your deployment requirements.

## Overview

Backend storage is where Pangolin stores **catalog metadata** including:
- Tenant information
- Warehouse configurations
- Catalog definitions
- Namespace hierarchies
- Asset (table/view) metadata
- Branch and tag information
- Audit logs

> **Note**: Backend storage is separate from **warehouse storage** (S3, Azure, GCS) which stores the actual data files.

## Available Backends

| Backend | Status | Best For | Production Ready |
|---------|--------|----------|------------------|
| [In-Memory](memory.md) | ✅ Stable | Development, testing, CI/CD | No (ephemeral) |
| [SQLite](sqlite.md) | ✅ Stable | Development, embedded, edge deployments | Yes |
| [PostgreSQL](postgresql.md) | ✅ Stable | Production deployments | Yes |
| [MongoDB](mongodb.md) | ✅ Stable | Cloud-native, scalable deployments | Yes |

## Quick Comparison

| Feature | In-Memory | SQLite | PostgreSQL | MongoDB |
|---------|-----------|--------|------------|---------|
| **Setup Complexity** | None | Low | Medium | Medium |
| **Scalability** | Low | Low | High | Very High |
| **Transactions** | ✅ ACID | ✅ ACID | ✅ ACID | ✅ ACID |
| **Foreign Keys** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No |
| **Schema** | Strict | Strict SQL | Strict SQL | Flexible |
| **Replication** | ❌ No | ❌ Manual | ✅ Built-in | ✅ Built-in |
| **Cloud Managed** | ❌ No | ❌ No | ✅ RDS, Azure, GCP | ✅ Atlas, Azure, GCP |
| **Resource Usage** | Very Low | Very Low | Medium | Medium |
| **Persistence** | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| **Multi-Tenant Isolation** | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Excellent |

## Choosing a Backend

### Use In-Memory When:
- ✅ You're developing locally
- ✅ You're running tests (unit or integration)
- ✅ You need instant setup with zero configuration
- ✅ You're prototyping or learning
- ✅ Data persistence is not required
- ✅ You're running in CI/CD pipelines

### Use SQLite When:
- ✅ You're developing locally and need persistence
- ✅ You need embedded database
- ✅ You're deploying to edge/IoT devices
- ✅ You want zero configuration with persistence
- ✅ You have low concurrent write needs
- ✅ You want minimal resource usage

### Use PostgreSQL When:
- ✅ You need a proven, battle-tested SQL database
- ✅ You want strong consistency and ACID guarantees
- ✅ You're deploying to traditional infrastructure
- ✅ You need complex queries and joins
- ✅ You want managed cloud options (RDS, Azure Database, Cloud SQL)

### Use MongoDB When:
- ✅ You prefer document-based storage
- ✅ You need horizontal scalability
- ✅ You're building cloud-native applications
- ✅ You want flexible schema evolution
- ✅ You're already using MongoDB in your stack

## Configuration

Set the backend using the `DATABASE_URL` environment variable:

```bash
# In-Memory (default - no DATABASE_URL needed)
# Just don't set DATABASE_URL

# SQLite
DATABASE_URL=sqlite:///path/to/pangolin.db

# PostgreSQL
DATABASE_URL=postgresql://user:password@localhost:5432/pangolin

# MongoDB
DATABASE_URL=mongodb://user:password@localhost:27017/pangolin
```

## Next Steps

- [In-Memory Setup Guide](memory.md)
- [SQLite Setup Guide](sqlite.md)
- [PostgreSQL Setup Guide](postgresql.md)
- [MongoDB Setup Guide](mongodb.md)
- [Detailed Comparison](comparison.md)

## Migration Between Backends

Currently, Pangolin does not provide automated migration tools between backends. If you need to migrate:

1. Export metadata from source backend (custom script)
2. Transform to target backend format
3. Import into target backend

> **Tip**: Start with SQLite for development, then migrate to PostgreSQL or MongoDB for production.
