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

        // 创建异步watcher
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new(&config_path), RecursiveMode::NonRecursive)?;

        // 在后台线程中处理文件变化
        std::thread::spawn(move || {
            for res in rx {
                match res {
                    Ok(event) => {
                        if matches!(event.kind, EventKind::Modify(_)) {
                            // 等待一小段时间以确保文件写入完成
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            
                            // 重新加载配置
                            if let Ok(content) = std::fs::read_to_string(&config_path) {
                                if let Ok(config_data) = serde_json::from_str::<ConfigData>(&content) {
                                    // 更新全局配置
                                    let mut global_write = futures::executor::block_on(global.write());
                                    *global_write = config_data.global.unwrap_or_default();
                                    
                                    // 更新公会配置
                                    let mut guilds_write = futures::executor::block_on(guilds.write());
                                    *guilds_write = config_data.guilds.unwrap_or_default();
                                    
                                    // 发送重载信号
                                    let _ = reload_tx.send(());
                                    log::info!("配置文件已热重载: {}", config_path);
                                }
                            }
                        }
                    }
                    Err(e) => log::error!("配置监视器错误: {:?}", e),
                }
            }
        });

        // 存储watcher实例
        let mut watcher_guard = self._watcher.lock().unwrap();
        *watcher_guard = Some(watcher);

        Ok(())
    }

    pub async fn load_config(&mut self) -> Result<(), ConfigError> {
        if Path::new(&self.config_path).exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let config_data: ConfigData = serde_json::from_str(&content)?;

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

// 添加一个用于测试异步访问的辅助函数
impl ConfigManager {
    pub async fn get_global_config(&self) -> GlobalConfig {
        let global_read = self.global.read().await;
        global_read.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let path = "test_config.json";
        let config = ConfigManager::new(path).expect("Failed to create ConfigManager in test");
        let global = config.get_global_config().await;
        assert!(!global.restart_mode.is_empty());
        
        // 清理测试文件
        let _ = std::fs::remove_file(path);
    }
}
