use crate::models::types::{GlobalConfig, GuildConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ConfigManager {
    pub global: GlobalConfig,
    pub guilds: HashMap<u64, GuildConfig>,
    config_path: String,
}

impl ConfigManager {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut manager = Self {
            global: GlobalConfig::default(),
            guilds: HashMap::new(),
            config_path: config_path.to_string(),
        };
        
        // Load existing config or create default
        manager.load_config()?;
        Ok(manager)
    }

    pub fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(&self.config_path).exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let config_data: ConfigData = serde_json::from_str(&content)?;
            
            self.global = config_data.global.unwrap_or_default();
            self.guilds = config_data.guilds.unwrap_or_default();
        } else {
            // Create default config file
            self.save_config()?;
        }
        
        Ok(())
    }

    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn set_guild_config(&mut self, guild_id: u64, config: GuildConfig) {
        self.guilds.insert(guild_id, config);
    }

    pub fn is_developer(&self, user_id: u64) -> bool {
        self.global.developers.contains(&user_id)
    }

    pub fn add_developer(&mut self, user_id: u64) {
        if !self.global.developers.contains(&user_id) {
            self.global.developers.push(user_id);
        }
    }

    pub fn remove_developer(&mut self, user_id: u64) {
        self.global.developers.retain(|&id| id != user_id);
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
        let config = ConfigManager::new("test_config.json").unwrap();
        assert_eq!(config.global.developers.len(), 0);
        assert_eq!(config.guilds.len(), 0);
    }
}