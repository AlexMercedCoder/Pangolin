# Admin Catalog Management

Catalogs provide a logical namespace (Iceberg Catalog) on top of a physical Warehouse.

## Commands

### List Catalogs
View all catalogs.

**Syntax**:
```bash
pangolin-admin list-catalogs [--limit <N>] [--offset <N>]
```

**Options**:
- `--limit`: Maximum number of items to return (default: 100)
- `--offset`: Number of items to skip (default: 0)

### Create Catalog
Create a new catalog backed by an existing warehouse.

> [!IMPORTANT]
> This command requires **Tenant Admin** privileges. The Root User cannot create catalogs.

**Syntax**:
```bash
pangolin-admin create-catalog <name> --warehouse <warehouse_name>
```

**Example**:
```bash
pangolin-admin create-catalog sales_catalog --warehouse main_lake
```

### Update Catalog
Modify a catalog's configuration.

**Syntax**:
```bash
pangolin-admin update-catalog --id <uuid> [--name <new_name>]
```

**Example**:
```bash
pangolin-admin update-catalog --id "catalog-uuid" --name "archived-sales"
```

### Delete Catalog
Delete a catalog.

**Syntax**:
```bash
pangolin-admin delete-catalog <name>
```

**Example**:
```bash
pangolin-admin delete-catalog temp_catalog
```
