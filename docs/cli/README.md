# Pangolin CLI Documentation

Command-line tools for managing and interacting with Pangolin catalogs.

## Overview

Pangolin provides two CLI tools:
- **pangolin-admin** - Administrative operations (requires elevated permissions)
- **pangolin-user** - User operations (standard user access)

## Getting Started

- [CLI Overview](overview.md) - Introduction to Pangolin CLI tools
- [Configuration](configuration.md) - Setting up CLI configuration
- [Docker Usage](docker-usage.md) - Using CLI via Docker container

## Admin CLI (`pangolin-admin`)

### Core Operations
- [Admin Overview](admin.md) - Complete admin CLI reference
- [Tenants](admin-tenants.md) - Manage tenants
- [Warehouses](admin-warehouses.md) - Manage warehouses
- [Catalogs](admin-catalogs.md) - Manage catalogs
- [Users](admin-users.md) - Manage users

### Advanced Features
- [Permissions](admin-permissions.md) - Grant and revoke permissions
- [Token Management](admin-token-management.md) - Generate and manage JWT tokens
- [Service Users](admin-service-users.md) - Manage service accounts
- [Federated Catalogs](admin-federated-catalogs.md) - Configure catalog federation
- [Merge Operations](admin-merge-operations.md) - Merge branches and resolve conflicts
- [Update Operations](admin-update-operations.md) - Update existing resources
- [Business Metadata](admin-metadata.md) - Manage business metadata
- [Audit Logging](admin-audit-logging.md) - Query audit logs
- [Optimization Commands](admin-optimization-commands.md) - Stats, Search, and Validation

## User CLI (`pangolin-user`)

- [User Overview](user.md) - Complete user CLI reference
- [Discovery](user-discovery.md) - Search and discover datasets
- [Access Requests](user-access.md) - Request access to datasets
- [Tokens](user-tokens.md) - Manage personal access tokens
- [Branches](user-branches.md) - Work with catalog branches
- [Tags](user-tags.md) - Manage dataset tags

## Installation

### Pre-compiled Binaries

Download binaries for your platform from [GitHub Releases](https://github.com/AlexMercedCoder/Pangolin/releases/latest):

- **Linux (x86_64)**: `pangolin-admin`, `pangolin-user`
- **macOS (Intel & ARM)**: `pangolin-admin`, `pangolin-user`
- **Windows (x86_64)**: `pangolin-admin.exe`, `pangolin-user.exe`

### Docker

```bash
docker pull alexmerced/pangolin-cli:0.1.0

# Run admin commands
docker run --rm alexmerced/pangolin-cli:0.1.0 pangolin-admin --help

# Run user commands
docker run --rm alexmerced/pangolin-cli:0.1.0 pangolin-user --help
```

See [Docker Usage Guide](docker-usage.md) for detailed instructions.

### Build from Source

```bash
cd pangolin/
cargo build --release --bin pangolin-admin --bin pangolin-user

# Binaries will be in target/release/
./target/release/pangolin-admin --help
./target/release/pangolin-user --help
```

## Quick Examples

### Admin Examples

```bash
# Create a tenant
pangolin-admin create-tenant --name "acme"

# Create a warehouse
pangolin-admin create-warehouse --name "prod_warehouse" --storage-type s3

# Create a catalog
pangolin-admin create-catalog --name "analytics" --warehouse-id <warehouse-id>

# Grant permissions
pangolin-admin grant-permission --user-id <user-id> --action read --scope "catalog:analytics"
```

### User Examples

```bash
# Search for datasets
pangolin-user search "customers"

# Request access
pangolin-user request-access --table "analytics.sales.customers"

# Create a branch
pangolin-user create-branch --catalog analytics --branch dev --from main

# Generate a token
pangolin-user generate-token --expiry 30d
```

## Configuration

CLI tools can be configured via:

1. **Environment Variables**:
   ```bash
   export PANGOLIN_API_URL=http://localhost:8080
   export PANGOLIN_TOKEN=your_jwt_token
   ```

2. **Configuration File** (`~/.pangolin/config.json`):
   ```json
   {
     "api_url": "http://localhost:8080",
     "token": "your_jwt_token"
   }
   ```

3. **Command-line Flags**:
   ```bash
   pangolin-admin --api-url http://localhost:8080 --token <token> list-tenants
   ```

See [Configuration Guide](configuration.md) for details.

## Related Documentation

- [Getting Started](../getting-started/getting_started.md)
- [API Reference](../api/README.md)
- [Deployment Guide](../../deployment_assets/README.md)
- [Binary Installation](../../deployment_assets/bin/README.md)
