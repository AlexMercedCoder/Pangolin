# Pangolin CLI Documentation

## Overview

The Pangolin CLI provides command-line tools for managing your Pangolin data catalog infrastructure.

## CLI Tools

- **`pangolin-admin`** - Administrative tool for managing tenants, warehouses, catalogs, users, and permissions
- **`pangolin-user`** - User-focused tool for data discovery and catalog operations

## Quick Start

### Installation

```bash
# Build from source
cd pangolin
cargo build --release --bin pangolin-admin

# The binary will be at: target/release/pangolin-admin
```

### First Steps

```bash
# 1. Login as root
pangolin-admin login --username admin --password password

# 2. Create a tenant
pangolin-admin create-tenant --name my-company \
  --admin-username tenant-admin \
  --admin-password secure-password

# 3. Login as tenant admin
pangolin-admin login --username tenant-admin --password secure-password

# 4. Create a warehouse
pangolin-admin create-warehouse my-warehouse --type s3

# 5. Create a catalog
pangolin-admin create-catalog my-catalog --warehouse my-warehouse
```

## Documentation Index

### Getting Started
- [Installation & Setup](./installation.md) - Installing and configuring the CLI
- [Authentication](./authentication.md) - Login methods and credential management

### Core Features
- **[Warehouse Management](./warehouse-management.md)** - Creating and managing S3, Azure, and GCS warehouses
- [Catalog Management](./catalog-management.md) - Managing catalogs and namespaces
- [User Management](./user-management.md) - Creating and managing users
- [Permission Management](./permissions.md) - RBAC and access control

### Advanced Features
- [Federated Catalogs](./federated-catalogs.md) - Connecting to external Iceberg catalogs
- [Token Management](./tokens.md) - API tokens and service users
- [Audit Logging](./audit-logging.md) - Viewing and analyzing audit events
- [Merge Operations](./merge-operations.md) - Managing branch merges

### Reference
- [Command Reference](./command-reference.md) - Complete command listing
- [Configuration](./configuration.md) - CLI configuration files
- [Docker Usage](./docker-usage.md) - Running CLI in Docker

## Common Tasks

### Warehouse Operations

```bash
# Create S3 warehouse
pangolin-admin create-warehouse prod-s3 --type s3 \
  --bucket my-bucket \
  --access-key AKIA... \
  --secret-key ... \
  --region us-east-1

# Create Azure warehouse
pangolin-admin create-warehouse prod-azure --type azure

# Create GCS warehouse
pangolin-admin create-warehouse prod-gcs --type gcs

# List warehouses
pangolin-admin list-warehouses

# Update warehouse credentials
pangolin-admin update-warehouse --id <uuid>

# Delete warehouse
pangolin-admin delete-warehouse my-warehouse
```

### Catalog Operations

```bash
# Create catalog
pangolin-admin create-catalog my-catalog --warehouse my-warehouse

# List catalogs
pangolin-admin list-catalogs

# Delete catalog
pangolin-admin delete-catalog my-catalog
```

### User Management

```bash
# Create user
pangolin-admin create-user john.doe \
  --email john@example.com \
  --role tenant-user

# List users
pangolin-admin list-users

# Update user
pangolin-admin update-user --id <uuid> --active false
```

## Interactive Mode

The CLI supports an interactive REPL mode:

```bash
# Start interactive mode
pangolin-admin

# You'll see a prompt:
(admin:username@tenant)> 

# Type commands without the 'pangolin-admin' prefix:
(admin:username@tenant)> list-warehouses
(admin:username@tenant)> create-catalog my-catalog --warehouse my-warehouse
(admin:username@tenant)> exit
```

## Environment Variables

```bash
# Set Pangolin API URL
export PANGOLIN_URL="http://localhost:8080"

# Set tenant context
export PANGOLIN_TENANT="my-tenant"

# Use in commands
pangolin-admin list-warehouses
```

## Configuration Profiles

The CLI saves authentication state in `~/.pangolin/config.json`:

```json
{
  "base_url": "http://localhost:8080",
  "username": "admin",
  "token": "...",
  "tenant_id": "...",
  "tenant_name": "my-tenant"
}
```

Use multiple profiles:

```bash
# Use specific profile
pangolin-admin --profile production list-warehouses

# Switch profiles
pangolin-admin --profile staging login --username admin
```

## Getting Help

```bash
# General help
pangolin-admin help

# Command-specific help
pangolin-admin create-warehouse --help

# List all commands
pangolin-admin help | grep "  "
```

## Troubleshooting

### Connection Issues

```bash
# Check API is running
curl http://localhost:8080/health

# Verify URL
echo $PANGOLIN_URL

# Test with explicit URL
pangolin-admin --url http://localhost:8080 list-tenants
```

### Authentication Issues

```bash
# Clear saved credentials
rm ~/.pangolin/config.json

# Login again
pangolin-admin login --username admin --password password
```

### Permission Issues

```bash
# Check current user context
pangolin-admin list-users  # Shows current user

# Verify tenant context
# Root users must use 'use' command to switch tenants
pangolin-admin use my-tenant
```

## Examples

See individual documentation pages for detailed examples:

- [Warehouse Management Examples](./warehouse-management.md#examples)
- [Multi-Cloud Setup](./warehouse-management.md#multi-cloud-strategy)
- [Federated Catalog Setup](./federated-catalogs.md#examples)

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/pangolin/issues)
- **Documentation**: [Full Documentation](../../README.md)
- **Architecture**: [Architecture Docs](../architecture/)

## Version

Current CLI version: 0.1.0

For API compatibility, see [API Documentation](../api/).
