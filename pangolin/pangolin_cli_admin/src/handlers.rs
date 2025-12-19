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

    // Reset tenant context before login to ensure fresh state
    client.config.tenant_id = None;
    client.config.tenant_name = None;

    client.login(&username, &password).await?;
    
    // If tenant_id was set by login (Tenant Admin), try to fetch name
    if let Some(_) = client.config.tenant_id {
         // Try to fetch accessible tenants to resolve name
         // For Tenant Admin, /api/v1/tenants might return their own tenant
         if let Ok(res_tenants) = client.get("/api/v1/tenants").await {
             if res_tenants.status().is_success() {
                  if let Ok(tenants) = res_tenants.json::<Vec<Value>>().await {
                      if let Some(my_id) = &client.config.tenant_id {
                          if let Some(t) = tenants.iter().find(|t| t["id"].as_str() == Some(my_id)) {
                              if let Some(name) = t["name"].as_str() {
                                  client.config.tenant_name = Some(name.to_string());
                              }
                          }
                      }
                  }
             }
         }
    } else {
        // Root user login - clear any previous context
        client.config.tenant_id = None;
        client.config.tenant_name = None;
    }
    
    println!("✅ Logged in successfully as {}", username);
    
    Ok(())
}

pub async fn handle_use(client: &mut PangolinClient, name: String) -> Result<(), CliError> {
    // 1. List tenants to find the ID
    let res = client.get("/api/v1/tenants").await?;
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to fetch tenants: {}", res.status())));
    }
    
    let tenants: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    // 2. Find matching name
    let found = tenants.iter().find(|t| t["name"].as_str() == Some(&name));
    
    if let Some(tenant) = found {
        if let Some(id) = tenant["id"].as_str() {
            client.config.tenant_id = Some(id.to_string());
            client.config.tenant_name = Some(name.clone());
            println!("✅ Switched context to tenant '{}' ({})", name, id);
        } else {
             return Err(CliError::ApiError("Tenant found but has no ID".to_string()));
        }
    } else {
        return Err(CliError::ApiError(format!("Tenant '{}' not found", name)));
    }
    
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

