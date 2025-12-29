use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use serde_json::Value;

pub async fn handle_create_service_user(
    client: &PangolinClient,
    name: String,
    description: Option<String>,
    role: String,
    expires_in_days: Option<i64>,
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({
        "name": name,
        "role": role,
    });
    
    if let Some(desc) = description {
        payload["description"] = serde_json::Value::String(desc);
    }
    
    if let Some(days) = expires_in_days {
        payload["expires_in_days"] = serde_json::Value::Number(days.into());
    }
    
    let res = client.post("/api/v1/service-users", &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to create service user ({}): {}", status, error_text)));
    }
    
    let result: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("✅ Service user created successfully!");
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚠️  IMPORTANT: Save this API key - it will not be shown again!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let id = result["id"].as_str().or_else(|| result["service_user_id"].as_str()).unwrap_or("-");
    println!("Service User ID: {}", id);
    println!("Name: {}", result["name"].as_str().unwrap_or("-"));
    println!("API Key: {}", result["api_key"].as_str().unwrap_or("-"));
    if let Some(expires) = result["expires_at"].as_str() {
        println!("Expires At: {}", expires);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}

pub async fn handle_list_service_users(client: &PangolinClient, limit: Option<usize>, offset: Option<usize>) -> Result<(), CliError> {
    let q = pangolin_cli_common::utils::pagination_query(limit, offset);
    let path = if q.is_empty() { "/api/v1/service-users".to_string() } else { format!("/api/v1/service-users?{}", q) };
    let res = client.get(&path).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to list service users: {}", res.status())));
    }
    
    let service_users: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    if service_users.is_empty() {
        println!("No service users found.");
        return Ok(());
    }
    
    let rows: Vec<Vec<String>> = service_users.iter().map(|su| {
        vec![
            su["id"].as_str().unwrap_or("-").to_string(),
            su["name"].as_str().unwrap_or("-").to_string(),
            su["role"].as_str().unwrap_or("-").to_string(),
            su["active"].as_bool().map(|a| if a { "✓" } else { "✗" }).unwrap_or("-").to_string(),
            su["last_used"].as_str().unwrap_or("Never").to_string(),
            su["expires_at"].as_str().unwrap_or("Never").to_string(),
        ]
    }).collect();
    
    print_table(
        vec!["ID", "Name", "Role", "Active", "Last Used", "Expires At"],
        rows
    );
    
    Ok(())
}

pub async fn handle_get_service_user(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/service-users/{}", id)).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to get service user: {}", res.status())));
    }
    
    let su: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Service User Details");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ID: {}", su["id"].as_str().unwrap_or("-"));
    println!("Name: {}", su["name"].as_str().unwrap_or("-"));
    if let Some(desc) = su["description"].as_str() {
        println!("Description: {}", desc);
    }
    println!("Role: {}", su["role"].as_str().unwrap_or("-"));
    println!("Active: {}", if su["active"].as_bool().unwrap_or(false) { "Yes" } else { "No" });
    println!("Created At: {}", su["created_at"].as_str().unwrap_or("-"));
    println!("Last Used: {}", su["last_used"].as_str().unwrap_or("Never"));
    if let Some(expires) = su["expires_at"].as_str() {
        println!("Expires At: {}", expires);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}

pub async fn handle_update_service_user(
    client: &PangolinClient,
    id: String,
    name: Option<String>,
    description: Option<String>,
    active: Option<bool>,
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({});
    
    if let Some(n) = name {
        payload["name"] = serde_json::Value::String(n);
    }
    
    if let Some(desc) = description {
        payload["description"] = serde_json::Value::String(desc);
    }
    
    if let Some(a) = active {
        payload["active"] = serde_json::Value::Bool(a);
    }
    
    let res = client.put(&format!("/api/v1/service-users/{}", id), &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update service user ({}): {}", status, error_text)));
    }
    
    println!("✅ Service user updated successfully!");
    
    Ok(())
}

pub async fn handle_delete_service_user(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/service-users/{}", id)).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to delete service user: {}", res.status())));
    }
    
    println!("✅ Service user deleted successfully!");
    
    Ok(())
}

pub async fn handle_rotate_service_user_key(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/service-users/{}/rotate", id), &serde_json::json!({})).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to rotate API key ({}): {}", status, error_text)));
    }
    
    let result: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("✅ API key rotated successfully!");
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚠️  IMPORTANT: Save this new API key - it will not be shown again!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let id = result["id"].as_str().or_else(|| result["service_user_id"].as_str()).unwrap_or("-");
    println!("Service User ID: {}", id);
    println!("Name: {}", result["name"].as_str().unwrap_or("-"));
    println!("New API Key: {}", result["api_key"].as_str().unwrap_or("-"));
    if let Some(expires) = result["expires_at"].as_str() {
        println!("Expires At: {}", expires);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}
