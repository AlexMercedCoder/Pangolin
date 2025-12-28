mod commands;
mod handlers;

use clap::Parser;
use commands::{UserCli, UserCommand};
use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::config::ConfigManager;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use shell_words;
use std::env;

#[derive(Parser, Debug)]
#[command(name = "pangolin-user")]
struct Args {
    #[arg(long, env = "PANGOLIN_URL")]
    url: Option<String>,

    #[arg(long, short = 'p')]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<UserCommand>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let args = Args::parse();

    let config_manager = ConfigManager::new(args.profile.as_deref())?;
    let mut config = config_manager.load()?;
    
    if let Some(url) = args.url {
        config.base_url = url;
    }
    
    let mut client = PangolinClient::new(config.clone());
    
    // Non-interactive mode
    if let Some(cmd) = args.command {
        match cmd {
            UserCommand::Login { username, password, tenant_id } => {
                handlers::handle_login(&mut client, username, password, tenant_id).await?;
                config_manager.save(&client.config).map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
            },
            UserCommand::ListCatalogs { limit, offset } => handlers::handle_list_catalogs(&client, limit, offset).await?,
            UserCommand::Search { query } => handlers::handle_search(&client, query).await?,
            UserCommand::GenerateCode { language, table } => handlers::handle_generate_code(&client, language, table).await?,
            UserCommand::ListBranches { catalog, limit, offset } => handlers::handle_list_branches(&client, catalog, limit, offset).await?,
            UserCommand::CreateBranch { catalog, name, from, branch_type, assets } => handlers::handle_create_branch(&client, catalog, name, from, branch_type, assets).await?,
            UserCommand::MergeBranch { catalog, source, target } => handlers::handle_merge_branch(&client, catalog, source, target).await?,
            UserCommand::ListTags { catalog, limit, offset } => handlers::handle_list_tags(&client, catalog, limit, offset).await?,
            UserCommand::CreateTag { catalog, name, commit_id } => handlers::handle_create_tag(&client, catalog, name, commit_id).await?,
            UserCommand::ListRequests { limit, offset } => handlers::handle_list_requests(&client, limit, offset).await?,
            UserCommand::RequestAccess { resource, role, reason } => handlers::handle_request_access(&client, resource, role, reason).await?,
            UserCommand::GetToken { description, expires_in } => handlers::handle_get_token(&client, description, expires_in).await?,
            _ => println!("Command not available in non-interactive mode."),
        }
        return Ok(());
    }

    println!("Welcome to Pangolin User CLI");
    println!("Connected to: {}", client.config.base_url);
    println!("Type 'help' for a list of commands.");
    
    if let Some(user) = &config.username {
        println!("Logged in as: {}", user);
    } else {
        println!("Not logged in. Use 'login' command.");
    }

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()?;
    
    loop {
        let prompt = if let Some(user) = &client.config.username {
            format!("(user:{})> ", user)
        } else {
            "(user:unauth)> ".to_string()
        };

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                rl.add_history_entry(line)?;

                let args_result = shell_words::split(line);
                match args_result {
                    Ok(args) => {
                        let mut full_args = vec!["pangolin-user".to_string()];
                        full_args.extend(args);

                        match UserCli::try_parse_from(full_args) {
                            Ok(cli) => {
                                match cli.command {
                                    UserCommand::Exit => {
                                        println!("Goodbye!");
                                        break;
                                    },
                                    UserCommand::Clear => {
                                        print!("\x1B[2J\x1B[1;1H");
                                    },
                                    UserCommand::Login { username, password, tenant_id } => {
                                        if let Err(e) = handlers::handle_login(&mut client, username, password, tenant_id).await {
                                            eprintln!("Error: {}", e);
                                        } else {
                                            if let Err(e) = config_manager.save(&client.config) {
                                                eprintln!("Warning: Failed to save config: {}", e);
                                            }
                                        }
                                    },
                                    UserCommand::ListCatalogs { limit, offset } => {
                                        if let Err(e) = handlers::handle_list_catalogs(&client, limit, offset).await {
                                            eprintln!("Error: {}", e);
                                        }
                                    },
                                    UserCommand::Search { query } => {
                                        if let Err(e) = handlers::handle_search(&client, query).await {
                                            eprintln!("Error: {}", e);
                                        }
                                    },
                                    UserCommand::GenerateCode { language, table } => {
                                        if let Err(e) = handlers::handle_generate_code(&client, language, table).await {
                                            eprintln!("Error: {}", e);
                                        }
                                    },
                                    UserCommand::ListBranches { catalog, limit, offset } => {
                                        if let Err(e) = handlers::handle_list_branches(&client, catalog, limit, offset).await { eprintln!("Error: {}", e); }
                                    },
                                    UserCommand::CreateBranch { catalog, name, from, branch_type, assets } => {
                                        if let Err(e) = handlers::handle_create_branch(&client, catalog, name, from, branch_type, assets).await { eprintln!("Error: {}", e); }
                                    },
                                    UserCommand::MergeBranch { catalog, source, target } => {
                                        if let Err(e) = handlers::handle_merge_branch(&client, catalog, source, target).await { eprintln!("Error: {}", e); }
                                    },
                                    UserCommand::ListTags { catalog, limit, offset } => {
                                         if let Err(e) = handlers::handle_list_tags(&client, catalog, limit, offset).await { eprintln!("Error: {}", e); }
                                    },
                                    UserCommand::CreateTag { catalog, name, commit_id } => {
                                        if let Err(e) = handlers::handle_create_tag(&client, catalog, name, commit_id).await { eprintln!("Error: {}", e); }
                                    },
                                    UserCommand::ListRequests { limit, offset } => {
                                        if let Err(e) = handlers::handle_list_requests(&client, limit, offset).await { eprintln!("Error: {}", e); }
                                    },
                                    UserCommand::RequestAccess { resource, role, reason } => {
                                        if let Err(e) = handlers::handle_request_access(&client, resource, role, reason).await { eprintln!("Error: {}", e); }
                                    },
                                    UserCommand::GetToken { description, expires_in } => {
                                        if let Err(e) = handlers::handle_get_token(&client, description, expires_in).await { eprintln!("Error: {}", e); }
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
