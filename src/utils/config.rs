use crate::models::types::{GlobalConfig, GuildConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;
use tokio::sync::RwLock;
use std::sync::Arc;
use notify::{Watcher, RecursiveMode, recommended_watcher, EventKind};
use tokio::sync::watch;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Watcher error: {0}")]
    Watcher(#[from] notify::Error),
}

#[derive(Debug)]
pub struct ConfigManager {
    pub global: Arc<tokio::sync::RwLock<GlobalConfig>>,
    pub guilds: Arc<tokio::sync::RwLock<HashMap<u64, GuildConfig>>>,
    config_path: String,
    _watcher: Arc<std::sync::Mutex<Option<notify::RecommendedWatcher>>>,
    reload_tx: watch::Sender<()>,
}

impl ConfigManager {
    pub async fn new(config_path: &str) -> Result<Self, ConfigError> {
        let mut manager = Self {
            global: Arc::new(RwLock::new(GlobalConfig::default())),
            guilds: Arc::new(RwLock::new(HashMap::new())),
            config_path: config_path.to_string(),
            _watcher: Arc::new(std::sync::Mutex::new(None)),
            reload_tx: watch::channel(()).0,
        };

        manager.load_config().await?;
        manager.start_watching()?;
        Ok(manager)
    }

    fn start_watching(&mut self) -> Result<(), ConfigError> {
        let config_path = self.config_path.clone();
        let global = Arc::clone(&self.global);
        let guilds = Arc::clone(&self.guilds);
        let reload_tx = self.reload_tx.clone();

        // 建立文件監視器
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new(&config_path), RecursiveMode::NonRecursive)?;

        // 後臺線程監視文件變化
        std::thread::spawn(move || {
            for res in rx {
                match res {
                    Ok(event) => {
                        if matches!(event.kind, EventKind::Modify(_)) {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            // 重新載入配置
                            if let Ok(content) = std::fs::read_to_string(&config_path) {
                                if let Ok(config_data) = serde_json::from_str::<ConfigData>(&content) {
                                    // 全域
                                    let mut global_write = futures::executor::block_on(global.write());
                                    *global_write = config_data.global.unwrap_or_default();
                                    
                                    // 群組
                                    let mut guilds_write = futures::executor::block_on(guilds.write());
                                    *guilds_write = config_data.guilds.unwrap_or_default();
                                    
                                    // 發送重載通知
                                    let _ = reload_tx.send(());
                                    log::info!("配置文件已重新加載: {}", config_path);
                                }
                            }
                        }
                    }
                    Err(e) => log::error!("配置監視器錯誤: {:?}", e),
                }
            }
        });

        // 保存watcher
        let mut watcher_guard = self._watcher.lock().unwrap();
        *watcher_guard = Some(watcher);

        Ok(())
    }

    pub async fn load_config(&mut self) -> Result<(), ConfigError> {
        if Path::new(&self.config_path).exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let mut config_data: ConfigData = serde_json::from_str(&content)?;

            // 檢查並轉換舊格式的API配置為新格式
            if let Some(ref mut guilds) = config_data.guilds {
                for (_, guild_config) in guilds.iter_mut() {
                    // 如果存在舊格式的api_config，則轉換為新格式
                    if guild_config.api_config.is_some() {
                        let old_api_config = guild_config.api_config.take().unwrap();
                        // 為舊配置設定一個預設名稱
                        let name = if old_api_config.api_url.is_empty() {
                            "default".to_string()
                        } else {
                            old_api_config.api_url.clone()
                        };
                        
                        // 設定名稱
                        let mut new_api_config = old_api_config;
                        new_api_config.name = name.clone();
                        
                        // 初始化api_configs映射並添加配置
                        guild_config.api_configs.insert(name.clone(), new_api_config);
                        // 將此配置設為活動配置
                        guild_config.active_api = Some(name);
                    }
                }
            }

            *self.global.write().await = config_data.global.unwrap_or_default();
            *self.guilds.write().await = config_data.guilds.unwrap_or_default();
        } else {
            self.save_config().await?;
        }

        Ok(())
    }

    pub async fn save_config(&self) -> Result<(), ConfigError> {
        let global_read = self.global.read().await;
        let guilds_read = self.guilds.read().await;
        
        let config_data = ConfigData {
            global: Some(global_read.clone()),
            guilds: Some(guilds_read.clone()),
        };

        let content = serde_json::to_string_pretty(&config_data)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub async fn get_guild_config(&self, guild_id: u64) -> GuildConfig {
        let guilds_read = self.guilds.read().await;
        guilds_read.get(&guild_id).cloned().unwrap_or_default()
    }

    pub async fn set_guild_config(
        &self,
        guild_id: u64,
        config: GuildConfig,
    ) -> Result<(), ConfigError> {
        let mut guilds_write = self.guilds.write().await;
        guilds_write.insert(guild_id, config);
        self.save_config().await
    }

    pub async fn get_guild_api_config(&self, guild_id: u64) -> crate::utils::api::ApiConfig {
        let guilds_read = self.guilds.read().await;
        if let Some(guild_config) = guilds_read.get(&guild_id) {
            if let Some(ref active_api_name) = guild_config.active_api {
                // 嘗試獲取活動的API配置
                if let Some(api_config) = guild_config.api_configs.get(active_api_name) {
                    api_config.clone()
                } else {
                    // 如果活動的API配置不存在，返回默認配置
                    crate::utils::api::ApiConfig::default()
                }
            } else {
                // 如果沒有設置活動API，返回默認配置
                crate::utils::api::ApiConfig::default()
            }
        } else {
            crate::utils::api::ApiConfig::default()
        }
    }

    pub async fn add_guild_api_config(
        &self,
        guild_id: u64,
        api_config: crate::utils::api::ApiConfig,
    ) -> Result<(), ConfigError> {
        let mut guilds_write = self.guilds.write().await;
        let guild_config = guilds_write.entry(guild_id).or_insert_with(GuildConfig::default);
        let config_name = api_config.name.clone();
        guild_config.api_configs.insert(config_name.clone(), api_config);
        // 如果這是第一個配置，設為活動配置
        if guild_config.active_api.is_none() {
            guild_config.active_api = Some(config_name);
        }
        drop(guilds_write);
        self.save_config().await
    }

    pub async fn get_guild_api_configs(&self, guild_id: u64) -> std::collections::HashMap<String, crate::utils::api::ApiConfig> {
        let guilds_read = self.guilds.read().await;
        if let Some(guild_config) = guilds_read.get(&guild_id) {
            guild_config.api_configs.clone()
        } else {
            std::collections::HashMap::new()
        }
    }

    pub async fn remove_guild_api_config(&self, guild_id: u64, name: &str) -> Result<bool, ConfigError> {
        let mut guilds_write = self.guilds.write().await;
        let mut removed = false;
        if let Some(guild_config) = guilds_write.get_mut(&guild_id) {
            if guild_config.api_configs.remove(name).is_some() {
                removed = true;
                // 如果刪除的是活動API配置，則將活動API設為空或選擇其他配置
                if let Some(ref active_name) = guild_config.active_api {
                    if active_name == name {
                        if guild_config.api_configs.is_empty() {
                            guild_config.active_api = None;
                        } else {
                            // 選擇第一個可用的API配置作為活動配置
                            if let Some(first_key) = guild_config.api_configs.keys().next() {
                                guild_config.active_api = Some(first_key.clone());
                            }
                        }
                    }
                }
            }
        }
        drop(guilds_write);
        self.save_config().await?;
        Ok(removed)
    }

    pub async fn set_active_api(&self, guild_id: u64, name: &str) -> Result<bool, ConfigError> {
        let mut guilds_write = self.guilds.write().await;
        let mut success = false;
        if let Some(guild_config) = guilds_write.get_mut(&guild_id) {
            // 檢查是否有名為name的配置
            if guild_config.api_configs.contains_key(name) {
                guild_config.active_api = Some(name.to_string());
                success = true;
            }
        }
        drop(guilds_write);
        if success {
            self.save_config().await?;
        }
        Ok(success)
    }

    pub async fn is_developer(&self, user_id: u64) -> bool {
        let global_read = self.global.read().await;
        global_read.developers.contains(&user_id)
    }

    pub async fn add_developer(&self, user_id: u64) -> Result<bool, ConfigError> {
        let mut global_write = self.global.write().await;
        if global_write.developers.contains(&user_id) {
            return Ok(false);
        }

        global_write.developers.push(user_id);
        self.save_config().await?;
        Ok(true)
    }

    pub async fn remove_developer(&self, user_id: u64) -> Result<bool, ConfigError> {
        let mut global_write = self.global.write().await;
        let original_len = global_write.developers.len();
        global_write.developers.retain(|&id| id != user_id);

        if global_write.developers.len() == original_len {
            return Ok(false);
        }

        self.save_config().await?;
        Ok(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigData {
    global: Option<GlobalConfig>,
    guilds: Option<HashMap<u64, GuildConfig>>,
}

// 測試用異步訪問輔助函數
impl ConfigManager {
    pub async fn get_global_config(&self) -> GlobalConfig {
        let global_read = self.global.read().await;
        global_read.clone()
    }
}

#[cfg(test)]
// 測試模組
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let path = "test_config.json";
        let config = ConfigManager::new(path).await.expect("Failed to create ConfigManager in test");
        let global = config.get_global_config().await;
        assert!(!global.restart_mode.is_empty());
        let _ = std::fs::remove_file(path);
    }
}
