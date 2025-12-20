# Table Maintenance

Maintaining your Iceberg tables is critical for both storage efficiency and query performance. Pangolin provides built-in utilities to manage metadata growth and clean up unreferenced data files.

---

## 🛠️ Available Operations

### 1. Expire Snapshots
Over time, Iceberg tables accumulate snapshots. Expiring old snapshots reduces the size of the metadata files and allows for the deletion of data files that are no longer part of any valid state.

-   **Endpoint**: `POST /v1/{prefix}/namespaces/{ns}/tables/{table}/maintenance`
-   **Payload**:
    ```json
    {
      "action": "expire_snapshots",
      "older_than_timestamp": 1735689600000,
      "retain_last": 10
    }
    ```
-   **Logic**: 
    1.  Identifies snapshots older than the timestamp OR outside the `retain_last` count.
    2.  Removes these snapshots from the metadata.
    3.  Triggers the underlying storage provider (S3/Azure/GCS) to delete unreferenced manifests and data files.

### 2. Remove Orphan Files
Failed write jobs or uncommitted transactions can leave "orphan" files in your storage bucket that aren't tracked by any metadata.

-   **Endpoint**: `POST /v1/{prefix}/namespaces/{ns}/tables/{table}/maintenance`
-   **Payload**:
    ```json
    {
      "action": "remove_orphan_files",
      "older_than_timestamp": 1735689600000
    }
    ```
-   **Logic**: 
    1.  Scans the table's storage location.
    2.  Compares files on disk with those referenced in *all* valid snapshots.
    3.  Deletes files not mentioned in metadata (subject to the `older_than` safety buffer).

---

## 🔐 Permissions Required

To run maintenance operations, the user must have the following permissions:

| Operation | Action | Scope |
| :--- | :--- | :--- |
| **All Maintenance** | `write` | Asset or Namespace |

> [!IMPORTANT]
> Because maintenance operations can physically delete data from your cloud storage, it is highly recommended to only grant these permissions to **Service Users** or **Data Administrators**.

---

## 🛠️ Tooling Status

| Interface | Status |
| :--- | :--- |
| **REST API** | ✅ Fully Supported |
| **Python SDK** | ✅ Supported via `table.expire_snapshots()` |
| **Pangolin CLI** | 🏗️ Coming Soon |
| **Management UI** | ✅ Supported in Asset Details view |

---

## 🚦 Best Practices
- **Retention Policy**: Set a standard retention (e.g., 7 days or 100 snapshots) to avoid metadata bloat.
- **Safety Buffers**: When removing orphan files, always use an `older_than` buffer of at least 24 hours to avoid deleting files from currently running ingest jobs.
- **Audit Trails**: Monitor maintenance actions in the **Audit Logs** to ensure they are running as scheduled.