pub async fn handle_create_tenant(client: &PangolinClient, name: String, admin_username: Option<String>, admin_password: Option<String>) -> Result<(), CliError> {
    let mut payload = serde_json::Map::new();
    payload.insert("name".to_string(), Value::String(name.clone()));
    payload.insert("properties".to_string(), serde_json::json!({}));
    
    if let Some(u) = admin_username {
        payload.insert("admin_username".to_string(), Value::String(u));
    }
    if let Some(p) = admin_password {
        payload.insert("admin_password".to_string(), Value::String(p));
    }
    
    let body = Value::Object(payload);

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

pub async fn handle_create_warehouse(
    client: &PangolinClient, 
    name: String, 
    type_: String,
    bucket_opt: Option<String>,
    access_key_opt: Option<String>,
    secret_key_opt: Option<String>,
    region_opt: Option<String>,
    endpoint_opt: Option<String>
) -> Result<(), CliError> {
    // Check if root
    if client.config.tenant_id.is_none() {
        return Err(CliError::ApiError("Root user cannot create warehouses. Please login as Tenant Admin.".to_string()));
    }

    // Interactive config based on type
    let mut config = serde_json::Map::new();
    if type_ == "s3" {
         let bucket: String = match bucket_opt {
             Some(b) => b,
             None => Input::new().with_prompt("Bucket Name").interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
         };
         config.insert("bucket".to_string(), Value::String(bucket));

         let access_key: String = match access_key_opt {
             Some(k) => k,
             None => Input::new().with_prompt("Access Key").interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
         };
         config.insert("access_key".to_string(), Value::String(access_key));

         let secret_key: String = match secret_key_opt {
             Some(k) => k,
             None => Password::new().with_prompt("Secret Key").interact().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
         };
         config.insert("secret_key".to_string(), Value::String(secret_key));

         let region: String = match region_opt {
             Some(r) => r,
             None => Input::new().with_prompt("Region").default("us-east-1".to_string()).interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
         };
         config.insert("region".to_string(), Value::String(region));

         let endpoint: String = match endpoint_opt {
             Some(e) => e,
             None => Input::new().with_prompt("Endpoint (Optional)").allow_empty(true).interact_text().map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
         };
         if !endpoint.is_empty() {
            config.insert("endpoint".to_string(), Value::String(endpoint));
         }
    }
    // Add logic for other types if needed (filesystem, etc)

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
    if client.config.tenant_id.is_none() {
        return Err(CliError::ApiError("Root user cannot create catalogs. Please login as Tenant Admin.".to_string()));
    }

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
    
    // Find user
    // Note: The /api/v1/users endpoint returns objects with "username" and "id" (if we added ID exposing to API, checking handler...)
    // Wait, the `list_users` handler in API usually returns User objects. Core User struct has ID.
    // Let's assume it has "id".
    // If not, we might need lookup by username endpoint?
    // Let's check `pangolin_api/src/user_handlers.rs`... User struct has id.
    
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
            // path is catalog_name
            let res = client.get("/api/v1/catalogs").await?; // Better lookup?
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
            // path is catalog.namespace
            let ns_parts: Vec<&str> = path.split('.').collect();
            if ns_parts.len() < 2 { return Err(CliError::ApiError("Invalid namespace format. Use catalog.namespace".to_string())); }
            let cat_name = ns_parts[0];
            let ns_name = ns_parts[1..].join("."); // Handle nested?
            
            // Resolve Catalog ID
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
            // path is catalog.namespace.table
            // This is tricky if namespace has dots. Assuming simple 3 parts for MVP or first dot is cat.
            // Helper: parse first component as catalog.
            if let Some((cat_name, rest)) = path.split_once('.') {
                if let Some((ns_name, tbl_name)) = rest.rsplit_once('.') { // Namespace might have dots, split at LAST dot for table?
                    // Resolve Catalog ID
                    let res = client.get("/api/v1/catalogs").await?;
                    let catalogs: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
                    let cat_id = if let Some(c) = catalogs.iter().find(|c| c["name"].as_str() == Some(cat_name)) {
                        c["id"].as_str().unwrap().to_string()
                    } else {
                        return Err(CliError::ApiError(format!("Catalog '{}' not found", cat_name)));
                    };
                    
                    // Resolve Table ID by fetching metadata
                    // GET /v1/{prefix}/namespaces/{namespace}/tables/{table}
                    let url = format!("/v1/{}/namespaces/{}/tables/{}", cat_name, ns_name, tbl_name);
                    let res = client.get(&url).await?;
                    if !res.status().is_success() {
                        return Err(CliError::ApiError(format!("Failed to resolve table: {}", res.status())));
                    }
                    let body: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
                    
                    // TableResponse -> metadata -> table-uuid
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

    // 1. Resolve User ID
    let user_id = resolve_user_id(client, &username).await?;
    
    // 2. Resolve Scope
    let scope = resolve_scope(client, &resource).await?;
    
    // 3. Parse Actions
    // Allow comma separated: "read,write"
    let actions: Vec<String> = action.split(',')
        .map(|s| s.trim().to_lowercase()) // normalize?
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
pub async fn handle_get_metadata(client: &PangolinClient, _entity_type: String, entity_id: String) -> Result<(), CliError> {
    // API uses asset ID directly
    let res = client.get(&format!("/api/v1/assets/{}/metadata", entity_id)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    
    let response: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    // Response body is { "metadata": { ... } }
    if let Some(metadata) = response.get("metadata") {
        println!("{}", serde_json::to_string_pretty(metadata).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&response).unwrap());
    }
    Ok(())
}

pub async fn handle_set_metadata(client: &PangolinClient, _entity_type: String, entity_id: String, key: String, value: String) -> Result<(), CliError> {
    // 1. Fetch existing metadata to preserve other fields
    let get_res = client.get(&format!("/api/v1/assets/{}/metadata", entity_id)).await?;
    
    let mut current_metadata = if get_res.status().is_success() {
        let body: Value = get_res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
        body.get("metadata").cloned().unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 2. Prepare defaults if missing
    let description = current_metadata.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let tags = current_metadata.get("tags").and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| Vec::<String>::new());
    let discoverable = current_metadata.get("discoverable").and_then(|v| v.as_bool()).unwrap_or(false);
    
    let mut properties = current_metadata.get("properties").cloned().unwrap_or(serde_json::json!({}));
    
    // 3. Update property (Try to parse as JSON, else String)
    let json_value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
    
    if let Some(obj) = properties.as_object_mut() {
        obj.insert(key, json_value);
    } else {
        // If properties was null or not an object, make it one
        properties = serde_json::json!({ key: json_value });
    }

    // 4. Construct Payload
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
// --- Federated Catalogs ---
pub async fn handle_create_federated_catalog(
    client: &PangolinClient,
    name: String,
    base_url: String,
    storage_location: String,
    auth_type: String,
    token: Option<String>,
    username: Option<String>,
    password: Option<String>,
    api_key: Option<String>,
    timeout: u32,
) -> Result<(), CliError> {
    if client.config.tenant_id.is_none() {
        return Err(CliError::ApiError("Root user cannot create federated catalogs. Please login as Tenant Admin.".to_string()));
    }

    // Build credentials based on auth_type
    let credentials = match auth_type.as_str() {
        "BearerToken" => {
            let t = token.ok_or_else(|| CliError::ApiError("--token required for BearerToken auth".to_string()))?;
            serde_json::json!({ "token": t })
        },
        "BasicAuth" => {
            let u = username.ok_or_else(|| CliError::ApiError("--username required for BasicAuth".to_string()))?;
            let p = password.ok_or_else(|| CliError::ApiError("--password required for BasicAuth".to_string()))?;
            serde_json::json!({ "username": u, "password": p })
        },
        "ApiKey" => {
            let k = api_key.ok_or_else(|| CliError::ApiError("--api-key required for ApiKey auth".to_string()))?;
            serde_json::json!({ "api_key": k })
        },
        "None" => serde_json::json!({}),
        _ => return Err(CliError::ApiError(format!("Invalid auth_type: {}. Use None, BasicAuth, BearerToken, or ApiKey", auth_type))),
    };

    let body = serde_json::json!({
        "name": name,
        "catalog_type": "Federated",
        "storage_location": storage_location,
        "federated_config": {
            "base_url": base_url,
            "auth_type": auth_type,
            "credentials": credentials,
            "timeout_seconds": timeout
        }
    });

    let res = client.post("/api/v1/catalogs", &body).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", status, text)));
    }

    println!("✅ Federated catalog '{}' created successfully!", name);
    println!("   Base URL: {}", base_url);
    println!("   Auth Type: {}", auth_type);
    Ok(())
}

pub async fn handle_list_federated_catalogs(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/catalogs").await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Error: {}", res.status())));
    }
    
    let catalogs: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    // Filter for federated catalogs
    let federated: Vec<&Value> = catalogs
        .iter()
        .filter(|c| c["catalog_type"].as_str() == Some("Federated"))
        .collect();
    
    if federated.is_empty() {
        println!("No federated catalogs found.");
        return Ok(());
    }
    
    let rows: Vec<Vec<String>> = federated.iter().map(|c| {
        let config = &c["federated_config"];
        vec![
            c["name"].as_str().unwrap_or("-").to_string(),
            config["base_url"].as_str().unwrap_or("-").to_string(),
            config["auth_type"].as_str().unwrap_or("-").to_string(),
        ]
    }).collect();
    
    print_table(vec!["Name", "Base URL", "Auth Type"], rows);
    Ok(())
}

