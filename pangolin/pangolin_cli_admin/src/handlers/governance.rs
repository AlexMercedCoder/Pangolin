use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use serde_json::Value;

// ==================== Permissions ====================

pub async fn handle_list_permissions(client: &PangolinClient, role: Option<String>, user: Option<String>) -> Result<(), CliError> {
    let mut query = String::new();
    
    if let Some(r_name) = role { 
        let role_id = resolve_role_id(client, &r_name).await?;
        query.push_str(&format!("role={}&", role_id)); 
    }
    
    if let Some(u_name) = user { 
        let user_id = resolve_user_id(client, &u_name).await?;
        query.push_str(&format!("user={}&", user_id)); 
    }
    
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

async fn resolve_role_id(client: &PangolinClient, role_name: &str) -> Result<String, CliError> {
    let res = client.get("/api/v1/roles").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Failed to list roles: {}", res.status()))); }
    let roles: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    if let Some(r) = roles.iter().find(|r| r["name"].as_str() == Some(role_name)) {
        if let Some(id) = r["id"].as_str() {
            return Ok(id.to_string());
        }
    }
    Err(CliError::ApiError(format!("Role '{}' not found", role_name)))
}

async fn resolve_user_id(client: &PangolinClient, username: &str) -> Result<String, CliError> {
    let res = client.get("/api/v1/users").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Failed to list users: {}", res.status()))); }
    let users: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    if let Some(u) = users.iter().find(|u| u["username"].as_str() == Some(username)) {
        if let Some(id) = u["id"].as_str() {
            return Ok(id.to_string());
        }
    }
    Err(CliError::ApiError(format!("User '{}' not found", username)))
}

async fn resolve_scope(client: &PangolinClient, resource: &str) -> Result<serde_json::Value, CliError> {
    let parts: Vec<&str> = resource.split(':').collect();
    if parts.len() != 2 {
        return Err(CliError::ApiError("Invalid resource format. Use type:name (e.g. catalog:mycat, namespace:mycat.myns)".to_string()));
    }
    let type_ = parts[0];
    let path = parts[1];

    match type_ {
        "catalog" => {
            let res = client.get("/api/v1/catalogs").await?;
            let catalogs: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
            if let Some(c) = catalogs.iter().find(|c| c["name"].as_str() == Some(path)) {
                let id = c["id"].as_str().unwrap().to_string();
                Ok(serde_json::json!({
                    "type": "catalog",
                    "catalog_id": id
                }))
            } else {
                Err(CliError::ApiError(format!("Catalog '{}' not found", path)))
            }
        },
        "namespace" => {
            let ns_parts: Vec<&str> = path.split('.').collect();
            if ns_parts.len() < 2 { return Err(CliError::ApiError("Invalid namespace format. Use catalog.namespace".to_string())); }
            let cat_name = ns_parts[0];
            let ns_name = ns_parts[1..].join(".");
            
            let res = client.get("/api/v1/catalogs").await?;
            let catalogs: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
            if let Some(c) = catalogs.iter().find(|c| c["name"].as_str() == Some(cat_name)) {
                let id = c["id"].as_str().unwrap().to_string();
                Ok(serde_json::json!({
                    "type": "namespace",
                    "catalog_id": id,
                    "namespace": ns_name
                }))
            } else {
                Err(CliError::ApiError(format!("Catalog '{}' not found", cat_name)))
            }
        },
        "table" => {
            if let Some((cat_name, rest)) = path.split_once('.') {
                if let Some((ns_name, tbl_name)) = rest.rsplit_once('.') {
                    let res = client.get("/api/v1/catalogs").await?;
                    let catalogs: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
                    let cat_id = if let Some(c) = catalogs.iter().find(|c| c["name"].as_str() == Some(cat_name)) {
                        c["id"].as_str().unwrap().to_string()
                    } else {
                        return Err(CliError::ApiError(format!("Catalog '{}' not found", cat_name)));
                    };
                    
                    let url = format!("/v1/{}/namespaces/{}/tables/{}", cat_name, ns_name, tbl_name);
                    let res = client.get(&url).await?;
                    if !res.status().is_success() {
                        return Err(CliError::ApiError(format!("Failed to resolve table: {}", res.status())));
                    }
                    let body: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
                    
                    if let Some(uuid) = body.pointer("/metadata/table-uuid").and_then(|v| v.as_str()) {
                         Ok(serde_json::json!({
                            "type": "asset",
                            "catalog_id": cat_id,
                            "namespace": ns_name,
                            "asset_id": uuid
                        }))
                    } else {
                        Err(CliError::ApiError("Could not find table UUID in response".to_string()))
                    }
                } else {
                     Err(CliError::ApiError("Invalid table format. Use catalog.namespace.table".to_string())) 
                }
            } else {
                 Err(CliError::ApiError("Invalid format".to_string()))
            }
        },
        _ => Err(CliError::ApiError(format!("Unsupported resource type: {}", type_)))
    }
}

pub async fn handle_grant_permission(client: &PangolinClient, username: String, action: String, resource: String) -> Result<(), CliError> {
    if client.config.tenant_id.is_none() {
        return Err(CliError::ApiError("Root user cannot grant granular permissions. Please login as Tenant Admin.".to_string()));
    }

    let user_id = resolve_user_id(client, &username).await?;
    let scope = resolve_scope(client, &resource).await?;
    
    let actions: Vec<String> = action.split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();
        
    let body = serde_json::json!({
        "user-id": user_id,
        "scope": scope,
        "actions": actions
    });
    
    let res = client.post("/api/v1/permissions", &body).await?;
    if !res.status().is_success() {
         let s = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Error: {} - {}", s, t))); 
    }
    println!("✅ Permission granted to user '{}' on '{}'", username, resource);
    Ok(())
}

pub async fn handle_revoke_permission(client: &PangolinClient, role: String, action: String, resource: String) -> Result<(), CliError> {
    let url = format!("/api/v1/permissions?role={}&action={}&resource={}", role, action, resource);
    let res = client.delete(&url).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    println!("✅ Permission revoked.");
    Ok(())
}

// ==================== Metadata ====================

pub async fn handle_get_metadata(client: &PangolinClient, _entity_type: String, entity_id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/assets/{}/metadata", entity_id)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    
    let response: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    if let Some(metadata) = response.get("metadata") {
        println!("{}", serde_json::to_string_pretty(metadata).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&response).unwrap());
    }
    Ok(())
}

