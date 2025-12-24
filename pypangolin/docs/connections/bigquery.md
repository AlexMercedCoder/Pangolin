# Google BigQuery Connections

> [!WARNING]
> **Untested Implementation**
> 
> This connection type has been implemented based on Google Cloud BigQuery's Python client documentation but has not been tested against a live BigQuery project. The implementation should work as designed, but users should verify functionality in their environment.

Securely store and connect to Google BigQuery with encrypted credentials.

## Installation

```bash
pip install "pypangolin[bigquery]"
```

## Registering a Connection

### Using Service Account JSON

```python
from pypangolin import PangolinClient
from pypangolin.assets.connections import BigQueryAsset
from cryptography.fernet import Fernet
import json

client = PangolinClient(uri="http://localhost:8080")
client.login("username", "password")

# User-managed encryption (recommended)
encryption_key = Fernet.generate_key().decode('utf-8')

# Load service account JSON
with open('service-account.json', 'r') as f:
    credentials_json = f.read()

BigQueryAsset.register(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_bigquery",
    connection_string="bigquery://my-gcp-project",
    credentials={
        "project_id": "my-gcp-project",
        "credentials_json": credentials_json  # Service account JSON as string
    },
    encryption_key=encryption_key,
    store_key=False,
    description="Production BigQuery project"
)
```

### Using Application Default Credentials

```python
BigQueryAsset.register(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="dev_bigquery",
    connection_string="bigquery://my-dev-project",
    credentials={
        "project_id": "my-dev-project"
        # No credentials_json - will use Application Default Credentials
    },
    encryption_key=encryption_key,
    store_key=False,
    description="Development BigQuery (uses ADC)"
)
```

## Connecting to BigQuery

```python
import os

encryption_key = os.getenv("BIGQUERY_ENCRYPTION_KEY")

bq_client = BigQueryAsset.connect(
    client,
    catalog="data_sources",
    namespace="warehouses",
    name="prod_bigquery",
    encryption_key=encryption_key
)

# Use the BigQuery client
query = """
    SELECT 
        DATE(timestamp) as date,
        COUNT(*) as event_count
    FROM `project.dataset.events`
    WHERE DATE(timestamp) >= DATE_SUB(CURRENT_DATE(), INTERVAL 7 DAY)
    GROUP BY 1
    ORDER BY 1 DESC
"""

query_job = bq_client.query(query)
results = query_job.result()

for row in results:
    print(f"Date: {row.date}, Events: {row.event_count}")
```

## Required Credentials

- **project_id** (required) - GCP project ID

## Optional Credentials

- **credentials_json** - Service account JSON as string (if not using ADC)

## Connection String Format

```
bigquery://project-id
```

## Authentication Methods

### 1. Service Account JSON (Recommended for Production)
- Most secure for production
- Credentials encrypted and stored in Pangolin
- No dependency on environment configuration

### 2. Application Default Credentials (Development)
- Uses `gcloud auth application-default login`
- Good for development/testing
- Requires proper GCP SDK configuration

## Example Usage

### Query Data

```python
bq_client = BigQueryAsset.connect(client, "data_sources", "warehouses", "prod_bigquery", encryption_key=key)

# Standard SQL query
query = """
    SELECT 
        user_id,
        COUNT(*) as action_count,
        MAX(timestamp) as last_action
    FROM `my-project.analytics.user_actions`
    WHERE DATE(timestamp) = CURRENT_DATE()
    GROUP BY user_id
    HAVING action_count > 10
"""

df = bq_client.query(query).to_dataframe()
print(df.head())
```

### Create Table

```python
from google.cloud import bigquery

bq_client = BigQueryAsset.connect(client, "data_sources", "warehouses", "prod_bigquery", encryption_key=key)

schema = [
    bigquery.SchemaField("name", "STRING", mode="REQUIRED"),
    bigquery.SchemaField("age", "INTEGER", mode="REQUIRED"),
    bigquery.SchemaField("email", "STRING", mode="NULLABLE"),
]

table_ref = bq_client.dataset("my_dataset").table("my_table")
table = bigquery.Table(table_ref, schema=schema)
table = bq_client.create_table(table)
print(f"Created table {table.project}.{table.dataset_id}.{table.table_id}")
```

## Security Best Practices

- Use service account JSON with minimal required permissions
- Rotate service account keys regularly
- Use user-managed encryption keys
- Enable BigQuery audit logging
- Use VPC Service Controls for additional security
- Never commit service account JSON to version control

## Troubleshooting

### Authentication Errors
- Verify service account JSON is valid
- Check service account has necessary IAM roles
- Ensure project_id matches the service account project

### Permission Denied
- Verify service account has BigQuery Data Viewer/Editor roles
- Check dataset-level permissions
- Ensure billing is enabled on the project

### Quota Exceeded
- Check BigQuery quotas in GCP Console
- Consider using batch queries for large operations
- Implement rate limiting in your application

## Notes

- BigQuery is serverless - no connection pooling needed
- Queries are billed based on data processed
- Use partitioned tables and clustering for cost optimization
- Standard SQL is recommended over Legacy SQL
- Test thoroughly in your environment before production deployment
