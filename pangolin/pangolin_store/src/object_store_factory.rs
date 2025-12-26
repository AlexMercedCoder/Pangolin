use anyhow::Result;
use object_store::ObjectStore;
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_object_store(
    storage_config: &HashMap<String, String>,
    location: &str,
) -> Result<Box<dyn ObjectStore>> {
    if location.starts_with("s3://") {
        create_s3_store(storage_config, location)
    } else if location.starts_with("az://") || location.starts_with("abfs://") {
        create_azure_store(storage_config, location)
    } else if location.starts_with("file://") || location.starts_with("/") {
        create_local_store(storage_config, location)
    } else {
        Err(anyhow::anyhow!("Unsupported scheme for location: {}", location))
    }
}

fn create_local_store(
    _config: &HashMap<String, String>,
    location: &str,
) -> Result<Box<dyn ObjectStore>> {
    let path = if location.starts_with("file://") {
        location.strip_prefix("file://").unwrap()
    } else {
        location
    };
    
    // Ensure directory exists
    std::fs::create_dir_all(path)?;
    
    Ok(Box::new(object_store::local::LocalFileSystem::new_with_prefix(path)?))
}

fn create_s3_store(
    config: &HashMap<String, String>,
    location: &str,
) -> Result<Box<dyn ObjectStore>> {
    // Use bucket from config if available, otherwise parse from location
    let bucket = config
        .get("s3.bucket")
        .or_else(|| config.get("bucket"))
        .map(|s| s.as_str())
        .or_else(|| {
            // Fallback: parse from location
            location
                .strip_prefix("s3://")
                .and_then(|rest| rest.split_once('/'))
                .map(|(b, _)| b)
        })
        .ok_or_else(|| anyhow::anyhow!("Cannot determine S3 bucket from config or location"))?;

    let region = config
        .get("s3.region")
        .or_else(|| config.get("region"))
        .map(|s| s.as_str())
        .unwrap_or("us-east-1");
    let access_key = config
        .get("s3.access-key-id")
        .or_else(|| config.get("access_key_id"))
        .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
    let secret_key = config
        .get("s3.secret-access-key")
        .or_else(|| config.get("secret_access_key"))
        .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
    let endpoint = config.get("s3.endpoint").or_else(|| config.get("endpoint"));

    let mut builder = object_store::aws::AmazonS3Builder::new()
        .with_region(region)
        .with_bucket_name(bucket)
        .with_access_key_id(access_key)
        .with_secret_access_key(secret_key)
        .with_allow_http(true); // Useful for MinIO

    if let Some(ep) = endpoint {
        builder = builder.with_endpoint(ep);
    }
    
    println!("DEBUG: s3.path-style-access config check: {:?}", config.get("s3.path-style-access"));
    if let Some(true) = config.get("s3.path-style-access").map(|s| s == "true") {
        println!("DEBUG: Enabling path-style access (virtual_hosted_style_request = false)");
        builder = builder.with_virtual_hosted_style_request(false);
    } else {
        println!("DEBUG: Path-style access NOT enabled");
    }

    Ok(Box::new(builder.build()?))
}

fn create_azure_store(
    config: &HashMap<String, String>,
    location: &str,
) -> Result<Box<dyn ObjectStore>> {
    // Basic Azure implementation (can expand as needed)
    let container = if location.starts_with("az://") {
        location.strip_prefix("az://").unwrap().split('/').next().unwrap()
    } else {
        location.strip_prefix("abfs://").unwrap().split('@').next().unwrap()
    };

    let account = config.get("azure.account-name").ok_or_else(|| anyhow::anyhow!("Missing azure.account-name"))?;
    let key = config.get("azure.account-key").ok_or_else(|| anyhow::anyhow!("Missing azure.account-key"))?;

    let builder = object_store::azure::MicrosoftAzureBuilder::new()
        .with_account(account)
        .with_access_key(key)
        .with_container_name(container);
        
    Ok(Box::new(builder.build()?))
}

fn create_gcp_store(
    config: &HashMap<String, String>,
    location: &str,
) -> Result<Box<dyn ObjectStore>> {
    let bucket = location.strip_prefix("gs://").unwrap().split('/').next().unwrap();
    let service_account = config.get("gcp.service-account").ok_or_else(|| anyhow::anyhow!("Missing gcp.service-account"))?; // Path to key file or json content? Usually path or env.

    // object_store crate for GCP usually expects GOOGLE_APPLICATION_CREDENTIALS env or path.
    // Builder has `with_service_account_path` or `with_service_account_key`.
    
    // For now assuming service-account points to a file path
    let builder = object_store::gcp::GoogleCloudStorageBuilder::new()
        .with_bucket_name(bucket)
        .with_service_account_path(service_account);

    Ok(Box::new(builder.build()?))
}
