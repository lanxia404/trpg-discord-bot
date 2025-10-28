use crate::models::types::{GlobalConfig, GuildConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct ConfigManager {
    pub global: GlobalConfig,
    pub guilds: HashMap<u64, GuildConfig>,
    config_path: String,
}

impl ConfigManager {
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let mut manager = Self {
            global: GlobalConfig::default(),
            guilds: HashMap::new(),
            config_path: config_path.to_string(),
        };

        manager.load_config()?;
        Ok(manager)
    }

    pub fn load_config(&mut self) -> Result<(), ConfigError> {
        if Path::new(&self.config_path).exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let config_data: ConfigData = serde_json::from_str(&content)?;

            self.global = config_data.global.unwrap_or_default();
            self.guilds = config_data.guilds.unwrap_or_default();
        } else {
            self.save_config()?;
        }

        Ok(())
    }

    pub fn save_config(&self) -> Result<(), ConfigError> {
        let config_data = ConfigData {
            global: Some(self.global.clone()),
            guilds: Some(self.guilds.clone()),
        };

        let content = serde_json::to_string_pretty(&config_data)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn get_guild_config(&self, guild_id: u64) -> GuildConfig {
        self.guilds.get(&guild_id).cloned().unwrap_or_default()
    }

    pub fn set_guild_config(
        &mut self,
        guild_id: u64,
        config: GuildConfig,
    ) -> Result<(), ConfigError> {
        self.guilds.insert(guild_id, config);
        self.save_config()
    }

    pub fn is_developer(&self, user_id: u64) -> bool {
        self.global.developers.contains(&user_id)
    }

    pub fn add_developer(&mut self, user_id: u64) -> Result<bool, ConfigError> {
        if self.global.developers.contains(&user_id) {
            return Ok(false);
        }

        self.global.developers.push(user_id);
        self.save_config()?;
        Ok(true)
    }

    pub fn remove_developer(&mut self, user_id: u64) -> Result<bool, ConfigError> {
        let original_len = self.global.developers.len();
        self.global.developers.retain(|&id| id != user_id);

        if self.global.developers.len() == original_len {
            return Ok(false);
        }

        self.save_config()?;
        Ok(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigData {
    global: Option<GlobalConfig>,
    guilds: Option<HashMap<u64, GuildConfig>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let path = "config.json";
        let config = ConfigManager::new(path).expect("Failed to create ConfigManager in test");
        assert!(!config.global.restart_mode.is_empty());
    }
}
