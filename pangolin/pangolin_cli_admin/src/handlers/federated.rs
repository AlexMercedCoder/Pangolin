use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use serde_json::Value;

pub async fn handle_sync_federated_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/catalogs/{}/sync", name), &serde_json::json!({})).await?;
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Sync failed: {}", res.status())));
    }
    println!("Catalog '{}' synced successfully.", name);
    Ok(())
}

pub async fn handle_get_federated_catalog_stats(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/catalogs/{}/stats", name)).await?;
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to get stats: {}", res.status())));
    }
    let stats: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    println!("{}", serde_json::to_string_pretty(&stats).unwrap());
    Ok(())
}

pub async fn handle_create_federated_catalog(
    client: &PangolinClient,
    name: String,
    storage_location: String,
    properties: Vec<String>,
) -> Result<(), CliError> {
    if client.config.tenant_id.is_none() {
        return Err(CliError::ApiError("Root user cannot create catalogs. Please login as Tenant Admin.".to_string()));
    }

    let mut props = serde_json::Map::new();
    for p in properties {
        if let Some((k, v)) = p.split_once('=') {
            props.insert(k.to_string(), serde_json::Value::String(v.to_string()));
        }
    }

    let body = serde_json::json!({
        "name": name,
        "type": "federated",
        "storage_location": storage_location,
        "properties": props
    });
    
    let res = client.post("/api/v1/catalogs", &body).await?;
    if !res.status().is_success() { 
        let s = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", s, t))); 
    }
    println!("✅ Federated Catalog '{}' created.", name);
    Ok(())
}

pub async fn handle_list_federated_catalogs(client: &PangolinClient, limit: Option<usize>, offset: Option<usize>) -> Result<(), CliError> {
    let q = pangolin_cli_common::utils::pagination_query(limit, offset);
    // Base query expects type=federated
    let query_str = if q.is_empty() { "type=federated".to_string() } else { format!("type=federated&{}", q) };
    let res = client.get(&format!("/api/v1/catalogs?{}", query_str)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<serde_json::Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    // Filter client side just in case API doesn't support query param perfectly yet or to be safe
    let fed_cats: Vec<Vec<String>> = items.iter()
        .filter(|i| i["type"].as_str() == Some("federated"))
        .map(|i| vec![
            i["name"].as_str().unwrap_or("").to_string(),
            i["storage_location"].as_str().unwrap_or("-").to_string()
        ]).collect();
        
    print_table(vec!["Name", "Location"], fed_cats);
    Ok(())
}

pub async fn handle_delete_federated_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/catalogs/{}", name)).await?;
    if !res.status().is_success() {
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to delete catalog: {} - {}", status, t)));
    }
    println!("✅ Federated Catalog '{}' deleted.", name);
    Ok(())
}

pub async fn handle_test_federated_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/catalogs/{}/test", name), &serde_json::json!({})).await?;
    if !res.status().is_success() {
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Connectivity Test Failed: {} - {}", status, t)));
    }
    println!("✅ Connection to '{}' successful.", name);
    Ok(())
}
