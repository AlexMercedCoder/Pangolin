use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use dialoguer::{Input, Password};
use serde_json::Value;

pub async fn handle_list_warehouses(client: &PangolinClient, limit: Option<usize>, offset: Option<usize>) -> Result<(), CliError> {
    let q = pangolin_cli_common::utils::pagination_query(limit, offset);
    let path = if q.is_empty() { "/api/v1/warehouses".to_string() } else { format!("/api/v1/warehouses?{}", q) };
    let res = client.get(&path).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["name"].as_str().unwrap_or("").to_string(),
        i["storage_type"].as_str().unwrap_or("").to_string()
    ]).collect();
    print_table(vec!["Name", "Type"], rows);
    Ok(())
}

pub async fn handle_create_warehouse(
    client: &PangolinClient, 
    name: String, 
    type_: String,
    bucket_opt: Option<String>,
    access_key_opt: Option<String>,
    secret_key_opt: Option<String>,
    region_opt: Option<String>,
    endpoint_opt: Option<String>,
    properties: Vec<String>
) -> Result<(), CliError> {
    // Check if root
    if client.config.tenant_id.is_none() {
        return Err(CliError::ApiError("Root user cannot create warehouses. Please login as Tenant Admin.".to_string()));
    }

    // Interactive config based on type
    let mut storage_config = serde_json::Map::new();
    
    // Parse generic properties first (can be overridden by explicit args if needed, or act as base)
    for prop in properties {
        if let Some((k, v)) = prop.split_once('=') {
            storage_config.insert(k.to_string(), Value::String(v.to_string()));
        } else {
             println!("Warning: Ignoring invalid property '{}'. format must be key=value", prop);
        }
    }
    
    match type_.as_str() {
        "s3" => {
            let bucket: String = match bucket_opt {
                Some(b) => b,
                None => Input::new().with_prompt("S3 Bucket Name").interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            };
            storage_config.insert("s3.bucket".to_string(), Value::String(bucket));

            let access_key: String = match access_key_opt {
                Some(k) => k,
                None => Input::new().with_prompt("Access Key ID").interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            };
            storage_config.insert("s3.access-key-id".to_string(), Value::String(access_key));

            let secret_key: String = match secret_key_opt {
                Some(k) => k,
                None => Password::new().with_prompt("Secret Access Key").interact().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            };
            storage_config.insert("s3.secret-access-key".to_string(), Value::String(secret_key));

            let region: String = match region_opt {
                Some(r) => r,
                None => Input::new().with_prompt("Region").default("us-east-1".to_string()).interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            };
            storage_config.insert("s3.region".to_string(), Value::String(region));

            let endpoint: String = match endpoint_opt {
                Some(e) => e,
                None => Input::new().with_prompt("Endpoint (Optional, for MinIO)").allow_empty(true).interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            };
            if !endpoint.is_empty() {
                storage_config.insert("s3.endpoint".to_string(), Value::String(endpoint));
            }
        },
        "azure" | "adls" => {
            let account_name: String = Input::new()
                .with_prompt("Azure Storage Account Name")
                .interact_text()
                .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            storage_config.insert("adls.account-name".to_string(), Value::String(account_name));

            let account_key: String = Password::new()
                .with_prompt("Azure Storage Account Key")
                .interact()
                .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            storage_config.insert("adls.account-key".to_string(), Value::String(account_key));

            let container: String = Input::new()
                .with_prompt("Azure Container Name")
                .interact_text()
                .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            storage_config.insert("azure.container".to_string(), Value::String(container));

            // Optional SAS token
            let use_sas: bool = dialoguer::Confirm::new()
                .with_prompt("Use SAS token instead of account key?")
                .default(false)
                .interact()
                .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
            if use_sas {
                let sas_token: String = Password::new()
                    .with_prompt("SAS Token")
                    .interact()
                    .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                storage_config.insert("adls.sas-token".to_string(), Value::String(sas_token));
            }
        },
        "gcs" => {
            let project_id: String = Input::new()
                .with_prompt("GCP Project ID")
                .interact_text()
                .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            storage_config.insert("gcs.project-id".to_string(), Value::String(project_id));

            let bucket: String = match bucket_opt {
                Some(b) => b,
                None => Input::new().with_prompt("GCS Bucket Name").interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            };
            storage_config.insert("gcs.bucket".to_string(), Value::String(bucket));

            // Service account file path
            let sa_file: String = Input::new()
                .with_prompt("Service Account JSON File Path")
                .interact_text()
                .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            storage_config.insert("gcs.service-account-file".to_string(), Value::String(sa_file));
        },
        _ => {
            return Err(CliError::ApiError(format!("Unsupported warehouse type: {}. Use 's3', 'azure', or 'gcs'", type_)));
        }
    }

    let body = serde_json::json!({
        "name": name,
        "storage_config": storage_config,
        "use_sts": false
    });
    
    let res = client.post("/api/v1/warehouses", &body).await?;
    if !res.status().is_success() { 
        let s = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", s, t))); 
    }
    println!("✅ Warehouse '{}' created.", name);
    Ok(())
}

pub async fn handle_delete_warehouse(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/warehouses/{}", name)).await?;
    if !res.status().is_success() {
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to delete warehouse: {} - {}", status, t)));
    }
    println!("✅ Warehouse '{}' deleted.", name);
    Ok(())
}

pub async fn handle_update_warehouse(
    client: &PangolinClient,
    id: String,
    name: Option<String>,
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({});
    
    if let Some(n) = name {
        payload["name"] = serde_json::Value::String(n);
    }
    
    let res = client.put(&format!("/api/v1/warehouses/{}", id), &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update warehouse ({}): {}", status, error_text)));
    }
    
    println!("✅ Warehouse updated successfully!");
    
    Ok(())
}