pub async fn handle_delete_federated_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    use dialoguer::Confirm;
    
    if !Confirm::new()
        .with_prompt(format!("Are you sure you want to delete federated catalog '{}'?", name))
        .interact()
        .map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    {
        println!("Cancelled.");
        return Ok(());
    }

    let res = client.delete(&format!("/api/v1/catalogs/{}", name)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", status, text)));
    }
    
    println!("✅ Federated catalog '{}' deleted successfully!", name);
    Ok(())
}

pub async fn handle_test_federated_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    println!("Testing connection to federated catalog '{}'...\n", name);
    
    // Test 1: Get catalog config
    print!("1. Checking catalog configuration... ");
    let res = client.get(&format!("/api/v1/catalogs")).await?;
    if !res.status().is_success() {
        println!("❌");
        return Err(CliError::ApiError(format!("Failed to fetch catalogs: {}", res.status())));
    }
    
    let catalogs: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let catalog = catalogs.iter().find(|c| c["name"].as_str() == Some(&name));
    
    if catalog.is_none() {
        println!("❌");
        return Err(CliError::ApiError(format!("Catalog '{}' not found", name)));
    }
    println!("✅");
    
    // Test 2: List namespaces
    print!("2. Testing namespace listing... ");
    let res = client.get(&format!("/v1/{}/namespaces", name)).await?;
    
    if !res.status().is_success() {
        println!("❌");
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("Failed to list namespaces: {} - {}", status, text)));
    }
    
    let namespaces: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let ns_list = namespaces["namespaces"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_array())
                .filter_map(|arr| arr.first())
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    
    println!("✅");
    
    // Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Federated catalog '{}' is working correctly!", name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Namespaces accessible: {:?}", ns_list);
    
    Ok(())
}

