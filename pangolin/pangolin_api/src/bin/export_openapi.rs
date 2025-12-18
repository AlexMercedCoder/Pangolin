use pangolin_api::openapi::ApiDoc;
use utoipa::OpenApi;
use std::env;

fn main() {
    let openapi = ApiDoc::openapi();
    
    let args: Vec<String> = env::args().collect();
    let format = args.get(1).map(|s| s.as_str()).unwrap_or("json");
    
    match format {
        "yaml" => {
            let yaml = serde_yaml::to_string(&openapi).expect("Failed to serialize OpenAPI to YAML");
            println!("{}", yaml);
        }
        "json" | _ => {
            let json = serde_json::to_string_pretty(&openapi).expect("Failed to serialize OpenAPI to JSON");
            println!("{}", json);
        }
    }
}
