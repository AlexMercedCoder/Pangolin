use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::error::CliError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
    pub username: Option<String>,
    pub tenant_id: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            auth_token: None,
            username: None,
            tenant_id: None,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(profile: Option<&str>) -> Result<Self, CliError> {
        let dirs = ProjectDirs::from("com", "pangolin", "cli")
            .ok_or(CliError::ConfigError("Could not determine config directory".to_string()))?;
        
        let filename = if let Some(p) = profile {
            format!("config-{}.json", p)
        } else {
            "config.json".to_string()
        };

        let config_path = dirs.config_dir().join(filename);
        
        Ok(Self { config_path })
    }

    pub fn load(&self) -> Result<CliConfig, CliError> {
        if !self.config_path.exists() {
            return Ok(CliConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        serde_json::from_str(&content).map_err(|e| CliError::ConfigError(e.to_string()))
    }

    pub fn save(&self, config: &CliConfig) -> Result<(), CliError> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| CliError::ConfigError(e.to_string()))?;
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}