// ==================== Service User Management ====================

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
    println!("Service User ID: {}", result["service_user_id"].as_str().unwrap_or("-"));
    println!("Name: {}", result["name"].as_str().unwrap_or("-"));
    println!("API Key: {}", result["api_key"].as_str().unwrap_or("-"));
    if let Some(expires) = result["expires_at"].as_str() {
        println!("Expires At: {}", expires);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}

pub async fn handle_list_service_users(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/service-users").await?;
    
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
    
    let mut res = client.put(&format!("/api/v1/service-users/{}", id), &payload).await?;
    
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
    println!("Service User ID: {}", result["service_user_id"].as_str().unwrap_or("-"));
    println!("Name: {}", result["name"].as_str().unwrap_or("-"));
    println!("New API Key: {}", result["api_key"].as_str().unwrap_or("-"));
    if let Some(expires) = result["expires_at"].as_str() {
        println!("Expires At: {}", expires);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}

// ==================== Update Operations ====================

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

pub async fn handle_update_catalog(
    client: &PangolinClient,
    id: String,
    name: Option<String>,
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({});
    
    if let Some(n) = name {
        payload["name"] = serde_json::Value::String(n);
    }
    
    let res = client.put(&format!("/api/v1/catalogs/{}", id), &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update catalog ({}): {}", status, error_text)));
    }
    
    println!("✅ Catalog updated successfully!");
    
    Ok(())
}

// ==================== Token Management ====================

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

// ==================== System Configuration ====================

pub async fn handle_get_system_settings(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/config/settings").await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to get system settings ({}): {}", status, error_text)));
    }
    
    let settings: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    println!("{}", serde_json::to_string_pretty(&settings).unwrap());
    Ok(())
}

pub async fn handle_update_system_settings(
    client: &PangolinClient,
    allow_public_signup: Option<bool>,
    default_warehouse_bucket: Option<String>,
    default_retention_days: Option<i32>,
) -> Result<(), CliError> {
    let mut payload = serde_json::Map::new();
    
    if let Some(signup) = allow_public_signup {
        payload.insert("allow_public_signup".to_string(), Value::Bool(signup));
    }
    if let Some(bucket) = default_warehouse_bucket {
        payload.insert("default_warehouse_bucket".to_string(), Value::String(bucket));
    }
    if let Some(days) = default_retention_days {
        payload.insert("default_retention_days".to_string(), Value::Number(days.into()));
    }
    
    let res = client.put("/api/v1/config/settings", &Value::Object(payload)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update system settings ({}): {}", status, error_text)));
    }
    
    println!("✅ System settings updated successfully!");
    Ok(())
}

