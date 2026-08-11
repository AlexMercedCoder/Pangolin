use anyhow::Result;

pub struct AzureSigner {
    account_name: String,
    account_key: String,
}

impl AzureSigner {
    pub fn new(account_name: String, account_key: String) -> Self {
        Self {
            account_name,
            account_key,
        }
    }

    /// Generate SAS token for a specific blob path
    /// Supports az://, abfs://, and abfss:// formats
    #[cfg(feature = "azure")]
    pub async fn generate_sas_token(&self, blob_path: &str) -> Result<String> {
        // Parse Azure path (supports all three formats)
        let (_container, _path) = parse_azure_path(blob_path)?;

        // Placeholder for future SDK integration
        // BlobSasPermissions and actual signing logic will accompany SDK 0.20 integration

        Err(anyhow::anyhow!(
            "SAS generation not fully implemented yet due to SDK version uncertainty"
        ))
    }

    #[cfg(not(feature = "azure"))]
    pub async fn generate_sas_token(&self, _blob_path: &str) -> Result<String> {
        Err(anyhow::anyhow!("Azure support not enabled"))
    }
}

pub fn parse_azure_path(path: &str) -> Result<(String, String)> {
    // Parse az://container/blob/path OR abfs://container@account/path OR abfss://container@account/path
    // `strip_prefix` rather than `starts_with` plus a hand-counted `&path[n..]`,
    // where the index and the prefix have to be kept in step by hand. The three
    // prefixes are mutually exclusive - `abfss://` does not start with
    // `abfs://` - so the order of the arms does not matter.
    let (scheme, rest) = if let Some(rest) = path.strip_prefix("az://") {
        ("az", rest)
    } else if let Some(rest) = path.strip_prefix("abfss://") {
        ("abfss", rest)
    } else if let Some(rest) = path.strip_prefix("abfs://") {
        ("abfs", rest)
    } else {
        return Err(anyhow::anyhow!("Invalid Azure path: {}", path));
    };

    let (container, blob_path) = if scheme == "az" {
        // az://container/blob/path format
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid az:// path format: {}", path));
        }
        (parts[0].to_string(), parts[1].to_string())
    } else {
        // abfs://container@account/path OR abfss://container@account/path format
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid abfs(s):// path format: {}", path));
        }

        // Extract container from container@account
        let container_account = parts[0];
        let container = container_account.split('@').next().ok_or_else(|| {
            anyhow::anyhow!("Invalid container@account format: {}", container_account)
        })?;

        (container.to_string(), parts[1].to_string())
    };

    Ok((container, blob_path))
}
