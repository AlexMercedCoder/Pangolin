# Generic Assets Best Practices

## Overview

This guide covers best practices for managing non-table assets in Pangolin, including ML models, files, videos, and other data artifacts.

## Asset Types

### Supported Generic Assets

**Data Files**
- Parquet, CSV, JSON
- Lance (vector database format)
- Vortex (columnar format)

**ML Artifacts**
- Trained models (PyTorch, TensorFlow, scikit-learn)
- Model metadata and metrics
- Training datasets
- Feature stores

**Media Files**
- Images (JPEG, PNG, TIFF)
- Videos (MP4, AVI, MOV)
- Audio files

**Other**
- Directories/datasets
- Database connection strings
- Custom file types

## Registration

### File Assets

**CSV Files**
```python
from pypangolin import PangolinClient
from pypangolin.assets import CsvAsset

client = PangolinClient("http://localhost:8080")
client.login("admin", "password")

# Register and write CSV
csv_asset = CsvAsset.write(
    client,
    catalog="data_files",
    namespace="exports",
    name="customer_export_2025",
    data=df,
    location="s3://bucket/exports/customers.csv"
)

# Read back
df = CsvAsset.read(client, "data_files", "exports", "customer_export_2025")
```

**JSON Files**
```python
from pypangolin.assets import JsonAsset

# Register JSON
json_asset = JsonAsset.write(
    client,
    catalog="data_files",
    namespace="configs",
    name="app_config",
    data=config_dict,
    location="s3://bucket/configs/app.json"
)
```

### ML Models

**Model Registration**
```python
from pypangolin.assets import MlModelAsset

# Register trained model
model_asset = MlModelAsset.register(
    client,
    catalog="ml_models",
    namespace="production",
    name="customer_churn_v2",
    location="s3://bucket/models/churn/v2/model.pkl",
    framework="scikit-learn",
    version="2.0.1",
    metrics={
        "accuracy": 0.92,
        "precision": 0.89,
        "recall": 0.94,
        "f1_score": 0.91
    },
    training_data="ml_features.customer_features@2025-12-01",
    description="Customer churn prediction model trained on Q4 2025 data"
)
```

**Model Metadata**
```python
# Comprehensive model metadata
metadata = {
    "framework": "pytorch",
    "framework_version": "2.1.0",
    "model_type": "transformer",
    "architecture": "bert-base",
    "parameters": 110000000,
    "training": {
        "dataset": "ml_features.text_corpus",
        "epochs": 10,
        "batch_size": 32,
        "learning_rate": 0.001,
        "optimizer": "adam"
    },
    "performance": {
        "train_loss": 0.15,
        "val_loss": 0.18,
        "test_accuracy": 0.94
    },
    "deployment": {
        "environment": "production",
        "endpoint": "https://api.example.com/predict",
        "sla_ms": 100
    }
}
```

### Vector Data (Lance)

**Lance Dataset**
```python
from pypangolin.assets import LanceAsset

# Write vector embeddings
lance_asset = LanceAsset.write(
    client,
    catalog="vectors",
    namespace="embeddings",
    name="product_embeddings",
    data=embeddings_df,
    location="/data/lance/products"
)

# Read for similarity search
embeddings = LanceAsset.read(client, "vectors", "embeddings", "product_embeddings")
```

### Media Files

**Image Assets**
```python
from pypangolin.assets import ImageAsset

# Register image dataset
image_asset = ImageAsset.register(
    client,
    catalog="media",
    namespace="product_images",
    name="catalog_photos_2025",
    location="s3://bucket/images/products/",
    format="jpeg",
    resolution="1920x1080",
    count=10000,
    total_size_mb=5000
)
```

**Video Assets**
```python
from pypangolin.assets import VideoAsset

# Register video collection
video_asset = VideoAsset.register(
    client,
    catalog="media",
    namespace="training_videos",
    name="onboarding_series",
    location="s3://bucket/videos/onboarding/",
    format="mp4",
    codec="h264",
    duration_seconds=3600,
    resolution="1920x1080"
)
```

## Organization

### Namespace Structure

**By Asset Type**
```
data_files/
├── csv/
├── json/
├── parquet/
└── lance/

ml_models/
├── production/
├── staging/
└── experiments/

media/
├── images/
├── videos/
└── audio/
```

**By Project/Domain**
```
customer_analytics/
├── models/
├── datasets/
├── reports/
└── visualizations/

marketing/
├── campaigns/
├── assets/
└── analytics/
```

## Versioning

### Model Versioning

**Semantic Versioning**
```python
# Version progression
models = [
    "customer_churn_v1.0.0",  # Initial release
    "customer_churn_v1.1.0",  # Feature addition
    "customer_churn_v1.1.1",  # Bug fix
    "customer_churn_v2.0.0",  # Major update
]
```

**Git-Style Tagging**
```bash
# Tag production model
pangolin-user create-tag \
  --catalog ml_models \
  --name production-2025-12-24 \
  --asset customer_churn_v2
```

### Dataset Versioning

