use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::utils::print_table;
use pangolin_cli_common::error::CliError;
use dialoguer::{Input, Password};
use serde_json::Value;

pub async fn handle_login(client: &mut PangolinClient, username_opt: Option<String>, password_opt: Option<String>) -> Result<(), CliError> {
    let username = match username_opt {
        Some(u) => u,
        None => Input::new().with_prompt("Username").interact_text()
            .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?,
    };

    let password = match password_opt {
        Some(p) => p,
        None => Password::new().with_prompt("Password").interact()
            .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?,
    };

    client.login(&username, &password).await?;
    println!("✅ Logged in successfully as {}", username);
    Ok(())
}

pub async fn handle_list_tenants(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/tenants").await?;
    
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

pub async fn handle_create_tenant(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let body = serde_json::json!({
        "name": name,
        "properties": {}
    });

    let res = client.post("/api/v1/tenants", &body).await?;

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("Failed to create tenant: {} - {}", status, error_text)));
    }

    println!("✅ Tenant '{}' created successfully.", name);
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

// --- Users ---
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

    let res = client.post("/api/v1/users", &payload).await?; // Assuming endpoint is /api/v1/users based on search

    if !res.status().is_success() { 
        let s = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", s, t))); 
    }
    println!("✅ User '{}' created.", username);
    Ok(())
}

// --- Warehouses ---
pub async fn handle_list_warehouses(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/warehouses").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["name"].as_str().unwrap_or("").to_string(),
        i["storage_type"].as_str().unwrap_or("").to_string()
    ]).collect();
    print_table(vec!["Name", "Type"], rows);
    Ok(())
}

pub async fn handle_create_warehouse(client: &PangolinClient, name: String, type_: String) -> Result<(), CliError> {
    // Interactive config based on type
    let mut config = serde_json::Map::new();
    if type_ == "s3" {
         let bucket: String = Input::new().with_prompt("Bucket Name").interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?; 
         config.insert("bucket".to_string(), Value::String(bucket));
         // ... more prompts
    }

    let body = serde_json::json!({
        "name": name,
        "storage_type": type_,
        "config": config
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

// --- Catalogs ---
pub async fn handle_list_catalogs(client: &PangolinClient) -> Result<(), CliError> {
     let res = client.get("/api/v1/catalogs").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["name"].as_str().unwrap_or("").to_string(),
        i["type"].as_str().unwrap_or("").to_string()
    ]).collect();
    print_table(vec!["Name", "Type"], rows);
    Ok(())
}

pub async fn handle_create_catalog(client: &PangolinClient, name: String, warehouse: String) -> Result<(), CliError> {
    let body = serde_json::json!({
        "name": name,
        "warehouse": warehouse,
        "type": "pangea" // defaulting to internal type
    });
    
    let res = client.post("/api/v1/catalogs", &body).await?;
    if !res.status().is_success() { 
        let s = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", s, t))); 
    }
    println!("✅ Catalog '{}' created.", name);
    Ok(())
}

pub async fn handle_delete_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/catalogs/{}", name)).await?;
    if !res.status().is_success() {
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to delete catalog: {} - {}", status, t)));
    }
    println!("✅ Catalog '{}' deleted.", name);
    Ok(())
}

// --- Governance: Permissions ---
pub async fn handle_list_permissions(client: &PangolinClient, role: Option<String>, user: Option<String>) -> Result<(), CliError> {
    // Construct query parameters
    let mut query = String::new();
    if let Some(r) = role { query.push_str(&format!("role={}&", r)); }
    if let Some(u) = user { query.push_str(&format!("user={}&", u)); }
    
    let res = client.get(&format!("/api/v1/permissions?{}", query)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["role"].as_str().unwrap_or("").to_string(),
        i["action"].as_str().unwrap_or("").to_string(),
        i["resource"].as_str().unwrap_or("").to_string(),
    ]).collect();
    print_table(vec!["Role", "Action", "Resource"], rows);
    Ok(())
}

pub async fn handle_grant_permission(client: &PangolinClient, role: String, action: String, resource: String) -> Result<(), CliError> {
    let body = serde_json::json!({
        "role": role,
        "action": action,
        "resource": resource
    });
    let res = client.post("/api/v1/permissions", &body).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    println!("✅ Permission granted.");
    Ok(())
}

pub async fn handle_revoke_permission(client: &PangolinClient, role: String, action: String, resource: String) -> Result<(), CliError> {
    // Assuming DELETE with body or query params. Using query params for simplicity as standard REST often uses ID.
    // However, for revocation by tuple, POST /revoke or DELETE with body is common.
    // Let's assume DELETE /api/v1/permissions?role=..&action=..&resource=..
    let url = format!("/api/v1/permissions?role={}&action={}&resource={}", role, action, resource);
    let res = client.delete(&url).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    println!("✅ Permission revoked.");
    Ok(())
}

// --- Governance: Metadata ---
pub async fn handle_get_metadata(client: &PangolinClient, entity_type: String, entity_id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/metadata/{}/{}", entity_type, entity_id)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    
    let metadata: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
    Ok(())
}

pub async fn handle_set_metadata(client: &PangolinClient, entity_type: String, entity_id: String, key: String, value: String) -> Result<(), CliError> {
    let body = serde_json::json!({
        "key": key,
        "value": value
    });
    let res = client.post(&format!("/api/v1/metadata/{}/{}", entity_type, entity_id), &body).await?;
     if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    println!("✅ Metadata set.");
    Ok(())
}
