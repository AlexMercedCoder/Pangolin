use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::utils::print_table;
use pangolin_cli_common::error::CliError;
use crate::commands::CodeLanguage;
use dialoguer::{Input, Password};
use serde_json::Value;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

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

pub async fn handle_list_catalogs(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/catalogs").await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to list catalogs: {}", res.status())));
    }
    
    let catalogs: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    let rows: Vec<Vec<String>> = catalogs.iter().map(|c| {
        vec![
            c["name"].as_str().unwrap_or("-").to_string(),
            c["catalog_type"].as_str().unwrap_or("-").to_string()
        ]
    }).collect();
    
    print_table(vec!["Name", "Type"], rows);
    Ok(())
}

pub async fn handle_search(_client: &PangolinClient, query: String) -> Result<(), CliError> {
    // Placeholder - Search API not fully implemented yet in handlers.rs of common?
    // We will just mock it for now or hit the real endpoint if available
    println!("Searching for '{}'...", query);
    println!("(Search functionality pending backend index implementation)");
    Ok(())
}

pub async fn handle_generate_code(client: &PangolinClient, language: CodeLanguage, table: String) -> Result<(), CliError> {
    let url = &client.config.base_url;
    let token = client.config.auth_token.as_deref().unwrap_or("<YOUR_TOKEN>");
    
    let parts: Vec<&str> = table.split('.').collect();
    let (catalog, namespace, table_name) = if parts.len() == 3 {
        (parts[0], parts[1], parts[2])
    } else {
        ("my_catalog", "my_ns", table.as_str())
    };

    let (code, syntax) = match language {
        CodeLanguage::Pyiceberg => (format!(r#"
from pyiceberg.catalog import load_catalog

catalog = load_catalog(
    "{}",
    **{{
        "uri": "{}",
        "credential": "{}"
    }}
)

table = catalog.load_table("{}.{}.{}")
print(table.schema())
df = table.scan().to_arrow()
"#, catalog, url, token, catalog, namespace, table_name), "py"),
        CodeLanguage::Pyspark => (format!(r#"
spark = SparkSession.builder \
    .config("spark.sql.catalog.{}", "org.apache.iceberg.spark.SparkCatalog") \
    .config("spark.sql.catalog.{}.type", "rest") \
    .config("spark.sql.catalog.{}.uri", "{}") \
    .config("spark.sql.catalog.{}.credential", "{}") \
    .getOrCreate()

df = spark.table("{}.{}.{}")
df.show()
"#, catalog, catalog, catalog, url, catalog, token, catalog, namespace, table_name), "py"),
        CodeLanguage::Dremio => (format!(r#"
-- Add Source in Dremio
SOURCE TYPE: Iceberg
REST URL: {}
CREDENTIAL: {}
"#, url, token), "sql"),
         CodeLanguage::Sql => (format!(r#"
SELECT * FROM {}.{}.{} LIMIT 10;
"#, catalog, namespace, table_name), "sql"),
    };

    println!("\n--- Generated {:?} Code ---\n", language);
    
    // Syntax Highlighting
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    
    let syntax = ps.find_syntax_by_extension(syntax).unwrap_or(ps.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    
    for line in LinesWithEndings::from(&code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
    println!("\x1b[0m"); // Reset colors

    Ok(())
}

// --- Branches ---
pub async fn handle_list_branches(client: &PangolinClient, catalog: String) -> Result<(), CliError> {
     // Expected API: GET /api/v1/catalogs/{name}/branches OR /api/v1/branches?catalog={name}
     // Based on previous exploration: /api/v1/branches?catalog={name} via list_branches handler
    let res = client.get(&format!("/api/v1/branches?catalog={}", catalog)).await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["name"].as_str().unwrap_or("").to_string(),
        i["head"].as_str().unwrap_or("-").to_string()
    ]).collect();
    print_table(vec!["Name", "Head Commit"], rows);
    Ok(())
}

pub async fn handle_create_branch(client: &PangolinClient, catalog: String, name: String, from: Option<String>) -> Result<(), CliError> {
    let body = serde_json::json!({
        "catalog": catalog,
        "name": name,
        "from_branch": from.unwrap_or("main".to_string())
    });
    let res = client.post("/api/v1/branches", &body).await?;
    if !res.status().is_success() { 
        let status = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", status, t))); 
    }
    println!("✅ Branch '{}' created.", name);
    Ok(())
}

pub async fn handle_merge_branch(client: &PangolinClient, catalog: String, source: String, target: String) -> Result<(), CliError> {
    let body = serde_json::json!({
        "catalog": catalog,
        "source": source,
        "target": target
    });
    let res = client.post("/api/v1/branches/merge", &body).await?;
    if !res.status().is_success() { 
        let status = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", status, t))); 
    }
    println!("✅ Branch '{}' merged into '{}'.", source, target);
    Ok(())
}

// --- Tags ---
pub async fn handle_list_tags(client: &PangolinClient, catalog: String) -> Result<(), CliError> {
    let res = client.get(&format!("/api/v1/tags?catalog={}", catalog)).await?; // Assumption on endpoint
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
     let rows = items.iter().map(|i| vec![
        i["name"].as_str().unwrap_or("").to_string(),
        i["commit_id"].as_str().unwrap_or("-").to_string()
    ]).collect();
    print_table(vec!["Name", "Commit ID"], rows);
    Ok(())
}

pub async fn handle_create_tag(client: &PangolinClient, catalog: String, name: String, commit_id: String) -> Result<(), CliError> {
    let body = serde_json::json!({
        "catalog": catalog,
        "name": name,
        "commit_id": commit_id
    });
    let res = client.post("/api/v1/tags", &body).await?;
    if !res.status().is_success() { 
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("{} - {}", status, t))); 
    }
    println!("✅ Tag '{}' created.", name);
    Ok(())
}

// --- Access Requests ---
pub async fn handle_list_requests(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/access-requests").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["id"].as_str().unwrap_or("").to_string(),
        i["resource"].as_str().unwrap_or("").to_string(),
        i["status"].as_str().unwrap_or("").to_string()
    ]).collect();
    print_table(vec!["ID", "Resource", "Status"], rows);
    Ok(())
}

pub async fn handle_request_access(client: &PangolinClient, resource: String, role: String, reason: String) -> Result<(), CliError> {
    Ok(())
}