**Snapshot-Based**
```python
# Reference specific snapshot
dataset = "ml_features.customer_features@snapshot-abc123"

# Reference by date
dataset = "ml_features.customer_features@2025-12-01"
```

## Metadata Management

### Rich Metadata

**File Assets**
```json
{
  "asset_type": "csv",
  "format": "csv",
  "compression": "gzip",
  "row_count": 1000000,
  "size_bytes": 50000000,
  "schema": {
    "columns": ["id", "name", "email", "created_at"],
    "types": ["int64", "string", "string", "timestamp"]
  },
  "created_by": "etl-pipeline",
  "source_system": "crm",
  "refresh_schedule": "daily"
}
```

**ML Models**
```json
{
  "asset_type": "ml_model",
  "framework": "tensorflow",
  "task": "classification",
  "input_schema": {
    "features": ["age", "income", "tenure"],
    "types": ["float", "float", "int"]
  },
  "output_schema": {
    "prediction": "binary",
    "confidence": "float"
  },
  "performance_metrics": {
    "accuracy": 0.92,
    "auc_roc": 0.95
  },
  "deployment_status": "production",
  "monitoring_dashboard": "https://monitoring.example.com/models/churn"
}
```

## Access Patterns

### Discovery

**Searchable Metadata**
```python
# Search for ML models
results = client.search(
    catalog="ml_models",
    query="classification",
    filters={
        "asset_type": "ml_model",
        "framework": "pytorch",
        "deployment_status": "production"
    }
)
```

**Tagging**
```bash
# Tag assets for discovery
pangolin-admin set-metadata \
  --catalog ml_models \
  --namespace production \
  --asset customer_churn \
  --tags "ml,classification,production,customer-analytics"
```

### Lineage Tracking

**Model Lineage**
```json
{
  "model": "customer_churn_v2",
  "lineage": {
    "training_data": [
      "ml_features.customer_features@2025-12-01",
      "ml_features.transaction_history@2025-12-01"
    ],
    "parent_model": "customer_churn_v1",
    "derived_models": [
      "customer_churn_v2_quantized",
      "customer_churn_v2_onnx"
    ]
  }
}
```

## Storage Optimization

### File Organization

**Hierarchical Structure**
```
s3://bucket/
├── models/
│   ├── customer_churn/
│   │   ├── v1/
│   │   │   ├── model.pkl
│   │   │   └── metadata.json
│   │   └── v2/
│   │       ├── model.pkl
│   │       └── metadata.json
│   └── product_recommendation/
└── datasets/
    ├── training/
    └── validation/
```

### Compression

**Appropriate Compression**
```python
# CSV with gzip
CsvAsset.write(..., compression="gzip")

# Parquet with snappy
ParquetAsset.write(..., compression="snappy")

# Models with pickle protocol 5
import pickle
with open("model.pkl", "wb") as f:
    pickle.dump(model, f, protocol=5)
```

### Lifecycle Policies

**S3 Lifecycle**
```json
{
  "Rules": [{
    "Id": "ArchiveOldModels",
    "Filter": {"Prefix": "models/experiments/"},
    "Status": "Enabled",
    "Transitions": [{
      "Days": 90,
      "StorageClass": "GLACIER"
    }]
  }]
}
```

## Governance

### Access Control

**Model Access**
```bash
# Grant read access to production models
pangolin-admin grant-permission \
  --user data-scientist@company.com \
  --catalog ml_models \
  --namespace production \
  --role READ

# Grant write access to experiments
pangolin-admin grant-permission \
  --user data-scientist@company.com \
  --catalog ml_models \
  --namespace experiments \
  --role READ_WRITE
```

### Compliance

**PII in Training Data**
```json
{
  "model": "customer_churn",
  "compliance": {
    "contains_pii": true,
    "pii_fields": ["email", "phone"],
    "anonymization": "applied",
    "retention_policy": "delete_after_90_days",
    "gdpr_compliant": true
  }
}
```

## Best Practices Checklist

### Registration
- [ ] Descriptive names following conventions
- [ ] Comprehensive metadata
- [ ] Appropriate namespace organization
- [ ] Version information included

### Metadata
- [ ] Asset type clearly specified
- [ ] Format and schema documented
- [ ] Performance metrics (for models)
- [ ] Lineage information
- [ ] Tags for discovery

### Storage
- [ ] Appropriate compression used
- [ ] Hierarchical organization
- [ ] Lifecycle policies configured
- [ ] Backup strategy defined

### Governance
- [ ] Access controls configured
- [ ] Compliance requirements met
- [ ] Audit logging enabled
- [ ] Retention policies defined

### Operations
- [ ] Monitoring configured
- [ ] Versioning strategy defined
- [ ] Deprecation process documented
- [ ] Cleanup procedures automated

## Additional Resources

- [Generic Assets Guide](../features/generic_assets.md)
- [PyPangolin File Assets](../../pypangolin/docs/csv.md)
- [PyPangolin ML Models](../../pypangolin/docs/other.md)
- [Lance Format](../../pypangolin/docs/lance.md)
