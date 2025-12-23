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

### Supported Properties (PyIceberg Compatible)
| PyIceberg Property | Description | Vended by Pangolin |
| :--- | :--- | :--- |
| `adls.token` | OAuth2 access token for Azure AD authentication. | ✅ Yes (OAuth2 mode) |
| `adls.account-name` | Azure storage account name. | ✅ Yes (all modes) |
| `adls.account-key` | Azure storage account key. | ✅ Yes (account key mode) |
| `adls.container` | Container name within the storage account. | ✅ Yes (all modes) |

**Note:** When using Pangolin's credential vending, you don't need to provide these properties manually. Pangolin vends them automatically based on your warehouse configuration.

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

### Supported Properties (PyIceberg Compatible)
| PyIceberg Property | Description | Vended by Pangolin |
| :--- | :--- | :--- |
| `gcp-oauth-token` | OAuth2 access token for GCP authentication. | ✅ Yes (OAuth2 mode) |
| `gcp-project-id` | GCP project ID. | ✅ Yes (all modes) |
| `gcs.service-account-key` | Path to service account JSON key (client-provided). | ❌ No (client manages) |

**Note:** When using Pangolin's credential vending, you only need to provide `token` for authentication. Pangolin vends `gcp-oauth-token` and `gcp-project-id` automatically.

---

## 🎛️ Hybrid Cloud Scenarios

Pangolin allows you to manage catalogs across different clouds from a single control plane. One tenant can have an S3 warehouse for analytics and a GCS warehouse for ML workloads.

### Key Considerations
1. **Signer Implementation**: Ensure your Pangolin server is built with the appropriate cloud SDK features (`--features azure-oauth` or `--features gcp-oauth`).
2. **Library Versions**: Earlier versions of PyIceberg had limited support for non-S3 backends. We recommend using **PyIceberg 0.7.0+** for the best multi-cloud experience.
3. **Region Consistency**: Ensure the `region` or `location` properties in your Pangolin Warehouse configuration match the physical bucket locations to minimize latency and costs.
