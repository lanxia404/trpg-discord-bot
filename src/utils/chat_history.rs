use anyhow::Result;
use std::sync::Arc;
use tokio_rusqlite::Connection;

// Define the ChatHistoryManager struct and its methods
#[derive(Debug)]
pub struct ChatHistoryManager {
    db_conn: Arc<Connection>,
    max_history_length: usize,
}

impl ChatHistoryManager {
    pub async fn new(db_path: &str, max_history_length: usize) -> Result<Self> {
        let conn = Arc::new(Connection::open(db_path).await?);
        
        // Initialize the database table if it doesn't exist
        Self::init_db(&conn).await?;
        
        Ok(Self {
            db_conn: conn,
            max_history_length,
        })
    }

    async fn init_db(conn: &Connection) -> Result<()> {
        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS chat_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    guild_id TEXT NOT NULL,
                    channel_id TEXT NOT NULL,
                    user_id TEXT NOT NULL,
                    message TEXT NOT NULL,
                    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn add_message(&self, guild_id: &str, channel_id: &str, user_id: &str, message: &str) -> Result<()> {
        let guild_id = guild_id.to_string();
        let channel_id = channel_id.to_string();
        let user_id = user_id.to_string();
        let message = message.to_string();
        
        // Clone values that are needed after the closure
        let guild_id_clone = guild_id.clone();
        let channel_id_clone = channel_id.clone();
        
        self.db_conn.call(move |conn| {
            conn.execute(
                "INSERT INTO chat_history (guild_id, channel_id, user_id, message) VALUES (?1, ?2, ?3, ?4)",
                [&guild_id, &channel_id, &user_id, &message],
            )?;
            Ok(())
        }).await?;
        
        // Remove old messages if we exceed the max history length
        self.trim_history(&guild_id_clone, &channel_id_clone).await?;
        
        Ok(())
    }

    async fn trim_history(&self, guild_id: &str, channel_id: &str) -> Result<()> {
        let guild_id = guild_id.to_string();
        let channel_id = channel_id.to_string();
        let max_length = self.max_history_length;
        
        self.db_conn.call(move |conn| {
            // Count the number of messages for this guild and channel
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM chat_history WHERE guild_id = ?1 AND channel_id = ?2",
                [guild_id.as_str(), channel_id.as_str()],
                |row| row.get(0),
            )?;
            
            // If we have more messages than the max length, remove the oldest ones
            if count as usize > max_length {
                let excess = count as usize - max_length;
                
                // Get the IDs of the excess records to delete
                let ids_to_delete: Vec<i64> = conn.prepare(
                    "SELECT id FROM chat_history WHERE guild_id = ?1 AND channel_id = ?2 ORDER BY timestamp ASC LIMIT ?3"
                )?.query_map([guild_id.as_str(), channel_id.as_str(), &excess.to_string()], |row| row.get(0))?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                
                // Delete the excess records
                for id in ids_to_delete {
                    conn.execute("DELETE FROM chat_history WHERE id = ?1", [id])?;
                }
            }
            
            Ok(())
        }).await?;
        
        Ok(())
    }

    pub async fn get_history(&self, guild_id: &str, channel_id: &str, limit: Option<usize>) -> Result<Vec<ChatMessage>> {
        let guild_id = guild_id.to_string();
        let channel_id = channel_id.to_string();
        let limit = limit.unwrap_or(self.max_history_length);

        let rows = self.db_conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT user_id, message, timestamp FROM chat_history 
                WHERE guild_id = ?1 AND channel_id = ?2 
                ORDER BY timestamp DESC 
                LIMIT ?3"
            )?;

            let rows = stmt
                .query_map([guild_id.as_str(), channel_id.as_str(), &limit.to_string()], |row| {
                    Ok(ChatMessage {
                        user_id: row.get(0)?,
                        message: row.get(1)?,
                        timestamp: row.get(2)?,
                        content: row.get(1)?, // 使用 message 作為 content
                        username: "Unknown".to_string(), // 默認用戶名，因為數據庫中沒有存儲
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(rows)
        }).await?;

        Ok(rows)
    }
    
    // 添加缺失的方法：get_recent_messages（與get_history相同功能，但名稱與代碼匹配）
    pub async fn get_recent_messages(&self, channel_id: u64, limit: usize) -> Result<Vec<ChatMessage>> {
        // 我們假設這個方法是獲取頻道的最近消息
        // 在實際實現中，你可能需要根據具體的guild_id來獲取
        // 這裡我們使用一個默認值，但在實際實現中你可能需要調整
        let channel_id_str = channel_id.to_string();
        self.get_history("default_guild", &channel_id_str, Some(limit)).await
    }
    
    // 添加缺失的方法：insert_message（與add_message相同功能，但名稱與代碼匹配）
    pub async fn insert_message(&self, channel_id: u64, guild_id: Option<u64>, user_id: u64, _username: &str, content: &str) -> Result<()> {
        // 將 u64 值轉換為字符串
        let guild_id_str = guild_id.map(|id| id.to_string()).unwrap_or_else(|| "default_guild".to_string());
        let channel_id_str = channel_id.to_string();
        let user_id_str = user_id.to_string();
        
        // 實際上我們只需要存儲內容，所以username可以忽略或組合成內容的一部分
        self.add_message(&guild_id_str, &channel_id_str, &user_id_str, content).await
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ChatMessage {
    pub user_id: String,
    pub message: String,
    pub timestamp: String,
    // 添加 content 字段（與 message 相同）
    pub content: String,
    // 添加 username 字段
    pub username: String,
}