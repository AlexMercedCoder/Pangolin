mod commands;
mod handlers;

use clap::Parser;
use commands::{AdminCli, AdminCommand};
use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::config::ConfigManager;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use shell_words;
use std::env;

#[derive(Parser, Debug)]
#[command(name = "pangolin-admin")]
struct Args {
    #[arg(long, env = "PANGOLIN_URL")]
    url: Option<String>,

    #[arg(long, short = 'p')]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<AdminCommand>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Parse startup args (outer layer)
    // We treat the "REPL" mode as the default if no subcommand is provided
    let args = Args::parse();

    // Load saved config
    let config_manager = ConfigManager::new(args.profile.as_deref())?;
    let mut config = config_manager.load()?;
    
    // Override URL if provided via Args or Env
    if let Some(url) = args.url {
        config.base_url = url;
    }
    
    let mut client = PangolinClient::new(config.clone());

    // If a subcommand was passed directly (non-interactive mode), execute it and exit
    if let Some(cmd) = args.command {
        match cmd {
            AdminCommand::Login { username, password } => {
                if let Err(e) = handlers::handle_login(&mut client, username, password).await { eprintln!("Error: {}", e); }
                else {
                    if let Err(e) = config_manager.save(&client.config) {
                        eprintln!("Error saving config: {}", e);
                    }
                }
            },
            AdminCommand::ListTenants => handlers::handle_list_tenants(&client).await?,
            AdminCommand::CreateTenant { name } => handlers::handle_create_tenant(&client, name).await?,
            AdminCommand::DeleteTenant { id } => handlers::handle_delete_tenant(&client, id).await?,
            AdminCommand::ListUsers => handlers::handle_list_users(&client).await?,
            AdminCommand::CreateUser { username, email, role, password, tenant_id } => handlers::handle_create_user(&client, username, email, role, password, tenant_id).await?,
            AdminCommand::DeleteUser { username } => handlers::handle_delete_user(&client, username).await?,
            AdminCommand::ListWarehouses => handlers::handle_list_warehouses(&client).await?,
            AdminCommand::CreateWarehouse { name, type_ } => handlers::handle_create_warehouse(&client, name, type_).await?,
            AdminCommand::DeleteWarehouse { name } => handlers::handle_delete_warehouse(&client, name).await?,
            AdminCommand::ListCatalogs => handlers::handle_list_catalogs(&client).await?,
            AdminCommand::CreateCatalog { name, warehouse } => handlers::handle_create_catalog(&client, name, warehouse).await?,
            AdminCommand::DeleteCatalog { name } => handlers::handle_delete_catalog(&client, name).await?,
            AdminCommand::ListPermissions { role, user } => handlers::handle_list_permissions(&client, role, user).await?,
            AdminCommand::GrantPermission { role, action, resource } => handlers::handle_grant_permission(&client, role, action, resource).await?,
            AdminCommand::RevokePermission { role, action, resource } => handlers::handle_revoke_permission(&client, role, action, resource).await?,
            AdminCommand::GetMetadata { entity_type, entity_id } => handlers::handle_get_metadata(&client, entity_type, entity_id).await?,
            AdminCommand::SetMetadata { entity_type, entity_id, key, value } => handlers::handle_set_metadata(&client, entity_type, entity_id, key, value).await?,
            _ => println!("Command not available in non-interactive mode."),
        }
        return Ok(());
    }

    println!("Welcome to Pangolin Admin CLI");
    println!("Connected to: {}", client.config.base_url);
    println!("Type 'help' for a list of commands.");
    
    if let Some(user) = &config.username {
        println!("Logged in as: {}", user);
    } else {
        println!("Not logged in. Use 'login' command.");
    }

    // Initialize rustyline
    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()?;
    
    loop {
        let prompt = if let Some(user) = &client.config.username {
            format!("(admin:{})> ", user)
        } else {
            "(admin:unauth)> ".to_string()
        };

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                rl.add_history_entry(line)?;

                // Parse command
                let args_result = shell_words::split(line);
                match args_result {
                    Ok(args) => {
                        // Prepend "pangolin-admin" to make clap happy since it expects program name 0th argument
                        let mut full_args = vec!["pangolin-admin".to_string()];
                        full_args.extend(args);

                        match AdminCli::try_parse_from(full_args) {
                            Ok(cli) => {
                                match cli.command {
                                    AdminCommand::Exit => {
                                        println!("Goodbye!");
                                        break;
                                    },
                                    AdminCommand::Clear => {
                                        print!("\x1B[2J\x1B[1;1H");
                                    },
                                    AdminCommand::Login { username, password } => {
                                        if let Err(e) = handlers::handle_login(&mut client, username, password).await {
                                            eprintln!("Error: {}", e);
                                        } else {
                                            // Save new config on success
                                            if let Err(e) = config_manager.save(&client.config) {
                                                eprintln!("Warning: Failed to save config: {}", e);
                                            }
                                        }
                                    },
                                    AdminCommand::ListTenants => {
                                        if let Err(e) = handlers::handle_list_tenants(&client).await {
                                            eprintln!("Error: {}", e);
                                        }
                                    },
                                    AdminCommand::CreateTenant { name } => {
                                         if let Err(e) = handlers::handle_create_tenant(&client, name).await {
                                            eprintln!("Error: {}", e);
                                        }
                                    },
                                    AdminCommand::DeleteTenant { id } => {
                                        if let Err(e) = handlers::handle_delete_tenant(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListUsers => {
                                        if let Err(e) = handlers::handle_list_users(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::CreateUser { username, email, role, password, tenant_id } => {
                                        if let Err(e) = handlers::handle_create_user(&client, username, email, role, password, tenant_id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::DeleteUser { username } => {
                                        if let Err(e) = handlers::handle_delete_user(&client, username).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListWarehouses => {
                                        if let Err(e) = handlers::handle_list_warehouses(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::CreateWarehouse { name, type_ } => {
                                        if let Err(e) = handlers::handle_create_warehouse(&client, name, type_).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::DeleteWarehouse { name } => {
                                        if let Err(e) = handlers::handle_delete_warehouse(&client, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListCatalogs => {
                                        if let Err(e) = handlers::handle_list_catalogs(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::CreateCatalog { name, warehouse } => {
                                        if let Err(e) = handlers::handle_create_catalog(&client, name, warehouse).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::DeleteCatalog { name } => {
                                        if let Err(e) = handlers::handle_delete_catalog(&client, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListPermissions { role, user } => {
                                        if let Err(e) = handlers::handle_list_permissions(&client, role, user).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GrantPermission { role, action, resource } => {
                                        if let Err(e) = handlers::handle_grant_permission(&client, role, action, resource).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::RevokePermission { role, action, resource } => {
                                        if let Err(e) = handlers::handle_revoke_permission(&client, role, action, resource).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetMetadata { entity_type, entity_id } => {
                                        if let Err(e) = handlers::handle_get_metadata(&client, entity_type, entity_id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::SetMetadata { entity_type, entity_id, key, value } => {
                                        if let Err(e) = handlers::handle_set_metadata(&client, entity_type, entity_id, key, value).await { eprintln!("Error: {}", e); }
                                    }
                                }
                            },
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    },
                    Err(e) => {
                        println!("Parse error: {}", e);
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    Ok(())
}

// Make `client` public or add a getter in `pangolin_cli_common::client` if `config` field is private.
// Wait, `config` field on `PangolinClient` defined earlier is private by default in Rust structs?
// Let me double check `pangolin_cli_common/src/client.rs`.
