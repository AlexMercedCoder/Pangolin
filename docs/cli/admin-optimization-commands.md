# CLI Optimization Commands

This guide covers the 5 new optimization commands added to `pangolin-admin` for improved performance and usability.

## Overview

These commands provide efficient access to dashboard statistics, catalog summaries, asset search, bulk operations, and name validation without requiring multiple API calls.

---

## Commands

### 1. `stats` - Dashboard Statistics

Get comprehensive dashboard statistics in a single API call.

**Usage**:
```bash
pangolin-admin stats
```

**Output**:
```
Dashboard Statistics (tenant)
  Catalogs:    6
  Warehouses:  2
  Namespaces:  15
  Tables:      42
  Users:       5
  Branches:    8
```

**Scope**: Results are scoped based on your role:
- **Root**: System-wide statistics
- **Tenant Admin**: Tenant-specific statistics  
- **Tenant User**: User-accessible resources only

---

### 2. `catalog-summary` - Catalog Summary

Get detailed summary for a specific catalog.

**Usage**:
```bash
pangolin-admin catalog-summary <catalog_name>
```

**Example**:
```bash
pangolin-admin catalog-summary production
```

**Output**:
```
Catalog: production
  Namespaces: 12
  Tables:     45
  Branches:   3
  Storage:    s3://my-bucket/warehouse/production
```

---

### 3. `search` - Asset Search

Search for assets across catalogs with optional filtering.

**Usage**:
```bash
pangolin-admin search <query> [--catalog <name>] [--limit <n>]
```

**Examples**:
```bash
# Basic search
pangolin-admin search "sales"

# Search within specific catalog
pangolin-admin search "customer" --catalog production

# Limit results
pangolin-admin search "analytics" --limit 20
```

**Output**:
```
Found 5 results:
  production.analytics.sales_data
  production.analytics.customer_metrics
  production.default.sales_summary
  staging.analytics.sales_test
  dev.analytics.sales_dev
```

**Features**:
- Full-text search across asset names
- Catalog filtering
- Pagination support
- URL encoding for special characters

---

### 4. `bulk-delete` - Bulk Asset Deletion

Delete multiple assets in a single operation.

**Usage**:
```bash
pangolin-admin bulk-delete --ids <uuid1,uuid2,...> [--confirm]
```

**Examples**:
```bash
# Interactive confirmation
pangolin-admin bulk-delete --ids "123e4567-e89b-12d3-a456-426614174000,987fcdeb-51a2-43f7-8b9c-0123456789ab"

# Skip confirmation
pangolin-admin bulk-delete --ids "uuid1,uuid2" --confirm
```

**Output**:
```
About to delete 2 assets:
  - 123e4567-e89b-12d3-a456-426614174000
  - 987fcdeb-51a2-43f7-8b9c-0123456789ab
Continue? (y/N): y

Bulk delete completed:
  Succeeded: 2
  Failed:    0
```

**Error Handling**:
```
Bulk delete completed:
  Succeeded: 3
  Failed:    2
Errors:
  - Asset not found: uuid1
  - Permission denied: uuid2
```

---

### 5. `validate` - Name Validation

Validate resource name availability before creation.

**Usage**:
```bash
pangolin-admin validate <resource_type> <name1> [name2...]
```

**Resource Types**:
- `catalog`
- `warehouse`

**Examples**:
```bash
# Validate single catalog name
pangolin-admin validate catalog new_production

# Validate multiple names
pangolin-admin validate catalog prod_v2 staging_v2 dev_v2

# Validate warehouse names
pangolin-admin validate warehouse s3_warehouse azure_warehouse
```

**Output**:
```
Name validation results:
  new_production: ✓ Available
  existing_catalog: ✗ Taken
    Reason: Name already exists
  invalid-name: ✗ Taken
    Reason: Invalid name format
```

---

## Performance Benefits

| Operation | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Dashboard Load | 5-10 API calls | 1 API call | **5-10x faster** |
| Catalog Overview | 3-4 API calls | 1 API call | **3-4x faster** |
| Asset Search | Client-side filtering | Server-side filtering | **Reduced bandwidth** |
| Name Validation | Sequential checks | Batch validation | **N requests → 1 request** |

---

## Configuration

### Environment Variables

```bash
# Set API URL (default: http://localhost:8081)
export PANGOLIN_URL=http://localhost:8080

# Set tenant context
export PANGOLIN_TENANT=my_tenant
```

### Using with Profiles

```bash
# Use specific profile
pangolin-admin --profile production stats

# Override URL
pangolin-admin --url http://api.example.com stats
```

---

## Interactive Mode

All commands work in both interactive REPL mode and non-interactive mode:

```bash
# Start interactive mode
pangolin-admin

# Then use commands without the prefix
(admin:user@tenant)> stats
(admin:user@tenant)> catalog-summary production
(admin:user@tenant)> search "sales" --limit 10
```

---

## Error Handling

All commands provide clear error messages:

```bash
# Catalog not found
$ pangolin-admin catalog-summary nonexistent
Error: Failed to get catalog summary: 404 Not Found

# Invalid resource type
$ pangolin-admin validate invalid_type some_name
Error: Invalid value 'invalid_type' for '<RESOURCE_TYPE>'
  [possible values: catalog, warehouse]

# API connection error
$ pangolin-admin stats
Error: API Request Failed: Connection refused
```

---

## Best Practices

1. **Use `stats` for dashboards**: Replace multiple API calls with a single stats command
2. **Validate before creating**: Always validate names before attempting to create resources
3. **Use catalog filters**: Narrow search results by specifying catalog names
4. **Confirm bulk operations**: Review the list before confirming bulk deletes
5. **Set environment variables**: Configure `PANGOLIN_URL` to avoid repetitive `--url` flags

---

## See Also

- [Admin CLI Overview](./admin.md)
- [API Reference](../api/api_overview.md)
- [Performance Optimizations](../../planning/performance_optimizations_status.md)
