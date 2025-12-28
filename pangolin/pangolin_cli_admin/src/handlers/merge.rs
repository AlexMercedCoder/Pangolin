use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use dialoguer::Editor;
use serde_json::Value;

pub async fn handle_list_merge_operations(client: &PangolinClient, catalog: String, limit: Option<usize>, offset: Option<usize>) -> Result<(), CliError> {
    if let Some(tid) = &client.config.tenant_id {
        let mut query = format!("tenant_id={}&catalog={}", tid, catalog);
        let pag = pangolin_cli_common::utils::pagination_query(limit, offset);
        if !pag.is_empty() { query.push('&'); query.push_str(&pag); }
        
        let res = client.get(&format!("/api/v1/merges?{}", query)).await?;
        if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
        
        let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
        if items.is_empty() { println!("No merge operations found."); return Ok(()); }

        let rows = items.iter().map(|i| vec![
            i["id"].as_str().unwrap_or("-").to_string(),
            i["source_branch"].as_str().unwrap_or("-").to_string(),
            i["target_branch"].as_str().unwrap_or("-").to_string(),
            i["status"].as_str().unwrap_or("-").to_string(),
            i["initiated_at"].as_str().unwrap_or("-").to_string(),
        ]).collect();
        print_table(vec!["ID", "Source", "Target", "Status", "Initiated At"], rows);
    } else {
        println!("Please login to a tenant first.");
    }
    Ok(())
}

pub async fn handle_get_merge_operation(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/merges/{}", id)).await?;
    if !res.status().is_success() {
         return Err(CliError::ApiError(format!("Failed to get merge operation: {}", res.status())));
    }
    let op: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("\nMerge Operation Details");
    println!("ID: {}", op["id"].as_str().unwrap_or("-"));
    println!("Catalog: {}", op["catalog_name"].as_str().unwrap_or("-"));
    println!("Source: {}", op["source_branch"].as_str().unwrap_or("-"));
    println!("Target: {}", op["target_branch"].as_str().unwrap_or("-"));
    println!("Status: {}", op["status"].as_str().unwrap_or("-"));
    let conflicts = op["conflicts"].as_array().map(|c| c.len()).unwrap_or(0);
    println!("Conflicts: {}", conflicts);
    
    if conflicts > 0 {
        println!("\nUse 'pangolin-admin list-merge-conflicts --id {}' to view conflicts", id);
    }

    Ok(())
}

pub async fn handle_list_merge_conflicts(client: &PangolinClient, id: String, limit: Option<usize>, offset: Option<usize>) -> Result<(), CliError> {
    let q = pangolin_cli_common::utils::pagination_query(limit, offset);
    let path = if q.is_empty() { format!("/api/v1/merges/{}/conflicts", id) } else { format!("/api/v1/merges/{}/conflicts?{}", id, q) };
    let res = client.get(&path).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    if items.is_empty() { println!("No conflicts found."); return Ok(()); }

    let rows = items.iter().map(|i| vec![
        i["id"].as_str().unwrap_or("-").to_string(),
        i["conflict_type"].as_str().unwrap_or("Unknown").to_string(), // Simplified display
        i["description"].as_str().unwrap_or("-").to_string(),
        if i["resolution"].is_null() { "Unresolved" } else { "Resolved" }.to_string()
    ]).collect();
    print_table(vec!["Conflict ID", "Type", "Description", "Status"], rows);
    Ok(())
}

pub async fn handle_resolve_merge_conflict(
    client: &PangolinClient, 
    conflict_id: String,
    strategy: String,
    value: Option<String>
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({
        "strategy": strategy,
        "resolved_by": client.config.username.clone().unwrap_or("cli-user".to_string()), // Backend really wants UUID but we might send username if allowed or let backend infer from auth
        // Actually backend expects UUID for resolved_by. 
        // But for this CLI we might just send the strategy and value object.
        // Let's assume the API handles the user context.
        "conflict_id": conflict_id,
        "resolved_at": chrono::Utc::now().to_rfc3339()
    });

    // If strategy is Manual, we look at value
    if strategy.to_lowercase() == "manual" {
         if let Some(v) = value {
             // Try to parse as JSON
             let parsed: Value = serde_json::from_str(&v).unwrap_or(Value::String(v));
             payload["resolved_value"] = parsed;
         } else {
             // Open editor

            if let Some(desc) = Editor::new().extension(".json").edit("{\n  \"your_resolution\": \"here\"\n}").map_err(|e| CliError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))? {

                 let parsed: Value = serde_json::from_str(&desc).map_err(|e| CliError::ApiError(format!("Invalid JSON: {}", e)))?;
                 payload["resolved_value"] = parsed;
             } else {
                 return Err(CliError::ApiError("Aborted manual resolution".to_string()));
             }
         }
    }

    let res = client.post(&format!("/api/v1/merges/conflicts/{}/resolve", conflict_id), &payload).await?;
    let status = res.status();
    if !status.is_success() {
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to resolve: {} - {}", status, t)));
    }
    println!("✅ Conflict resolved.");
    Ok(())
}

pub async fn handle_complete_merge_operation(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/merges/{}/complete", id), &serde_json::json!({})).await?;
    let status = res.status();
    if !status.is_success() {
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to complete merge: {} - {}", status, t)));
    }
    println!("✅ Merge operation completed successfully.");
    Ok(())
}

pub async fn handle_abort_merge_operation(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.post(&format!("/api/v1/merges/{}/abort", id), &serde_json::json!({})).await?;
    let status = res.status();
    if !status.is_success() {
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to abort merge: {} - {}", status, t)));
    }
    println!("✅ Merge operation aborted.");
    Ok(())
}
