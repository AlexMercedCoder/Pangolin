# User Data Discovery

Explore the available datasets and integrate them into your tools.

## Commands

### List Catalogs
See which catalogs you have access to.

**Syntax**:
```bash
pangolin-user list-catalogs
```

### Search
Search for tables, views, or dashboards across all catalogs.

**Syntax**:
```bash
pangolin-user search <query>
```

**Example**:
```bash
pangolin-user search "customer revenue"
```

### Generate Code
Generate boilerplate code to connect to a specific table. Pangolin outputs code with syntax highlighting for immediate use.

**Syntax**:
```bash
pangolin-user generate-code --language <lang> --table <full_table_name>
```

**Supported Languages**: `pyiceberg`, `pyspark`, `dremio`, `sql`.

**Example**:
```bash
pangolin-user generate-code --language pyiceberg --table sales_catalog.public.orders
```

### Get Token
Generate a personal API key for use in external scripts or tools.

**Syntax**:
```bash
pangolin-user get-token [--description <text>] [--expires-in <days>]
```

**Example**:
```bash
pangolin-user get-token --description "CI/CD Pipeline" --expires-in 30
```
