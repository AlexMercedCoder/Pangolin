//! Best-effort deletion of a single object at a warehouse location.
//!
//! Added for the metadata-orphan problem (B16d/B16g): the Iceberg commit loop
//! writes a full metadata file *before* attempting the compare-and-swap that
//! publishes it. On a lost CAS it retried and wrote a fresh file, abandoning the
//! previous one, and up to five orphans were left behind on a final give-up.
//! Orphaned metadata files are indistinguishable from live ones from the
//! outside, so they cannot be reaped later by inspection alone - they have to be
//! removed at the moment the writer knows they are unreferenced.
//!
//! Every backend routes here so the four of them cannot drift apart, which is
//! the failure mode most of the storage-layer audit findings share.

use anyhow::Result;
use object_store::ObjectStore;
use std::collections::HashMap;

/// Delete `location`, resolving credentials from `storage_config` when the
/// location points at object storage.
///
/// A missing object is *not* an error: callers use this to clean up after a
/// failure, and the object may never have been written.
pub async fn delete_location(
    storage_config: Option<&HashMap<String, String>>,
    location: &str,
) -> Result<()> {
    if let Some(rest) = location.strip_prefix("file://") {
        return match tokio::fs::remove_file(rest).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Failed to delete {}: {}", rest, e)),
        };
    }

    let is_object_store = location.starts_with("s3://")
        || location.starts_with("az://")
        || location.starts_with("abfs://")
        || location.starts_with("gs://");

    if !is_object_store {
        // A bare local path.
        return match tokio::fs::remove_file(location).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Failed to delete {}: {}", location, e)),
        };
    }

    let empty = HashMap::new();
    let config = storage_config.unwrap_or(&empty);
    let store = crate::object_store_factory::create_object_store(config, location)?;

    let key = location
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('/'))
        .map(|(_, key)| key)
        .unwrap_or(location);

    match store.delete(&object_store::path::Path::from(key)).await {
        Ok(()) => Ok(()),
        Err(object_store::Error::NotFound { .. }) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Failed to delete {}: {}", location, e)),
    }
}
