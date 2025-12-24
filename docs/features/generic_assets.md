# Generic Assets (Universal Catalog)

Pangolin is not just for Iceberg tables. It serves as a **Universal Data Catalog**, allowing you to govern, tag, and discover any file-based asset in your data lake—from ML models and video datasets to raw CSVs and PDF documentation.

## Overview

Generic Assets are managed entities in the catalog that point to a location (S3/Azure/GCP) but don't strictly follow the Iceberg table format. They share the same governance features as tables:
- **RBAC**: Control who can view or modify the asset.
- **Discovery**: Full-text search and tagging.
- **Lineage**: Track relationships (e.g., "This ML Model was trained on Table X").
- **Branching**: Include generic assets in your data branches.

## Supported Asset Types

Pangolin natively supports the following types:
- **Data Files**: `ParquetTable`, `CsvTable`, `JsonTable`
- **AI/ML**: `MlModel`
- **Modern Formats**: `DeltaTable`, `HudiTable`, `ApachePaimon`, `Lance`, `Vortex`, `Nimble`
- **Media**: `VideoFile`, `ImageFile`
- **System**: `View`, `DbConnString`, `Directory`, `Other`

---

## 🛠️ Usage Guide (API)

Currently, Generic Assets are managed primarily via the REST API.

### 1. Registering an Asset

**Endpoint**: `POST /api/v1/catalogs/{catalog}/namespaces/{namespace}/assets`

#### Example 1: Registering an ML Model
Catalog a trained model artifact stored in S3.

```bash
curl -X POST http://localhost:8080/api/v1/catalogs/analytics/namespaces/ml_models/assets \
  -H "Authorization: Bearer <token>" \
  -H "X-Pangolin-Tenant: <tenant-id>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "fraud_detection_v2",
    "kind": "MlModel",
    "location": "s3://my-datalake/models/fraud_v2.tar.gz",
    "properties": {
      "framework": "pytorch",
      "version": "2.1.0",
      "accuracy": "0.985",
      "training_date": "2023-11-15",
      "trained_by": "alice@example.com"
    }
  }'
```

#### Example 2: Registering a Video Dataset
Catalog a folder of raw video footage for ingestion.

```bash
curl -X POST http://localhost:8080/api/v1/catalogs/raw_data/namespaces/videos/assets \
  -d '{
    "name": "cctv_logs_2023",
    "kind": "VideoFile",
    "location": "s3://raw-zone/cctv/2023/",
    "properties": {
      "format": "mp4",
      "codec": "h264",
      "retention": "30_days"
    }
  }'
```

#### Example 3: Registering an External Delta Table
Catalog an existing Delta Lake table for discovery (without managing it via Iceberg).

```bash
curl -X POST http://localhost:8080/api/v1/catalogs/analytics/namespaces/silver/assets \
  -d '{
    "name": "marketing_leads",
    "kind": "DeltaTable",
    "location": "abfss://data@account.dfs.core.windows.net/delta/leads",
    "properties": {
      "managed_by": "databricks"
    }
  }'
```

### 2. Discovering Assets

Generic assets appear alongside Iceberg tables in search results.

**Search API**:
```bash
GET /api/v1/assets/search?q=fraud
```

**Response**:
```json
[
  {
    "id": "uuid...",
    "name": "fraud_detection_v2",
    "kind": "MlModel",
    "catalog": "analytics",
    "namespace": "ml_models"
  },
  {
    "id": "uuid...",
    "name": "fraud_transactions",
    "kind": "IcebergTable",
    "catalog": "analytics",
    "namespace": "finance"
  }
]
```

---

## 🖥️ Usage Guide (UI)

You can interactively register and manage generic assets directly from the **Data Explorer**.

1. **Navigate**: Go to **Data Explorer** and drill down into a **Catalog** and **Namespace**.
2. **Register**: Click the **"Register Asset"** button in the top-right corner.
3. **Configure**:
   - **Name**: Enter a unique name for the asset.
   - **Type**: Select from the dropdown (e.g., `ML Model`, `Video File`, `Delta Table`).
   - **Location**: Provide the full S3/Azure/GCS URI (e.g., `s3://my-bucket/path/to/asset`).
   - **Properties**: Add key-value pairs for metadata (e.g., `accuracy: 0.99`, `owner: data-team`).
4. **Submit**: Click **Register**. The asset will now be discoverable in search.

---

## 📚 Related Documentation

- **[Modern Table Formats](./table_formats.md)**: Specific guide for cataloging **Delta Lake**, **Hudi**, and **Paimon**.
- **[Asset Management](./asset_management.md)**: General governance guide.

---

## 🔒 Security

Generic Assets leverage the same permission model as tables.
- **READ**: Required to view metadata or search for the asset.
- **WRITE**: Required to update properties or location.

**Example Policy**:
Grant Data Scientists access to register models.
```json
{
  "effect": "Allow",
  "action": "RegisterAsset",
  "resource": "catalog/analytics/namespace/ml_models/*"
}
```
