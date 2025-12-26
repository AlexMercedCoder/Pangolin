use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use dialoguer::Password;
use serde_json::Value;

pub async fn handle_delete_user(client: &PangolinClient, username: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/users/{}", username)).await?;
    if !res.status().is_success() {
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to delete user: {} - {}", status, t)));
    }
    println!("✅ User '{}' deleted.", username);
    Ok(())
}

pub async fn handle_list_users(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/users").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["username"].as_str().unwrap_or("").to_string(),
        i["role"].as_str().unwrap_or("").to_string()
    ]).collect();
    print_table(vec!["Username", "Role"], rows);
    Ok(())
}

pub async fn handle_create_user(client: &PangolinClient, username: String, email: String, role: Option<String>, password_opt: Option<String>, tenant_id_opt: Option<String>) -> Result<(), CliError> {
    let password = match password_opt {
        Some(p) => p,
        None => Password::new().with_prompt("New User Password").interact()
            .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?,
    };

    let payload = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "role": role.unwrap_or_else(|| "tenant-user".to_string()),
        "tenant_id": tenant_id_opt.or(client.config.tenant_id.clone())
    });

    let res = client.post("/api/v1/users", &payload).await?; 

    if !res.status().is_success() { 
        let s = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", s, t))); 
    }
    println!("✅ User '{}' created.", username);
    Ok(())
}

pub async fn handle_update_user(
    client: &PangolinClient,
    id: String,
    username: Option<String>,
    email: Option<String>,
    active: Option<bool>,
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({});
    
    if let Some(u) = username {
        payload["username"] = serde_json::Value::String(u);
    }
    
    if let Some(e) = email {
        payload["email"] = serde_json::Value::String(e);
    }
    
    if let Some(a) = active {
        payload["active"] = serde_json::Value::Bool(a);
    }
    
    let res = client.put(&format!("/api/v1/users/{}", id), &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update user ({}): {}", status, error_text)));
    }
    
    println!("✅ User updated successfully!");
    
    Ok(())
}
