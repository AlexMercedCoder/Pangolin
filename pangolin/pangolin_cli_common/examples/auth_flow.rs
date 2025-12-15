use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::config::CliConfig;
use std::env;

#[tokio::main]
async fn main() {
    let username = env::args().nth(1).expect("Please provide username");
    let password = env::args().nth(2).expect("Please provide password");
    
    let config = CliConfig::default();
    let mut client = PangolinClient::new(config);
    
    println!("Attempting to login as {}...", username);
    match client.login(&username, &password).await {
        Ok(_) => println!("Login successful! Token saved."),
        Err(e) => println!("Login failed: {}", e),
    }
}
