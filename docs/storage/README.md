# Storage Configuration

Pangolin supports multiple cloud storage providers for storing Iceberg tables.

## Supported Storage Providers

- **AWS S3** - Amazon S3 and S3-compatible storage (MinIO)
- **Azure Blob Storage** - Azure ADLS Gen2
- **Google Cloud Storage** - GCS buckets
- **MongoDB** - MongoDB as metadata store
- **PostgreSQL** - PostgreSQL as metadata store

## Object Storage

### [AWS S3](storage_s3.md)
- S3-compatible storage (AWS S3, MinIO, etc.)
- Static credentials or STS
- Tested and production-ready ✅

### [Azure Blob Storage](storage_azure.md)
- Azure ADLS Gen2 support
- Account key or OAuth2 authentication
- Full credential vending support ✅

### [Google Cloud Storage](storage_gcs.md)
- GCS bucket support
- Service account or OAuth2 authentication
- Full credential vending support ✅

## Metadata Storage

### [MongoDB](storage_mongo.md)
- MongoDB as catalog metadata store
- Document-based storage

### [PostgreSQL](storage_postgres.md)
- PostgreSQL as catalog metadata store
- Relational storage

## Quick Comparison

| Provider | Status | Auth Methods | Tested |
|----------|--------|--------------|--------|
| AWS S3 | ✅ Production Ready | Static, STS | Yes (MinIO) |
| Azure | ✅ Implemented | Account Key, OAuth2 | Unit Tests |
| GCS | ✅ Implemented | Service Account, OAuth2 | Unit Tests |
| MongoDB | 📝 Documented | Connection String | No |
| PostgreSQL | 📝 Documented | Connection String | No |

## Credential Vending

All object storage providers support automatic credential vending:

1. Create a warehouse with storage credentials
2. Link a catalog to the warehouse
3. PyIceberg automatically receives credentials

See individual storage guides for detailed configuration.

## Contents

| Document | Description |
|----------|-------------|
| [storage_s3.md](storage_s3.md) | AWS S3 and S3-compatible storage |
| [storage_azure.md](storage_azure.md) | Azure Blob Storage (ADLS Gen2) |
| [storage_gcs.md](storage_gcs.md) | Google Cloud Storage |
| [storage_mongo.md](storage_mongo.md) | MongoDB metadata storage |
| [storage_postgres.md](storage_postgres.md) | PostgreSQL metadata storage |

## See Also

- [Warehouse Management](../features/warehouse_management.md) - Creating and managing warehouses
- [Security & Credential Vending](../features/security_vending.md) - Credential vending details
- [PyIceberg Testing](../features/pyiceberg_testing.md) - Testing with PyIceberg
