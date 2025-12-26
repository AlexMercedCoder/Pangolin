use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::optimization_types::{BulkOperationResponse, ValidateNamesResponse};


pub async fn handle_bulk_delete(
    client: &PangolinClient,
    ids: String,
    confirm: bool
) -> Result<(), CliError> {
    let asset_ids: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
    
    if !confirm {
        use dialoguer::Confirm;
        
        println!("About to delete {} assets:", asset_ids.len());
        for id in &asset_ids {
            println!("  - {}", id);
        }
        
        if !Confirm::new()
            .with_prompt("Continue?")
            .interact()
            .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
        {
            println!("Cancelled.");
            return Ok(());
        }
    }
    
    let payload = serde_json::json!({ "asset_ids": asset_ids });
    let res = client.post("/api/v1/bulk/assets/delete", &payload).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Bulk delete failed: {}", res.status())));
    }
    
    let result: BulkOperationResponse = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("Bulk delete completed:");
    println!("  Succeeded: {}", result.succeeded);
    println!("  Failed:    {}", result.failed);
    
    if !result.errors.is_empty() {
        println!("Errors:");
        for error in result.errors {
            println!("  - {}", error);
        }
    }
    
    Ok(())
}

pub async fn handle_validate_names(
    client: &PangolinClient,
    resource_type: String,
    names: Vec<String>
) -> Result<(), CliError> {
    let payload = serde_json::json!({
        "resource_type": resource_type,
        "names": names
    });
    
    let res = client.post("/api/v1/validate/names", &payload).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Validation failed: {}", res.status())));
    }
    
    let result: ValidateNamesResponse = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("Name validation results:");
    for validation in result.results {
        let status = if validation.available { "✓ Available" } else { "✗ Taken" };
        println!("  {}: {}", validation.name, status);
        if let Some(reason) = validation.reason {
            println!("    Reason: {}", reason);
        }
    }
    
    Ok(())
}