pub async fn handle_set_metadata(client: &PangolinClient, _entity_type: String, entity_id: String, key: String, value: String) -> Result<(), CliError> {
    let get_res = client.get(&format!("/api/v1/assets/{}/metadata", entity_id)).await?;
    
    let current_metadata = if get_res.status().is_success() {
        let body: Value = get_res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
        body.get("metadata").cloned().unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let description = current_metadata.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let tags = current_metadata.get("tags").and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| Vec::<String>::new());
    let discoverable = current_metadata.get("discoverable").and_then(|v| v.as_bool()).unwrap_or(false);
    
    let mut properties = current_metadata.get("properties").cloned().unwrap_or(serde_json::json!({}));
    
    let json_value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
    
    if let Some(obj) = properties.as_object_mut() {
        obj.insert(key, json_value);
    } else {
        properties = serde_json::json!({ key: json_value });
    }

    let payload = serde_json::json!({
        "description": description,
        "tags": tags,
        "properties": properties,
        "discoverable": discoverable
    });

    let res = client.post(&format!("/api/v1/assets/{}/metadata", entity_id), &payload).await?;
    if !res.status().is_success() { 
        let s = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("Error: {} - {}", s, t))); 
    }
    println!("✅ Metadata updated.");
    Ok(())
}

pub async fn handle_delete_metadata(client: &PangolinClient, asset_id: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/business-metadata/{}", asset_id)).await?;
    if !res.status().is_success() {
         return Err(CliError::ApiError(format!("Failed to delete metadata: {}", res.status())));
    }
    println!("✅ Business metadata deleted for asset {}", asset_id);
    Ok(())
}

// ==================== Access Requests ====================

pub async fn handle_request_access(
    client: &PangolinClient, 
    asset_id: String, 
    reason: String // Changed signature slightly to match handlers.rs (reason instead of permission/reason)? 
    // Wait, handlers.rs had handle_request_access with reason. My previous governance.rs had permission + reason.
    // Let's stick to what main.rs will call.
    // main.rs calls: handlers::handle_request_access(&client, asset_id, reason).
    // So permission arg is NOT in main.rs invocation for RequestAccess command.
    // I should remove `permission` arg from here.
) -> Result<(), CliError> {
    let payload = serde_json::json!({
        "asset_id": asset_id,
        "reason": reason
    });
    let res = client.post("/api/v1/access-requests", &payload).await?;
    if !res.status().is_success() {
         return Err(CliError::ApiError(format!("Failed: {}", res.status())));
    }
    println!("✅ Access request submitted.");
    Ok(())
}

pub async fn handle_list_access_requests(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/access-requests").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["id"].as_str().unwrap_or("-").to_string(),
        i["username"].as_str().unwrap_or("-").to_string(), 
        i["asset_id"].as_str().unwrap_or("-").to_string(),
        i["status"].as_str().unwrap_or("-").to_string(),
        i["reason"].as_str().unwrap_or("-").to_string(),
    ]).collect();
    print_table(vec!["ID", "User", "Asset", "Status", "Reason"], rows);
    Ok(())
}

pub async fn handle_update_access_request(
    client: &PangolinClient, 
    id: String, 
    status: String, 
) -> Result<(), CliError> {
    let payload = serde_json::json!({
        "status": status,
        "id": id 
    });
    let res = client.put(&format!("/api/v1/access-requests/{}", id), &payload).await?;
    if !res.status().is_success() {
         return Err(CliError::ApiError(format!("Failed to update request: {}", res.status())));
    }
    println!("✅ Access request updated to {}", status);
    Ok(())
}
