# Documentation

Welcome to the Pangolin documentation!

## 📖 Documentation Structure

### [Getting Started](getting-started/)
Setup, configuration, and deployment guides.

### [Storage](warehouse/)
Multi-cloud storage configuration (S3, Azure, GCS).

### [API](api/)
REST API reference and authentication.

### [Features](features/)
Advanced features like branching, time travel, and credential vending.

### [Research](../planning/research/)
Implementation plans and research notes.

## Quick Navigation

**New to Pangolin?** Start with the [Getting Started Guide](getting-started/getting_started.md)

**Setting up storage?** Check the [Storage Documentation](warehouse/)

**Using PyIceberg?** See [PyIceberg Integration](features/pyiceberg_testing.md)

**API Reference?** Browse the [API Documentation](api/)

## Documentation Categories

| Category | Description | Key Documents |
|----------|-------------|---------------|
| **Getting Started** | Setup and deployment | [Quick Start](getting-started/getting_started.md), [Evaluating](getting-started/evaluating-pangolin.md), [Deployment](getting-started/deployment.md) |
| **Storage** | Cloud storage configuration | [S3](warehouse/s3.md), [Azure](warehouse/azure.md), [GCS](warehouse/gcs.md) |
| **API** | REST API reference | [API Overview](api/api_overview.md), [Authentication](api/authentication.md) |
| **Features** | Advanced capabilities | [Branching](features/branch_management.md), [Time Travel](features/time_travel.md), [PyIceberg](features/pyiceberg_testing.md) |

## Feature Highlights

- ✅ **Multi-Cloud Storage** - S3, Azure Blob Storage, Google Cloud Storage
- ✅ **Git-Like Branching** - Isolated development environments
- ✅ **Time Travel** - Query historical data via snapshots
- ✅ **Credential Vending** - Automatic credential management
- ✅ **PyIceberg Compatible** - Full Apache Iceberg REST Catalog spec
- ✅ **Multi-Tenant** - Isolated tenants with separate data

## See Also

- [Main README](../README.md) - Project overview
- [Architecture](../architecture.md) - System architecture
- [Test Results](../tests/pyiceberg/TEST_RESULTS.md) - PyIceberg test results
