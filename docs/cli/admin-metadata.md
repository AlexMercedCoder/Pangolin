# Admin Metadata Management

Attach arbitrary business metadata or configuration to entities.

## Commands

### Get Metadata
Retrieve the metadata JSON attached to an entity.

**Syntax**:
```bash
pangolin-admin get-metadata --entity-type <type> --entity-id <id>
```

**Entities**:
- `catalog`
- `table`
- `view`

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
pangolin-admin set-metadata --entity-type table --entity-id sales.orders owner "Data Team"
pangolin-admin set-metadata --entity-type catalog --entity-id sales_catalog description "Primary sales data"
```