// ==================== Federated Catalog Operations ====================

pub async fn handle_sync_federated_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    println!("Triggering sync for federated catalog '{}'...", name);
    
    let res = client.post(&format!("/api/v1/federated-catalogs/{}/sync", name), &serde_json::json!({})).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to sync catalog ({}): {}", status, error_text)));
    }
    
    println!("✅ Sync triggered successfully for catalog '{}'!", name);
    Ok(())
}

pub async fn handle_get_federated_stats(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/federated-catalogs/{}/stats", name)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to get stats ({}): {}", status, error_text)));
    }
    
    let stats: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("Federated Catalog Stats for '{}':", name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Last Synced: {}", stats["last_synced_at"].as_str().unwrap_or("Never"));
    println!("Status: {}", stats["status"].as_str().unwrap_or("Unknown"));
    println!("Tables Synced: {}", stats["tables_synced"].as_i64().unwrap_or(0));
    if let Some(error) = stats["last_error"].as_str() {
        println!("Last Error: {}", error);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    Ok(())
}

// ==================== Data Explorer ====================

pub async fn handle_list_namespace_tree(client: &PangolinClient, catalog: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/catalogs/{}/namespaces/tree", catalog)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to get namespace tree ({}): {}", status, error_text)));
    }
    
    let tree: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("Namespace Tree for '{}':", catalog);
    println!("{}", serde_json::to_string_pretty(&tree).unwrap());
    Ok(())
}

// ==================== Merge Operations ====================

pub async fn handle_list_merge_operations(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/merge-operations").await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to list merge operations: {}", res.status())));
    }
    
    let merges: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    if merges.is_empty() {
        println!("No merge operations found.");
        return Ok(());
    }
    
    let rows: Vec<Vec<String>> = merges.iter().map(|m| {
        vec![
            m["id"].as_str().unwrap_or("-").to_string(),
            m["source_branch"].as_str().unwrap_or("-").to_string(),
            m["target_branch"].as_str().unwrap_or("-").to_string(),
            m["status"].as_str().unwrap_or("-").to_string(),
            m["created_at"].as_str().unwrap_or("-").to_string(),
        ]
    }).collect();
    
    print_table(
        vec!["ID", "Source Branch", "Target Branch", "Status", "Created At"],
        rows
    );
    
    Ok(())
}

pub async fn handle_get_merge_operation(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/merge-operations/{}", id)).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to get merge operation: {}", res.status())));
    }
    
    let merge: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Merge Operation Details");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ID: {}", merge["id"].as_str().unwrap_or("-"));
    println!("Source Branch: {}", merge["source_branch"].as_str().unwrap_or("-"));
    println!("Target Branch: {}", merge["target_branch"].as_str().unwrap_or("-"));
    println!("Status: {}", merge["status"].as_str().unwrap_or("-"));
    println!("Created At: {}", merge["created_at"].as_str().unwrap_or("-"));
    if let Some(completed) = merge["completed_at"].as_str() {
        println!("Completed At: {}", completed);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}

pub async fn handle_list_conflicts(client: &PangolinClient, merge_id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/merge-operations/{}/conflicts", merge_id)).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to list conflicts: {}", res.status())));
    }
    
    let conflicts: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    if conflicts.is_empty() {
        println!("No conflicts found.");
        return Ok(());
    }
    
    let rows: Vec<Vec<String>> = conflicts.iter().map(|c| {
        vec![
            c["id"].as_str().unwrap_or("-").to_string(),
            c["asset_id"].as_str().unwrap_or("-").to_string(),
            c["conflict_type"].as_str().unwrap_or("-").to_string(),
            c["resolved"].as_bool().map(|r| if r { "Yes" } else { "No" }).unwrap_or("-").to_string(),
        ]
    }).collect();
    
    print_table(
        vec!["ID", "Asset ID", "Conflict Type", "Resolved"],
        rows
    );
    
    Ok(())
}

