use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;

use serde_json::Value;
use pangolin_cli_common::optimization_types::SearchResponse;

// Helper recursive function to print tree
fn print_namespace_tree(items: &[Value], prefix: &str) {
    for (i, item) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = item["name"].as_str().unwrap_or("?");
        let type_ = item["type"].as_str().unwrap_or("?"); // "namespace" or "asset"
        
        let icon = if type_ == "namespace" { "📂" } else { "📄" };
        println!("{}{}{}{}", prefix, connector, icon, name);
        
        if let Some(children) = item["children"].as_array() {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_namespace_tree(children, &new_prefix);
        }
    }
}

pub async fn handle_namespace_tree(client: &PangolinClient, catalog: String) -> Result<(), CliError> {
    // This assumes an endpoint that returns a nested tree structure
    let res = client.get(&format!("/api/v1/explorer/tree?catalog={}", catalog)).await?;
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to fetch tree: {}", res.status())));
    }
    let root: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("\n📦 Catalog: {}", catalog);
    print_namespace_tree(&root, "");
    println!();
    Ok(())
}

pub async fn handle_get_asset(client: &PangolinClient, catalog: String, namespace: String, table: String) -> Result<(), CliError> {
    // Assuming endpoint structure
    // Namespace usually dot separated in query or path
    let res = client.get(&format!("/api/v1/catalogs/{}/namespaces/{}/tables/{}", catalog, namespace, table)).await?;
    if !res.status().is_success() {
         return Err(CliError::ApiError(format!("Failed to get asset: {}", res.status())));
    }
    let asset: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("\nAsset Details");
    println!("Name: {}", asset["name"].as_str().unwrap_or("-"));
    println!("Type: {}", asset["kind"].as_str().unwrap_or("-"));
    println!("Location: {}", asset["location"].as_str().unwrap_or("-"));
    // Add more details if available
    Ok(())
}

pub async fn handle_search(
    client: &PangolinClient,
    query: String,
    catalog: Option<String>,
    limit: usize
) -> Result<(), CliError> {
    let mut url = format!("/api/v1/search/assets?q={}&limit={}", 
        urlencoding::encode(&query), limit);
    
    if let Some(cat) = catalog {
        url.push_str(&format!("&catalog={}", urlencoding::encode(&cat)));
    }
    
    let res = client.get(&url).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Search failed: {}", res.status())));
    }
    
    let results: SearchResponse = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("Found {} results:", results.total);
    for result in results.results {
        println!("  {}.{}.{}", 
            result.catalog,
            result.namespace.join("."),
            result.name
        );
    }
    
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
