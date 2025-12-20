# Admin Metadata Management

Business metadata allows you to attach technical and organizational context to any entity (Catalog, Warehouse, Table, View).

## Commands

### Get Metadata
Retrieve the metadata JSON attached to an entity.

**Syntax**:
```bash
pangolin-admin get-metadata --entity-type <type> --entity-id <id>
```

**Entity Types**: `catalog`, `warehouse`, `table`, `view`.

**Example**:
```bash
pangolin-admin get-metadata --entity-type table --entity-id sales.orders
```

### Set Metadata
Upsert a key-value pair into an entity's metadata.

**Syntax**:
```bash
pangolin-admin set-metadata --entity-type <type> --entity-id <id> <key> <value>
```

**Example**:
```bash
pangolin-admin set-metadata --entity-type table --entity-id sales.orders owner "Data Engineering"
```

### Delete Metadata
Remove all business metadata from an asset.

**Syntax**:
```bash
pangolin-admin delete-metadata --asset-id <uuid>
```

**Example**:
```bash
pangolin-admin delete-metadata --asset-id "table-uuid"
```