pub async fn handle_resolve_conflict(
    client: &PangolinClient,
    merge_id: String,
    conflict_id: String,
    resolution: String,
) -> Result<(), CliError> {
    let payload = serde_json::json!({
        "resolution": resolution,
    });
    
    let res = client.post(
        &format!("/api/v1/merge-operations/{}/conflicts/{}/resolve", merge_id, conflict_id),
        &payload
    ).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to resolve conflict ({}): {}", status, error_text)));
    }
    
    println!("✅ Conflict resolved successfully!");
    
    Ok(())
}

pub async fn handle_complete_merge(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/merge-operations/{}/complete", id), &serde_json::json!({})).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to complete merge ({}): {}", status, error_text)));
    }
    
    println!("✅ Merge completed successfully!");
    
    Ok(())
}

pub async fn handle_abort_merge(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/merge-operations/{}/abort", id), &serde_json::json!({})).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to abort merge ({}): {}", status, error_text)));
    }
    
    println!("✅ Merge aborted successfully!");
    
    Ok(())
}

// ==================== Business Metadata ====================

pub async fn handle_delete_metadata(client: &PangolinClient, asset_id: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/business-metadata/{}", asset_id)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to delete metadata ({}): {}", status, error_text)));
    }
    
    println!("✅ Business metadata deleted successfully!");
    
    Ok(())
}

pub async fn handle_request_access(
    client: &PangolinClient,
    asset_id: String,
    reason: String,
) -> Result<(), CliError> {
    let payload = serde_json::json!({
        "asset_id": asset_id,
        "reason": reason,
    });
    
    let res = client.post("/api/v1/access-requests", &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to request access ({}): {}", status, error_text)));
    }
    
    let result: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("✅ Access request submitted successfully!");
    println!("Request ID: {}", result["id"].as_str().unwrap_or("-"));
    
    Ok(())
}

pub async fn handle_list_access_requests(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/access-requests").await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to list access requests: {}", res.status())));
    }
    
    let requests: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    if requests.is_empty() {
        println!("No access requests found.");
        return Ok(());
    }
    
    let rows: Vec<Vec<String>> = requests.iter().map(|r| {
        vec![
            r["id"].as_str().unwrap_or("-").to_string(),
            r["asset_id"].as_str().unwrap_or("-").to_string(),
            r["user_id"].as_str().unwrap_or("-").to_string(),
            r["status"].as_str().unwrap_or("-").to_string(),
            r["reason"].as_str().unwrap_or("-").to_string(),
            r["created_at"].as_str().unwrap_or("-").to_string(),
        ]
    }).collect();
    
    print_table(
        vec!["ID", "Asset ID", "User ID", "Status", "Reason", "Created At"],
        rows
    );
    
    Ok(())
}

pub async fn handle_update_access_request(
    client: &PangolinClient,
    id: String,
    status: String,
) -> Result<(), CliError> {
    let payload = serde_json::json!({
        "status": status,
    });
    
    let res = client.put(&format!("/api/v1/access-requests/{}", id), &payload).await?;
    
    if !res.status().is_success() {
        let status_code = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update access request ({}): {}", status_code, error_text)));
    }
    
    println!("✅ Access request updated to: {}", status);
    
    Ok(())
}

