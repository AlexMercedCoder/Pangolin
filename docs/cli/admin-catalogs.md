# Admin Catalog Management

Catalogs provide a logical namespace (Iceberg Catalog) on top of a physical Warehouse.

## Commands

### List Catalogs
View all catalogs.

**Syntax**:
```bash
pangolin-admin list-catalogs
```

### Create Catalog
Create a new catalog backed by an existing warehouse.

**Syntax**:
```bash
pangolin-admin create-catalog <name> --warehouse <warehouse_name>
```

**Example**:
```bash
pangolin-admin create-catalog sales_catalog --warehouse main_lake
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
