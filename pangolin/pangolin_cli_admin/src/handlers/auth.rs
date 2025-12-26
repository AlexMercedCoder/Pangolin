use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use dialoguer::{Input, Password};
use serde_json::Value;

pub async fn handle_login(client: &mut PangolinClient, username_opt: Option<String>, password_opt: Option<String>, tenant_id_opt: Option<String>) -> Result<(), CliError> {
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

    client.login_with_tenant(&username, &password, tenant_id_opt.as_deref()).await?;
    
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