pub async fn handle_get_asset_details(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/assets/{}", id)).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to get asset details: {}", res.status())));
    }
    
    let asset: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Asset Details");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ID: {}", asset["id"].as_str().unwrap_or("-"));
    println!("Name: {}", asset["name"].as_str().unwrap_or("-"));
    println!("Type: {}", asset["asset_type"].as_str().unwrap_or("-"));
    println!("Catalog: {}", asset["catalog_id"].as_str().unwrap_or("-"));
    if let Some(desc) = asset["description"].as_str() {
        println!("Description: {}", desc);
    }
    println!("Discoverable: {}", asset["discoverable"].as_bool().unwrap_or(false));
    println!("Created At: {}", asset["created_at"].as_str().unwrap_or("-"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}

// ==================== Audit Logging ====================

pub async fn handle_list_audit_events(
    client: &PangolinClient,
    user_id: Option<String>,
    action: Option<String>,
    resource_type: Option<String>,
    result: Option<String>,
    limit: usize,
) -> Result<(), CliError> {
    let mut query_params = vec![format!("limit={}", limit)];
    if let Some(uid) = user_id { query_params.push(format!("user_id={}", uid)); }
    if let Some(act) = action { query_params.push(format!("action={}", act)); }
    if let Some(rt) = resource_type { query_params.push(format!("resource_type={}", rt)); }
    if let Some(res) = result { query_params.push(format!("result={}", res)); }
    
    let query_string = if query_params.is_empty() { String::new() } else { format!("?{}", query_params.join("&")) };
    let res = client.get(&format!("/api/v1/audit{}", query_string)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Failed: {}", res.status()))); }
    
    let events: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    if events.is_empty() { println!("No audit events found."); return Ok(()); }
    
    let rows: Vec<Vec<String>> = events.iter().map(|e| vec![
        e["id"].as_str().unwrap_or("-").chars().take(8).collect::<String>(),
        e["username"].as_str().unwrap_or("-").to_string(),
        e["action"].as_str().unwrap_or("-").to_string(),
        e["resource_type"].as_str().unwrap_or("-").to_string(),
        e["resource_name"].as_str().unwrap_or("-").to_string(),
        e["result"].as_str().unwrap_or("-").to_string(),
        e["timestamp"].as_str().unwrap_or("-").chars().take(19).collect::<String>(),
    ]).collect();
    
    print_table(vec!["ID", "User", "Action", "Resource Type", "Resource Name", "Result", "Timestamp"], rows);
    println!("\nShowing {} events", events.len());
    Ok(())
}

pub async fn handle_count_audit_events(
    client: &PangolinClient,
    user_id: Option<String>,
    action: Option<String>,
    resource_type: Option<String>,
    result: Option<String>,
) -> Result<(), CliError> {
    let mut query_params = vec![];
    if let Some(uid) = user_id { query_params.push(format!("user_id={}", uid)); }
    if let Some(act) = action { query_params.push(format!("action={}", act)); }
    if let Some(rt) = resource_type { query_params.push(format!("resource_type={}", rt)); }
    if let Some(res) = result { query_params.push(format!("result={}", res)); }
    
    let query_string = if query_params.is_empty() { String::new() } else { format!("?{}", query_params.join("&")) };
    let res = client.get(&format!("/api/v1/audit/count{}", query_string)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Failed: {}", res.status()))); }
    
    let count_response: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    println!("Total audit events: {}", count_response["count"].as_u64().unwrap_or(0));
    Ok(())
}

pub async fn handle_get_audit_event(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/audit/{}", id)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Failed: {}", res.status()))); }
    
    let event: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Audit Event Details");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ID: {}", event["id"].as_str().unwrap_or("-"));
    println!("User: {}", event["username"].as_str().unwrap_or("-"));
    println!("Action: {}", event["action"].as_str().unwrap_or("-"));
    println!("Resource: {} ({})", event["resource_name"].as_str().unwrap_or("-"), event["resource_type"].as_str().unwrap_or("-"));
    println!("Result: {}", event["result"].as_str().unwrap_or("-"));
    println!("Timestamp: {}", event["timestamp"].as_str().unwrap_or("-"));
    if let Some(error) = event["error_message"].as_str() { println!("Error: {}", error); }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    Ok(())
}
