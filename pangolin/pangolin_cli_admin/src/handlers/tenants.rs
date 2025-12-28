use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use serde_json::Value;

pub async fn handle_list_tenants(client: &PangolinClient, limit: Option<usize>, offset: Option<usize>) -> Result<(), CliError> {
    let q = pangolin_cli_common::utils::pagination_query(limit, offset);
    let path = if q.is_empty() { "/api/v1/tenants".to_string() } else { format!("/api/v1/tenants?{}", q) };
    let res = client.get(&path).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to list tenants: {}", res.status())));
    }
    
    let tenants: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    let rows: Vec<Vec<String>> = tenants.iter().map(|t| {
        vec![
            t["id"].as_str().unwrap_or("-").to_string(),
            t["name"].as_str().unwrap_or("-").to_string()
        ]
    }).collect();
    
    print_table(vec!["ID", "Name"], rows);
    Ok(())
}

pub async fn handle_create_tenant(client: &PangolinClient, name: String, admin_username: Option<String>, admin_password: Option<String>) -> Result<(), CliError> {
    let mut payload = serde_json::Map::new();
    payload.insert("name".to_string(), Value::String(name.clone()));
    payload.insert("properties".to_string(), serde_json::json!({}));
    
    if let Some(u) = &admin_username {
        payload.insert("admin_username".to_string(), Value::String(u.clone()));
    }
    if let Some(p) = &admin_password {
        payload.insert("admin_password".to_string(), Value::String(p.clone()));
    }
    
    let body = Value::Object(payload);

    let res = client.post("/api/v1/tenants", &body).await?;

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("Failed to create tenant: {} - {}", status, error_text)));
    }

    let created_tenant: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let tenant_id = created_tenant["id"].as_str().ok_or(CliError::ApiError("Failed to parse tenant ID".to_string()))?;

    println!("✅ Tenant '{}' created successfully (ID: {}).", name, tenant_id);

    if let (Some(u), Some(p)) = (admin_username, admin_password) {
        println!("Creating admin user '{}'...", u);
        let mut user_payload = serde_json::Map::new();
        user_payload.insert("username".to_string(), Value::String(u.clone()));
        user_payload.insert("email".to_string(), Value::String(format!("{}@example.com", u)));
        user_payload.insert("password".to_string(), Value::String(p));
        user_payload.insert("role".to_string(), Value::String("tenant-admin".to_string()));
        user_payload.insert("tenant_id".to_string(), Value::String(tenant_id.to_string()));

        let user_res = client.post("/api/v1/users", &Value::Object(user_payload)).await?;
        if !user_res.status().is_success() {
             let status = user_res.status();
             let text = user_res.text().await.unwrap_or_default();
             println!("⚠️ Tenant created, but failed to create admin user: {} - {}", status, text);
        } else {
             println!("✅ Tenant Admin user created.");
        }
    }

    Ok(())
}

pub async fn handle_delete_tenant(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/tenants/{}", id)).await?;
    if !res.status().is_success() {
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to delete tenant: {} - {}", status, t)));
    }
    println!("✅ Tenant '{}' deleted.", id);
    Ok(())
}

pub async fn handle_update_tenant(
    client: &PangolinClient,
    id: String,
    name: Option<String>,
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({});
    
    if let Some(n) = name {
        payload["name"] = serde_json::Value::String(n);
    }
    
    let res = client.put(&format!("/api/v1/tenants/{}", id), &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update tenant ({}): {}", status, error_text)));
    }
    
    println!("✅ Tenant updated successfully!");
    
    Ok(())
}
