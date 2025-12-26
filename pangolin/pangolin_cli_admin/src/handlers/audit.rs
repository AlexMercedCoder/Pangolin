use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use serde_json::Value;

pub async fn handle_list_audit_events(
    client: &PangolinClient,
    user_id: Option<String>,
    action: Option<String>,
    resource_type: Option<String>,
    result: Option<String>,
    tenant_id: Option<String>,
    limit: usize,
) -> Result<(), CliError> {
    let mut query_params = vec![format!("limit={}", limit)];
    if let Some(uid) = user_id { query_params.push(format!("user_id={}", uid)); }
    if let Some(act) = action { query_params.push(format!("action={}", act)); }
    if let Some(rt) = resource_type { query_params.push(format!("resource_type={}", rt)); }
    if let Some(res) = result { query_params.push(format!("result={}", res)); }
    if let Some(tid) = tenant_id { query_params.push(format!("tenant_id={}", tid)); }
    
    let query_string = if query_params.is_empty() { String::new() } else { format!("?{}", query_params.join("&")) };
    let res = client.get(&format!("/api/v1/audit{}", query_string)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Failed: {}", res.status()))); }
    
    let events: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    if events.is_empty() { println!("No audit events found."); return Ok(()); }
    
    // UPDATED TABLE COLUMNS FOR PARITY
    let rows: Vec<Vec<String>> = events.iter().map(|e| vec![
        e["id"].as_str().unwrap_or("-").chars().take(8).collect::<String>(),
        e["username"].as_str().unwrap_or("-").to_string(),
        e["action"].as_str().unwrap_or("-").to_string(),
        e["resource_type"].as_str().unwrap_or("-").to_string(),
        e["resource_name"].as_str().unwrap_or("-").to_string(),
        e["result"].as_str().unwrap_or("-").to_string(),
        e["ip_address"].as_str().unwrap_or("-").to_string(), // Added IP
        e["timestamp"].as_str().unwrap_or("-").chars().take(19).collect::<String>(),
    ]).collect();
    
    print_table(vec!["ID", "User", "Action", "Res Type", "Resource", "Result", "IP Address", "Timestamp"], rows);
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
    
    let count: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    println!("Count: {}", count["count"]);
    Ok(())
}

pub async fn handle_get_audit_event(client: &PangolinClient, id: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/audit/{}", id)).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to get audit event: {}", res.status())));
    }
    
    let event: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Audit Event Details");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ID:           {}", event["id"].as_str().unwrap_or("-"));
    println!("User:         {} (ID: {})", event["username"].as_str().unwrap_or("-"), event["user_id"].as_str().unwrap_or("None"));
    println!("Action:       {}", event["action"].as_str().unwrap_or("-"));
    println!("Result:       {}", event["result"].as_str().unwrap_or("-"));
    println!("Resource:     {} ({})", event["resource_name"].as_str().unwrap_or("-"), event["resource_type"].as_str().unwrap_or("-"));
    println!("Timestamp:    {}", event["timestamp"].as_str().unwrap_or("-"));
    
    // ADDED PARITY FIELDS
    if let Some(ip) = event.get("ip_address").and_then(|v| v.as_str()) {
        println!("IP Address:   {}", ip);
    }
    if let Some(agent) = event.get("user_agent").and_then(|v| v.as_str()) {
        println!("User Agent:   {}", agent);
    }
    if let Some(error) = event.get("error_message").and_then(|v| v.as_str()) {
        println!("Error:        {}", error);
    }
    
    if let Some(meta) = event.get("metadata") {
        if !meta.is_null() {
             println!("Metadata:     {}", serde_json::to_string_pretty(meta).unwrap());
        }
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}
