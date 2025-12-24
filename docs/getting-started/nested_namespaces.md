# Working with Nested Namespaces

Pangolin supports hierarchical (nested) namespaces, allowing you to organize tables into logical structures like `finance.payroll.2023` or `raw.iot.sensors`.

## Overview

In the Iceberg spec, a namespace is a list of strings (e.g., `["finance", "payroll"]`).
- **Storage**: Maps to nested directories in your warehouse (e.g., `s3://bucket/wh/finance/payroll/`).
- **API**: Represented as JSON arrays in bodies, and usually dot-separated or URL-encoded strings in path parameters.

---

## 🏗️ Creating Nested Namespaces

Currently, the most reliable way to create nested namespaces is via the **REST API**, as the UI currently supports single-level creation.

### API Endpoint

`POST /v1/{catalog}/namespaces`

### Request Body

To create `marketing.campaigns.q1`:

```bash
curl -X POST http://localhost:8080/v1/analytics/namespaces \
  -H "Authorization: Bearer <token>" \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Content-Type: application/json" \
  -d '{
    "namespace": ["marketing", "campaigns", "q1"],
    "properties": {
      "owner": "marketing-team",
      "retention": "365d"
    }
  }'
```

**Note**: You do NOT need to create the parent `marketing` or `marketing.campaigns` explicitly before creating the child. However, creating them explicitly is good practice for setting properties at each level.

---

## 🔍 Specifying Namespaces in API Calls

When referencing a nested namespace in URL paths (e.g., to list tables), the handling depends on the client and the specific endpoint.

### 1. Dot Notation (Common)
For most HTTP clients and simple setups, you can often use dot-notation if your namespace parts don't contain dots themselves.

`GET /v1/analytics/namespaces/marketing.campaigns.q1/tables`

### 2. URL Encoding (Robust)
The default separator for Iceberg REST is the Unit Separator character `\u001F` (ASCII 31). However, many implementations (including typical usage patterns in Pangolin) accept encoded dots or specific delimiters.

If accessing programmatically:
- **PyIceberg**: Handles this automatically.
- **REST Clients**: If `marketing.campaigns.q1` fails, ensure you are encoding the separator used by your namespace creation.

---

## 🐍 Client Examples

### PyIceberg (Python)
PyIceberg creates generic `["part1", "part2"]` namespaces natively.

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog("pangolin", **config)

# Create nested namespace
catalog.create_namespace(["marketing", "campaigns", "q1"])

# List tables
tables = catalog.list_tables("marketing.campaigns.q1")
```

### Spark (SQL)

```sql
-- Creating nested namespaces (if supported by the catalog property set)
CREATE NAMESPACE pangolin.marketing.campaigns.q1;

-- Creating a table in a nested namespace
CREATE TABLE pangolin.marketing.campaigns.q1.leads (
    id bigint,
    email string
) USING iceberg;
```

---

## 📂 Storage Layout

Pangolin maps namespaces to physical folders in your object storage.

For a warehouse bucket `my-datalake`:
- **Namespace**: `["marketing", "campaigns"]`
- **Table**: `leads`
- **Path**: `s3://my-datalake/analytics/marketing/campaigns/leads/`

This hierarchical structure is preserved, making it easy to browse data directly in S3/Azure/GCP consoles if needed.
