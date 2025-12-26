use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use serde_json::Value;

pub async fn handle_revoke_token(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.post("/api/v1/tokens/revoke", &serde_json::json!({})).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to revoke token ({}): {}", status, error_text)));
    }
    
    println!("✅ Token revoked successfully!");
    println!("You have been logged out. Please login again to continue.");
    
    Ok(())
}

pub async fn handle_revoke_token_by_id(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/tokens/revoke/{}", id), &serde_json::json!({})).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to revoke token ({}): {}", status, error_text)));
    }
    
    println!("✅ Token {} revoked successfully!", id);
    
    Ok(())
}

pub async fn handle_list_user_tokens(client: &PangolinClient, user_id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/users/{}/tokens", user_id)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to list tokens ({}): {}", status, error_text)));
    }
    
    let tokens: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    if tokens.is_empty() {
        println!("No active tokens found for user {}", user_id);
        return Ok(());
    }
    
    let rows: Vec<Vec<String>> = tokens.iter().map(|t| {
        vec![
            t["token_id"].as_str().unwrap_or("-").to_string(),
            t["created_at"].as_str().unwrap_or("-").to_string(),
            t["expires_at"].as_str().unwrap_or("-").to_string(),
            t["is_valid"].as_bool().map(|b| if b { "Yes" } else { "No" }).unwrap_or("-").to_string(),
        ]
    }).collect();
    
    print_table(vec!["Token ID", "Created At", "Expires At", "Valid"], rows);
    Ok(())
}

pub async fn handle_delete_token(client: &PangolinClient, token_id: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/tokens/{}", token_id)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to delete token ({}): {}", status, error_text)));
    }
    
    println!("✅ Token {} deleted successfully!", token_id);
    Ok(())
}
