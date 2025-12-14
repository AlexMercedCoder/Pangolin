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
| [PostgreSQL](postgresql.md) | ✅ Stable | Production deployments | Yes |
| [MongoDB](mongodb.md) | ✅ Stable | Cloud-native, scalable deployments | Yes |
| [SQLite](sqlite.md) | ✅ Stable | Development, embedded, edge deployments | Yes |

## Quick Comparison

| Feature | PostgreSQL | MongoDB | SQLite |
|---------|-----------|---------|--------|
| **Setup Complexity** | Medium | Medium | Low |
| **Scalability** | High | Very High | Low |
| **Transactions** | ✅ ACID | ✅ ACID | ✅ ACID |
| **Foreign Keys** | ✅ Yes | ❌ No | ✅ Yes |
| **Schema** | Strict SQL | Flexible | Strict SQL |
| **Replication** | ✅ Built-in | ✅ Built-in | ❌ Manual |
| **Cloud Managed** | ✅ RDS, Azure, GCP | ✅ Atlas, Azure, GCP | ❌ No |
| **Resource Usage** | Medium | Medium | Very Low |
| **Multi-Tenant Isolation** | ✅ Excellent | ✅ Excellent | ✅ Excellent |

## Choosing a Backend

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

### Use SQLite When:
- ✅ You're developing locally
- ✅ You need embedded database (no separate server)
- ✅ You're deploying to edge/IoT devices
- ✅ You have low concurrent write requirements
- ✅ You want zero configuration

## Configuration

Set the backend using the `DATABASE_URL` environment variable:

```bash
# PostgreSQL
DATABASE_URL=postgresql://user:password@localhost:5432/pangolin

# MongoDB
DATABASE_URL=mongodb://user:password@localhost:27017/pangolin

# SQLite
DATABASE_URL=sqlite:///path/to/pangolin.db
# Or in-memory for testing:
DATABASE_URL=sqlite::memory:
```

## Next Steps

- [PostgreSQL Setup Guide](postgresql.md)
- [MongoDB Setup Guide](mongodb.md)
- [SQLite Setup Guide](sqlite.md)
- [Detailed Comparison](comparison.md)

## Migration Between Backends

Currently, Pangolin does not provide automated migration tools between backends. If you need to migrate:

1. Export metadata from source backend (custom script)
2. Transform to target backend format
3. Import into target backend

> **Tip**: Start with SQLite for development, then migrate to PostgreSQL or MongoDB for production.
