use std::sync::Arc;
use tokio::sync::Mutex;

use crate::utils::api::ApiManager;
use crate::utils::config::ConfigManager;
use crate::utils::conversation::ConversationManager;
use crate::utils::memory::MemoryManager;

#[derive(Clone, Debug)]
pub struct BotData {
    pub config: Arc<Mutex<ConfigManager>>,
    pub api_manager: Arc<ApiManager>,
    pub memory_manager: Arc<MemoryManager>,
    pub conversation_manager: Arc<ConversationManager>,
    pub initial_history_loaded: Arc<Mutex<std::collections::HashSet<u64>>>, // 跟蹤已載入歷史的頻道
    pub skills_db: tokio_rusqlite::Connection,
    #[allow(dead_code)] // 將在未來實現
    pub base_settings_db: tokio_rusqlite::Connection,
}
