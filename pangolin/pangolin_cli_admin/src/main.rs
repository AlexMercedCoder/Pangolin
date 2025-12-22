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

    #[arg(long, env = "PANGOLIN_TENANT")]
    tenant: Option<String>,

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
    let config_manager = ConfigManager::new(args.profile.as_deref()).unwrap();
    let mut config = config_manager.load().unwrap_or_default();
    
    // Override URL if provided via Args or Env
    if let Some(url) = args.url {
        config.base_url = url;
    }
    
    if let Some(tenant) = args.tenant {
        config.tenant_id = Some(tenant);
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
            AdminCommand::Use { name } => {
                 if let Err(e) = handlers::handle_use(&mut client, name).await {
                    eprintln!("Error: {}", e);
                } else {
                     if let Err(e) = config_manager.save(&client.config) {
                         eprintln!("Warning: Failed to save config: {}", e);
                     }
                }
            },
            AdminCommand::ListTenants => handlers::handle_list_tenants(&client).await?,
            AdminCommand::CreateTenant { name, admin_username, admin_password } => handlers::handle_create_tenant(&client, name, admin_username, admin_password).await?,
            AdminCommand::DeleteTenant { id } => handlers::handle_delete_tenant(&client, id).await?,
            AdminCommand::ListUsers => handlers::handle_list_users(&client).await?,
            AdminCommand::CreateUser { username, email, role, password, tenant_id } => handlers::handle_create_user(&client, username, email, role, password, tenant_id).await?,
            AdminCommand::DeleteUser { username } => handlers::handle_delete_user(&client, username).await?,
            AdminCommand::ListWarehouses => handlers::handle_list_warehouses(&client).await?,
            AdminCommand::CreateWarehouse { name, type_, bucket, access_key, secret_key, region, endpoint } => handlers::handle_create_warehouse(&client, name, type_, bucket, access_key, secret_key, region, endpoint).await?,
            AdminCommand::DeleteWarehouse { name } => handlers::handle_delete_warehouse(&client, name).await?,
            AdminCommand::ListCatalogs => handlers::handle_list_catalogs(&client).await?,
            AdminCommand::CreateCatalog { name, warehouse } => handlers::handle_create_catalog(&client, name, warehouse).await?,
            AdminCommand::DeleteCatalog { name } => handlers::handle_delete_catalog(&client, name).await?,
            AdminCommand::CreateFederatedCatalog { name, storage_location, properties } => handlers::handle_create_federated_catalog(&client, name, storage_location, properties).await?,
            AdminCommand::ListFederatedCatalogs => handlers::handle_list_federated_catalogs(&client).await?,
            AdminCommand::DeleteFederatedCatalog { name } => handlers::handle_delete_federated_catalog(&client, name).await?,
            AdminCommand::TestFederatedCatalog { name } => handlers::handle_test_federated_catalog(&client, name).await?,
            AdminCommand::ListPermissions { role, user } => handlers::handle_list_permissions(&client, role, user).await?,
            AdminCommand::GrantPermission { username, action, resource } => handlers::handle_grant_permission(&client, username, action, resource).await?,
            AdminCommand::RevokePermission { role, action, resource } => handlers::handle_revoke_permission(&client, role, action, resource).await?,
            AdminCommand::GetMetadata { entity_type, entity_id } => handlers::handle_get_metadata(&client, entity_type, entity_id).await?,
            AdminCommand::SetMetadata { entity_type, entity_id, key, value } => handlers::handle_set_metadata(&client, entity_type, entity_id, key, value).await?,
            AdminCommand::CreateServiceUser { name, description, role, expires_in_days } => handlers::handle_create_service_user(&client, name, description, role, expires_in_days).await?,
            AdminCommand::ListServiceUsers => handlers::handle_list_service_users(&client).await?,
            AdminCommand::GetServiceUser { id } => handlers::handle_get_service_user(&client, id).await?,
            AdminCommand::UpdateServiceUser { id, name, description, active } => handlers::handle_update_service_user(&client, id, name, description, active).await?,
            AdminCommand::DeleteServiceUser { id } => handlers::handle_delete_service_user(&client, id).await?,
            AdminCommand::RotateServiceUserKey { id } => handlers::handle_rotate_service_user_key(&client, id).await?,
            AdminCommand::UpdateTenant { id, name } => handlers::handle_update_tenant(&client, id, name).await?,
            AdminCommand::UpdateUser { id, username, email, active } => handlers::handle_update_user(&client, id, username, email, active).await?,
            AdminCommand::UpdateWarehouse { id, name } => handlers::handle_update_warehouse(&client, id, name).await?,
            AdminCommand::UpdateCatalog { id, name } => handlers::handle_update_catalog(&client, id, name).await?,
            AdminCommand::RevokeToken => handlers::handle_revoke_token(&client).await?,
            AdminCommand::RevokeTokenById { id } => handlers::handle_revoke_token_by_id(&client, id).await?,
            AdminCommand::ListMergeOperations => handlers::handle_list_merge_operations(&client).await?,
            AdminCommand::GetMergeOperation { id } => handlers::handle_get_merge_operation(&client, id).await?,
            AdminCommand::ListConflicts { merge_id } => handlers::handle_list_conflicts(&client, merge_id).await?,
            AdminCommand::ResolveConflict { merge_id, conflict_id, resolution } => handlers::handle_resolve_conflict(&client, merge_id, conflict_id, resolution).await?,
            AdminCommand::CompleteMerge { id } => handlers::handle_complete_merge(&client, id).await?,
            AdminCommand::AbortMerge { id } => handlers::handle_abort_merge(&client, id).await?,
            AdminCommand::DeleteMetadata { asset_id } => handlers::handle_delete_metadata(&client, asset_id).await?,
            AdminCommand::RequestAccess { asset_id, reason } => handlers::handle_request_access(&client, asset_id, reason).await?,
            AdminCommand::ListAccessRequests => handlers::handle_list_access_requests(&client).await?,
            AdminCommand::UpdateAccessRequest { id, status } => handlers::handle_update_access_request(&client, id, status).await?,
            AdminCommand::GetAssetDetails { id } => handlers::handle_get_asset_details(&client, id).await?,
            AdminCommand::ListAuditEvents { user_id, action, resource_type, result, limit } => handlers::handle_list_audit_events(&client, user_id, action, resource_type, result, limit).await?,
            AdminCommand::CountAuditEvents { user_id, action, resource_type, result } => handlers::handle_count_audit_events(&client, user_id, action, resource_type, result).await?,
            AdminCommand::GetAuditEvent { id } => handlers::handle_get_audit_event(&client, id).await?,
            AdminCommand::ListUserTokens { user_id } => handlers::handle_list_user_tokens(&client, user_id).await?,
            AdminCommand::DeleteToken { token_id } => handlers::handle_delete_token(&client, token_id).await?,
            AdminCommand::GetSystemSettings => handlers::handle_get_system_settings(&client).await?,
            AdminCommand::UpdateSystemSettings { allow_public_signup, default_warehouse_bucket, default_retention_days } => handlers::handle_update_system_settings(&client, allow_public_signup, default_warehouse_bucket, default_retention_days).await?,
            AdminCommand::SyncFederatedCatalog { name } => handlers::handle_sync_federated_catalog(&client, name).await?,
            AdminCommand::GetFederatedStats { name } => handlers::handle_get_federated_stats(&client, name).await?,
            AdminCommand::Stats => handlers::handle_stats(&client).await?,
            AdminCommand::CatalogSummary { name } => handlers::handle_catalog_summary(&client, name).await?,
            AdminCommand::Search { query, catalog, limit } => handlers::handle_search(&client, query, catalog, limit).await?,
            AdminCommand::BulkDelete { ids, confirm } => handlers::handle_bulk_delete(&client, ids, confirm).await?,
            AdminCommand::Validate { resource_type, names } => handlers::handle_validate_names(&client, resource_type, names).await?,
            AdminCommand::ListNamespaceTree { catalog } => handlers::handle_list_namespace_tree(&client, catalog).await?,
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
        let prompt = match (&client.config.username, &client.config.tenant_name) {
            (Some(user), Some(tenant)) => format!("(admin:{}@{})> ", user, tenant),
            (Some(user), None) => format!("(admin:{})> ", user),
            (None, _) => "(admin:unauth)> ".to_string(),
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
                                    AdminCommand::Use { name } => {
                                         if let Err(e) = handlers::handle_use(&mut client, name).await {
                                            eprintln!("Error: {}", e);
                                        } else {
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
            AdminCommand::CreateTenant { name, admin_username, admin_password } => handlers::handle_create_tenant(&client, name, admin_username, admin_password).await?,
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
                                    AdminCommand::CreateWarehouse { name, type_, bucket, access_key, secret_key, region, endpoint } => {
                                        if let Err(e) = handlers::handle_create_warehouse(&client, name, type_, bucket, access_key, secret_key, region, endpoint).await { eprintln!("Error: {}", e); }
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
                                    AdminCommand::CreateFederatedCatalog { name, storage_location, properties } => {
                                        if let Err(e) = handlers::handle_create_federated_catalog(&client, name, storage_location, properties).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListFederatedCatalogs => {
                                        if let Err(e) = handlers::handle_list_federated_catalogs(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::DeleteFederatedCatalog { name } => {
                                        if let Err(e) = handlers::handle_delete_federated_catalog(&client, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::TestFederatedCatalog { name } => {
                                        if let Err(e) = handlers::handle_test_federated_catalog(&client, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListPermissions { role, user } => {
                                        if let Err(e) = handlers::handle_list_permissions(&client, role, user).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GrantPermission { username, action, resource } => {
                                        if let Err(e) = handlers::handle_grant_permission(&client, username, action, resource).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::RevokePermission { role, action, resource } => {
                                        if let Err(e) = handlers::handle_revoke_permission(&client, role, action, resource).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetMetadata { entity_type, entity_id } => {
                                        if let Err(e) = handlers::handle_get_metadata(&client, entity_type, entity_id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::SetMetadata { entity_type, entity_id, key, value } => {
                                        if let Err(e) = handlers::handle_set_metadata(&client, entity_type, entity_id, key, value).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::CreateServiceUser { name, description, role, expires_in_days } => {
                                        if let Err(e) = handlers::handle_create_service_user(&client, name, description, role, expires_in_days).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListServiceUsers => {
                                        if let Err(e) = handlers::handle_list_service_users(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetServiceUser { id } => {
                                        if let Err(e) = handlers::handle_get_service_user(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::UpdateServiceUser { id, name, description, active } => {
                                        if let Err(e) = handlers::handle_update_service_user(&client, id, name, description, active).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::DeleteServiceUser { id } => {
                                        if let Err(e) = handlers::handle_delete_service_user(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::RotateServiceUserKey { id } => {
                                        if let Err(e) = handlers::handle_rotate_service_user_key(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::UpdateTenant { id, name } => {
                                        if let Err(e) = handlers::handle_update_tenant(&client, id, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::UpdateUser { id, username, email, active } => {
                                        if let Err(e) = handlers::handle_update_user(&client, id, username, email, active).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::UpdateWarehouse { id, name } => {
                                        if let Err(e) = handlers::handle_update_warehouse(&client, id, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::UpdateCatalog { id, name } => {
                                        if let Err(e) = handlers::handle_update_catalog(&client, id, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::RevokeToken => {
                                        if let Err(e) = handlers::handle_revoke_token(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::RevokeTokenById { id } => {
                                        if let Err(e) = handlers::handle_revoke_token_by_id(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListMergeOperations => {
                                        if let Err(e) = handlers::handle_list_merge_operations(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetMergeOperation { id } => {
                                        if let Err(e) = handlers::handle_get_merge_operation(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListConflicts { merge_id } => {
                                        if let Err(e) = handlers::handle_list_conflicts(&client, merge_id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ResolveConflict { merge_id, conflict_id, resolution } => {
                                        if let Err(e) = handlers::handle_resolve_conflict(&client, merge_id, conflict_id, resolution).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::CompleteMerge { id } => {
                                        if let Err(e) = handlers::handle_complete_merge(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::AbortMerge { id } => {
                                        if let Err(e) = handlers::handle_abort_merge(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::DeleteMetadata { asset_id } => {
                                        if let Err(e) = handlers::handle_delete_metadata(&client, asset_id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::RequestAccess { asset_id, reason } => {
                                        if let Err(e) = handlers::handle_request_access(&client, asset_id, reason).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListAccessRequests => {
                                        if let Err(e) = handlers::handle_list_access_requests(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::UpdateAccessRequest { id, status } => {
                                        if let Err(e) = handlers::handle_update_access_request(&client, id, status).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetAssetDetails { id } => {
                                        if let Err(e) = handlers::handle_get_asset_details(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListAuditEvents { user_id, action, resource_type, result, limit } => {
                                        if let Err(e) = handlers::handle_list_audit_events(&client, user_id, action, resource_type, result, limit).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::CountAuditEvents { user_id, action, resource_type, result } => {
                                        if let Err(e) = handlers::handle_count_audit_events(&client, user_id, action, resource_type, result).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetAuditEvent { id } => {
                                        if let Err(e) = handlers::handle_get_audit_event(&client, id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::Stats => {
                                        if let Err(e) = handlers::handle_stats(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::CatalogSummary { name } => {
                                        if let Err(e) = handlers::handle_catalog_summary(&client, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::Search { query, catalog, limit } => {
                                        if let Err(e) = handlers::handle_search(&client, query, catalog, limit).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::BulkDelete { ids, confirm } => {
                                        if let Err(e) = handlers::handle_bulk_delete(&client, ids, confirm).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::Validate { resource_type, names } => {
                                        if let Err(e) = handlers::handle_validate_names(&client, resource_type, names).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListUserTokens { user_id } => {
                                        if let Err(e) = handlers::handle_list_user_tokens(&client, user_id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::DeleteToken { token_id } => {
                                        if let Err(e) = handlers::handle_delete_token(&client, token_id).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetSystemSettings => {
                                        if let Err(e) = handlers::handle_get_system_settings(&client).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::UpdateSystemSettings { allow_public_signup, default_warehouse_bucket, default_retention_days } => {
                                        if let Err(e) = handlers::handle_update_system_settings(&client, allow_public_signup, default_warehouse_bucket, default_retention_days).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::SyncFederatedCatalog { name } => {
                                        if let Err(e) = handlers::handle_sync_federated_catalog(&client, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::GetFederatedStats { name } => {
                                        if let Err(e) = handlers::handle_get_federated_stats(&client, name).await { eprintln!("Error: {}", e); }
                                    },
                                    AdminCommand::ListNamespaceTree { catalog } => {
                                        if let Err(e) = handlers::handle_list_namespace_tree(&client, catalog).await { eprintln!("Error: {}", e); }
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
