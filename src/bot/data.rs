use std::sync::Arc;
use tokio::sync::Mutex;

use crate::utils::config::ConfigManager;

#[derive(Clone)]
pub struct BotData {
    pub config: Arc<Mutex<ConfigManager>>,
    pub skills_db: tokio_rusqlite::Connection,
    pub base_settings_db: tokio_rusqlite::Connection,
}
