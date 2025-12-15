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

**Supported Languages**:
- `pyiceberg`: Python code using PyIceberg.
- `pyspark`: PySpark configuration and DataFrame loading.
- `dremio`: SQL command to add the source in Dremio.
- `sql`: Standard SELECT query.

**Example**:
```bash
pangolin-user generate-code --language pyiceberg --table sales_catalog.public.orders
```
