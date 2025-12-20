# PyIceberg Multi-Cloud Integration

Pangolin's credential vending and REST support extend beyond AWS to Azure Blob Storage (ADLS Gen2) and Google Cloud Storage (GCS).

## 🔷 Azure ADLS Gen2

### Dependencies
Install PyIceberg with the `adlfs` and `pyarrow` extras:
```bash
pip install "pyiceberg[adlfs,pyarrow]"
```

### Configuration (Authenticated)
When using Azure, the catalog URI remains the same, but the storage properties change.

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "azure_catalog",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/azure_catalog",
        "token": "YOUR_JWT_TOKEN",
        
        # Enable Vending
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        
        # Optional: Direct Key Access
        # "adls.account-name": "mystorageaccount",
        # "adls.account-key": "YOUR_ACCOUNT_KEY"
    }
)
```

### Supported Properties
| PyIceberg Property | Description |
| :--- | :--- |
| `adls.account-name` | Azure storage account name. |
| `adls.account-key` | Azure storage account key (if not using vending). |
| `adls.connection-string` | Full Azure connection string. |

---

## 🔶 Google Cloud Storage (GCS)

### Dependencies
Install PyIceberg with the `gcsfs` and `pyarrow` extras:
```bash
pip install "pyiceberg[gcsfs,pyarrow]"
```

### Configuration (Authenticated)

```python
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "gcp_catalog",
    **{
        "type": "rest",
        "uri": "http://localhost:8080/v1/gcp_catalog",
        "token": "YOUR_JWT_TOKEN",
        
        # Enable Vending
        "header.X-Iceberg-Access-Delegation": "vended-credentials",
        
        # Optional: Direct Service Account Access
        # "gcs.project-id": "my-project",
        # "gcs.service-account-key": "/path/to/key.json"
    }
)
```

### Supported Properties
| PyIceberg Property | Description |
| :--- | :--- |
| `gcs.project-id` | GCP project ID. |
| `gcs.service-account-key` | Path to service account JSON key (if not using vending). |
| `gcs.oauth2.token` | A raw OAuth2 token (Pangolin can vend this via `vended-credentials`). |

---

## 🎛️ Hybrid Cloud Scenarios

Pangolin allows you to manage catalogs across different clouds from a single control plane. One tenant can have an S3 warehouse for analytics and a GCS warehouse for ML workloads.

### Key Considerations
1. **Signer Implementation**: Ensure your Pangolin server is built with the appropriate cloud SDK features (`--features azure-oauth` or `--features gcp-oauth`).
2. **Library Versions**: Earlier versions of PyIceberg had limited support for non-S3 backends. We recommend using **PyIceberg 0.7.0+** for the best multi-cloud experience.
3. **Region Consistency**: Ensure the `region` or `location` properties in your Pangolin Warehouse configuration match the physical bucket locations to minimize latency and costs.
