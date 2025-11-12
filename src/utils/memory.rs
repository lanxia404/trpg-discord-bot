use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};
use bincode;
use rusqlite;
use crate::models::types::VectorStorageMethod;

// 記憶條目結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: i32,
    pub user_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub content: String,
    pub content_type: String, // message, summary, setting, etc.
    pub importance_score: f32,
    pub tags: String,
    pub enabled: bool,
    pub created_at: String,
    pub last_accessed: String,
    pub embedding_vector: Option<Vec<f32>>, // 向量嵌入
}

// 搜尋選項
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub max_results: usize,
    pub guild_id: Option<String>,
    pub user_id: Option<String>,
    pub channel_id: Option<String>,
    pub tags: Option<String>,
}

// 重要性計算的元數據
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ImportanceMetadata {
    pub mention_count: Option<usize>,
    pub reaction_count: Option<usize>,
    pub has_reference: bool,
}

use crate::utils::api::ApiManager;

#[derive(Debug)]
pub struct MemoryManager {
    db_conn: Arc<Connection>,
    #[allow(dead_code)]
    api_manager: Option<Arc<ApiManager>>, // 可選的API管理器,用於獲取嵌入向量
    vector_storage_method: VectorStorageMethod, // 向量儲存計算方式
}

impl MemoryManager {
    pub async fn new(db_path: &str, api_manager: Option<Arc<ApiManager>>, vector_storage_method: VectorStorageMethod) -> Result<Self> {
        // 確保資料庫目錄存在且可寫
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 嘗試打開資料庫，如果失敗則提供詳細錯誤訊息
        let conn = Arc::new(Connection::open(db_path).await.map_err(|e| {
            log::error!("無法打開資料庫 {}: {}", db_path, e);
            anyhow::Error::msg(format!("資料庫打開失敗: {}", e))
        })?);
        
        // 測試資料庫是否可寫
        conn.call(|conn| {
            conn.execute("CREATE TABLE IF NOT EXISTS _write_test (id INTEGER)", [])?;
            conn.execute("DROP TABLE IF EXISTS _write_test", [])?;
            Ok(())
        }).await.map_err(|e| {
            log::error!("資料庫寫入測試失敗 {}: {}", db_path, e);
            anyhow::Error::msg(format!("資料庫不可寫: {}. 請檢查檔案權限", e))
        })?;
        
        // 初始化數據庫表
        Self::init_db(&conn).await?;

        log::info!("記憶管理器初始化成功: {}", db_path);
        
        Ok(Self {
            db_conn: conn,
            api_manager,
            vector_storage_method,
        })
    }

    async fn init_db(conn: &Connection) -> Result<()> {
        conn.call(|conn| {
            // 創建記憶表，包含向量存儲欄位
            conn.execute(
                "CREATE TABLE IF NOT EXISTS memory_embeddings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id TEXT NOT NULL,
                    guild_id TEXT,
                    channel_id TEXT,
                    content TEXT NOT NULL,
                    content_type TEXT DEFAULT 'message',
                    importance_score REAL DEFAULT 0.0,
                    tags TEXT,
                    enabled BOOLEAN DEFAULT 1,
                    created_at TEXT NOT NULL,
                    last_accessed TEXT NOT NULL,
                    embedding_vector BLOB  -- 用於存儲序列化的向量
                )",
                [],
            )?;
            
            // 創建索引以提高搜尋效率
            conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_user_guild ON memory_embeddings(user_id, guild_id)", [])?;
            conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_channel ON memory_embeddings(channel_id)", [])?;
            conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_enabled ON memory_embeddings(enabled)", [])?;
            
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn save_memory(&self, mut memory_entry: MemoryEntry) -> Result<i32> {
        // 如果嵌入向量尚未生成，則生成它
        if memory_entry.embedding_vector.is_none() {
            memory_entry.embedding_vector = Some(self.generate_embedding_for_text(&memory_entry.content).await?);
        }
        
        let user_id = memory_entry.user_id.clone();
        let guild_id = memory_entry.guild_id.clone();
        let channel_id = memory_entry.channel_id.clone();
        let content = memory_entry.content.clone();
        let content_type = memory_entry.content_type.clone();
        let importance_score = memory_entry.importance_score;
        let tags = memory_entry.tags.clone();
        let enabled = memory_entry.enabled;
        let created_at = memory_entry.created_at.clone();
        let last_accessed = memory_entry.last_accessed.clone();
        let embedding_vector = memory_entry.embedding_vector.clone();

        // 序列化嵌入向量
        let embedding_bytes = serialize_embedding(&embedding_vector);

        let id = self.db_conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "INSERT INTO memory_embeddings (user_id, guild_id, channel_id, content, content_type, importance_score, tags, enabled, created_at, last_accessed, embedding_vector) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            )?;
            stmt.execute((
                &user_id,
                &guild_id,
                &channel_id,
                &content,
                &content_type,
                &importance_score,
                &tags,
                &(enabled as i32),
                &created_at,
                &last_accessed,
                &embedding_bytes,
            ))?;
            Ok(conn.last_insert_rowid() as i32)
        }).await?;

        Ok(id)
    }

    pub async fn search_memory(&self, query: &str, options: &SearchOptions) -> Result<Vec<MemoryEntry>> {
        // 生成查詢向量
        let query_embedding = self.generate_embedding_for_text(query).await?;
        
        let guild_id = options.guild_id.clone().unwrap_or_default();
        let user_id = options.user_id.clone().unwrap_or_default();
        let channel_id = options.channel_id.clone().unwrap_or_default();
        let tags = options.tags.clone().unwrap_or_default();
        let max_results = max_results_to_i32(options.max_results);

        let rows = self.db_conn.call(move |conn| {
            // 構建 SQL 查詢
            let mut sql = String::from("SELECT id, user_id, guild_id, channel_id, content, content_type, importance_score, tags, enabled, created_at, last_accessed, embedding_vector FROM memory_embeddings WHERE enabled = 1");
            let mut params = Vec::new();

            if !guild_id.is_empty() {
                sql.push_str(" AND guild_id = ?");
                params.push(guild_id);
            }

            if !user_id.is_empty() {
                sql.push_str(" AND user_id = ?");
                params.push(user_id);
            }

            if !channel_id.is_empty() {
                sql.push_str(" AND channel_id = ?");
                params.push(channel_id);
            }

            if !tags.is_empty() {
                sql.push_str(" AND tags LIKE ?");
                params.push(format!("%{}%", tags));
            }

            sql.push_str(" ORDER BY importance_score DESC LIMIT ?");
            params.push(max_results.to_string());

            // 創建參數數組並根據數量選擇合適的方法
            match params.len() {
                0 => {
                    let mut stmt = conn.prepare(&sql)?;
                    let rows = stmt.query_map([], |row| {
                        process_row_result(row)
                    })?;
                    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                },
                1 => {
                    let mut stmt = conn.prepare(&sql)?;
                    let rows = stmt.query_map([params[0].as_str()], |row| {
                        process_row_result(row)
                    })?;
                    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                },
                2 => {
                    let mut stmt = conn.prepare(&sql)?;
                    let rows = stmt.query_map([params[0].as_str(), params[1].as_str()], |row| {
                        process_row_result(row)
                    })?;
                    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                },
                3 => {
                    let mut stmt = conn.prepare(&sql)?;
                    let rows = stmt.query_map([params[0].as_str(), params[1].as_str(), params[2].as_str()], |row| {
                        process_row_result(row)
                    })?;
                    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                },
                4 => {
                    let mut stmt = conn.prepare(&sql)?;
                    let rows = stmt.query_map([params[0].as_str(), params[1].as_str(), params[2].as_str(), params[3].as_str()], |row| {
                        process_row_result(row)
                    })?;
                    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                },
                5 => {
                    let mut stmt = conn.prepare(&sql)?;
                    let rows = stmt.query_map([params[0].as_str(), params[1].as_str(), params[2].as_str(), params[3].as_str(), params[4].as_str()], |row| {
                        process_row_result(row)
                    })?;
                    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                },
                _ => {
                    // 如果超出預期參數數量，只處理前5個
                    let mut stmt = conn.prepare(&sql)?;
                    let valid_params = &params[..std::cmp::min(5, params.len())];
                    match valid_params.len() {
                        1 => {
                            let rows = stmt.query_map([valid_params[0].as_str()], |row| {
                                process_row_result(row)
                            })?;
                            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                        },
                        2 => {
                            let rows = stmt.query_map([valid_params[0].as_str(), valid_params[1].as_str()], |row| {
                                process_row_result(row)
                            })?;
                            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                        },
                        3 => {
                            let rows = stmt.query_map([valid_params[0].as_str(), valid_params[1].as_str(), valid_params[2].as_str()], |row| {
                                process_row_result(row)
                            })?;
                            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                        },
                        4 => {
                            let rows = stmt.query_map([valid_params[0].as_str(), valid_params[1].as_str(), valid_params[2].as_str(), valid_params[3].as_str()], |row| {
                                process_row_result(row)
                            })?;
                            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                        },
                        5 => {
                            let rows = stmt.query_map([valid_params[0].as_str(), valid_params[1].as_str(), valid_params[2].as_str(), valid_params[3].as_str(), valid_params[4].as_str()], |row| {
                                process_row_result(row)
                            })?;
                            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                        },
                        _ => {
                            let rows = stmt.query_map([], |row| {
                                process_row_result(row)
                            })?;
                            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
                        }
                    }
                }
            }
        }).await?;

        // 計算與查詢的語意相似度 (簡化實現，僅返回前N個結果)
        // 在實際實現中，這裡應該計算向量之間的餘弦相似度
        let mut scored_rows = rows;
        for entry in &mut scored_rows {
            // 模擬相似度計算
            entry.importance_score = calculate_similarity(&query_embedding, &entry.embedding_vector)?;
        }

        // 按相似度排序並返回前 N 個結果
        scored_rows.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap_or(std::cmp::Ordering::Equal));
        scored_rows.truncate(options.max_results);

        // 更新找到的記憶的訪問時間
        for entry in &scored_rows {
            let _ = self.update_last_accessed(entry.id).await;  // 暱藏錯誤
        }

        Ok(scored_rows)
    }

    pub async fn list_memory(&self, user_id: &str, guild_id: &str, offset: i32, limit: i32) -> Result<Vec<MemoryEntry>> {
        let user_id = user_id.to_string();
        let guild_id = guild_id.to_string();
        
        let rows = self.db_conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, guild_id, channel_id, content, content_type, importance_score, tags, enabled, created_at, last_accessed, embedding_vector 
                 FROM memory_embeddings 
                 WHERE user_id = ?1 AND guild_id = ?2 AND enabled = 1
                 ORDER BY created_at DESC
                 LIMIT ?3 OFFSET ?4"
            )?;
            
            let rows = stmt
                .query_map([&user_id, &guild_id, &limit.to_string(), &offset.to_string()], |row| {
                    process_row_result(row)
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
                
            Ok(rows)
        }).await?;

        Ok(rows)
    }

    pub async fn delete_memory(&self, id: i32, user_id: &str, guild_id: &str) -> Result<bool> {
        let user_id = user_id.to_string();
        let guild_id = guild_id.to_string();
        let id_str = id.to_string();
        
        let changes = self.db_conn.call(move |conn| {
            let changes = conn.execute(
                "DELETE FROM memory_embeddings WHERE id = ?1 AND user_id = ?2 AND guild_id = ?3",
                [&id_str, &user_id, &guild_id],
            )?;
            Ok(changes)
        }).await?;

        Ok(changes > 0)
    }

    pub async fn clear_memory(&self, user_id: &str, guild_id: &str) -> Result<i32> {
        let user_id = user_id.to_string();
        let guild_id = guild_id.to_string();
        
        let changes = self.db_conn.call(move |conn| {
            let changes = conn.execute(
                "DELETE FROM memory_embeddings WHERE user_id = ?1 AND guild_id = ?2",
                [&user_id, &guild_id],
            )?;
            Ok(changes as i32)
        }).await?;

        Ok(changes)
    }
    
    // 更新最後訪問時間
    pub async fn update_last_accessed(&self, id: i32) -> Result<()> {
        let timestamp = get_current_timestamp();
        self.db_conn.call(move |conn| {
            let sql = format!(
                "UPDATE memory_embeddings SET last_accessed = '{}' WHERE id = {}",
                timestamp,
                id
            );
            conn.execute(&sql, [])?;
            Ok(())
        }).await?;
        Ok(())
    }
    
    // 添加傳統對話歷史功能
    pub async fn add_message(&self, guild_id: &str, channel_id: &str, user_id: &str, message: &str) -> Result<()> {
        let guild_id = guild_id.to_string();
        let channel_id = channel_id.to_string();
        let user_id = user_id.to_string();
        let message = message.to_string();

        // 保存到記憶系統
        let memory_entry = MemoryEntry {
            id: 0, // ID 將由數據庫自動生成
            user_id: user_id.clone(),
            guild_id: guild_id.clone(),
            channel_id: channel_id.clone(),
            content: message.clone(),
            content_type: "message".to_string(),
            importance_score: 0.0, // 可以根據消息特徵計算重要性
            tags: "".to_string(),
            enabled: true,
            created_at: get_current_timestamp(),
            last_accessed: get_current_timestamp(),
            embedding_vector: None, // 將在 save_memory 中生成
        };

        self.save_memory(memory_entry).await?;

        Ok(())
    }

    pub async fn get_history(&self, guild_id: &str, channel_id: &str, limit: Option<usize>) -> Result<Vec<ChatMessage>> {
        let guild_id = guild_id.to_string();
        let channel_id = channel_id.to_string();
        let limit = limit.unwrap_or(50) as i32;
        
        let rows = self.db_conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT user_id, content, created_at FROM memory_embeddings 
                WHERE guild_id = ?1 AND channel_id = ?2 AND enabled = 1
                ORDER BY created_at DESC
                LIMIT ?3"
            )?;
            
            let rows = stmt
                .query_map([&guild_id, &channel_id, &limit.to_string()], |row| {
                    // 嘗試獲取 created_at，支持 TEXT 和 INTEGER 兩種類型
                    let timestamp: String = match row.get::<_, String>(2) {
                        Ok(t) => t,
                        Err(_) => {
                            // 如果是 INTEGER 類型，轉換為字符串
                            match row.get::<_, i64>(2) {
                                Ok(t) => t.to_string(),
                                Err(_) => chrono::Utc::now().to_rfc3339(),
                            }
                        }
                    };
                    
                    Ok(ChatMessage {
                        user_id: row.get(0)?,
                        message: row.get(1)?,
                        timestamp,
                        content: row.get(1)?,
                        username: "Unknown".to_string(),
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(rows)
        }).await?;

        Ok(rows)
    }

    // 添加缺失的方法：get_recent_messages（與get_history相同功能，但名稱與代碼匹配）
    pub async fn get_recent_messages(&self, guild_id: u64, channel_id: u64, limit: usize) -> Result<Vec<ChatMessage>> {
        let guild_id_str = guild_id.to_string();
        let channel_id_str = channel_id.to_string();
        self.get_history(&guild_id_str, &channel_id_str, Some(limit)).await
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

// 計算向量相似度的輔助函數
fn calculate_similarity(query_embedding: &[f32], entry_embedding: &Option<Vec<f32>>) -> Result<f32> {
    if let Some(entry_vec) = entry_embedding {
        if query_embedding.is_empty() || entry_vec.is_empty() {
            return Ok(0.0); // 如果任一向量為空，相似度為0
        }
        
        // 確保向量維度匹配
        let min_len = std::cmp::min(query_embedding.len(), entry_vec.len());
        
        // 計算餘弦相似度
        let mut dot_product: f32 = 0.0;
        let mut magnitude_a: f32 = 0.0;
        let mut magnitude_b: f32 = 0.0;
        
        for i in 0..min_len {
            dot_product += query_embedding[i] * entry_vec[i];
            magnitude_a += query_embedding[i] * query_embedding[i];
            magnitude_b += entry_vec[i] * entry_vec[i];
        }
        
        let magnitude_a = magnitude_a.sqrt();
        let magnitude_b = magnitude_b.sqrt();
        
        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            Ok(0.0)
        } else {
            Ok(dot_product / (magnitude_a * magnitude_b))
        }
    } else {
        Ok(0.0) // 沒有嵌入向量，相似度為0
    }
}

// 序列化嵌入向量為字節數組
fn serialize_embedding(embedding: &Option<Vec<f32>>) -> Vec<u8> {
    if let Some(vec) = embedding {
        bincode::serialize(vec).unwrap_or_default()
    } else {
        Vec::new()
    }
}

// 從字節數組反序列化嵌入向量
fn deserialize_embedding(bytes: &[u8]) -> Result<Option<Vec<f32>>> {
    if bytes.is_empty() {
        return Ok(None);
    }
    
    match bincode::deserialize(bytes) {
        Ok(vec) => Ok(Some(vec)),
        Err(e) => Err(anyhow::Error::msg(format!("Deserialization error: {}", e))),
    }
}

// 在 MemoryManager impl 塊中添加方法生成嵌入向量
impl MemoryManager {
    async fn generate_embedding_for_text(&self, text: &str) -> Result<Vec<f32>> {
        // 根據配置的存儲方式選擇向量計算方法
        match &self.vector_storage_method {
            VectorStorageMethod::Local => {
                // 使用本地算法
                Ok(self.generate_embedding_locally(text))
            },
            VectorStorageMethod::EmbeddingApi => {
                // API embedding 需要 guild_id 上下文
                // 在階段 3 實現 API Manager 的 embedding 支援後啟用
                // 目前回退到本地算法
                log::debug!("EmbeddingApi 模式尚未完全實現,使用本地 TF-IDF");
                Ok(self.generate_embedding_locally(text))
            },
            VectorStorageMethod::VectorDatabase => {
                // 如果使用向量數據庫,通常在外部進行向量計算和檢索
                // 這裏回退到本地算法,實際的向量數據庫集成需要額外實現
                Ok(self.generate_embedding_locally(text))
            }
        }
    }

    // 本地生成嵌入向量的函數
    fn generate_embedding_locally(&self, text: &str) -> Vec<f32> {
        use std::collections::HashMap;
        
        // 使用簡化的TF-IDF算法生成嵌入向量
        let tokens = simple_tokenize(text);
        let mut term_freq: HashMap<String, f32> = HashMap::new();
        
        for token in &tokens {
            *term_freq.entry(token.clone()).or_insert(0.0) += 1.0;
        }
        
        // 計算嵌入向量（簡化的TF-IDF）
        let mut embedding = Vec::with_capacity(1536); // OpenAI嵌入向量維度
        
        // 使用詞彙表的簡單哈希生成固定長度向量
        for i in 0..1536 {
            let mut value = 0.0;
            for (token, freq) in &term_freq {
                // 使用詞的哈希值結合維度索引來生成特定位的值
                let hash = hash_str_to_f32(&format!("{}{}", token, i));
                value += hash * freq;
            }
            embedding.push(value);
        }
        
        // 正規化向量
        let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            embedding = embedding.iter().map(|x| x / magnitude).collect();
        }
        
        embedding
    }

    // 從API獲取嵌入向量的函數
    /// 設置向量存儲方法
    #[allow(dead_code)]  // 在某些部署配置中可能未使用，保留以供將來擴展
    pub fn set_vector_storage_method(&mut self, method: VectorStorageMethod) {
        self.vector_storage_method = method;
    }
    
    /// 獲取當前向量存儲方法
    #[allow(dead_code)]  // 在某些部署配置中可能未使用，保留以供將來擴展
    pub fn get_vector_storage_method(&self) -> &VectorStorageMethod {
        &self.vector_storage_method
    }
    
    // 從API獲取嵌入向量的函數
    #[allow(dead_code)]
    async fn get_embedding_from_api(
        &self, 
        text: &str, 
        api_manager: &ApiManager,
        guild_id: u64,
    ) -> Result<Vec<f32>> {
        // 獲取該 guild 的 API 配置
        let api_config = api_manager.get_guild_config(guild_id).await;
        
        // 使用 OpenAI 的 embedding 模型
        let embedding_model = "text-embedding-3-small"; // 或 text-embedding-ada-002
        
        // 獲取 API key
        let api_key = api_config.api_key.clone()
            .or_else(|| crate::utils::api::get_api_key_from_env(&api_config.provider));
        
        // 調用 embedding API
        let embeddings = crate::utils::api::call_embedding_api(
            &api_config.api_url,
            api_key.as_deref(),
            &[text.to_string()],
            embedding_model,
            &api_config.provider,
            true, // 使用快取
        )
        .await
        .map_err(|e| anyhow::anyhow!("調用 embedding API 失敗: {}", e))?;
        
        embeddings.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("未獲取到 embedding 結果"))
    }
    
    /// 計算記憶重要性分數 (0.0 - 1.0)
    #[allow(dead_code)]
    pub fn calculate_importance(&self, content: &str, content_type: &str, metadata: &ImportanceMetadata) -> f32 {
        let mut score = 0.0;
        
        // 1. 內容類型基礎分數
        score += match content_type {
            "summary" => 0.9,      // 摘要很重要
            "setting" => 0.8,      // 設定重要
            "decision" => 0.7,     // 決策重要
            "event" => 0.6,        // 事件中等重要
            "message" => 0.3,      // 普通訊息較不重要
            _ => 0.5,              // 預設中等
        };
        
        // 2. 內容長度 (更長的內容可能更重要)
        let content_length = content.chars().count();
        if content_length > 200 {
            score += 0.1;
        }
        if content_length > 500 {
            score += 0.1;
        }
        
        // 3. 關鍵詞匹配
        let keywords = vec![
            "重要", "關鍵", "決定", "規則", "設定", "任務", "目標", 
            "NPC", "BOSS", "寶物", "線索", "劇情", "死亡", "失敗"
        ];
        for keyword in keywords {
            if content.contains(keyword) {
                score += 0.05;
            }
        }
        
        // 4. 提及次數 (如果有人回應這條訊息)
        if let Some(mentions) = metadata.mention_count {
            score += (mentions as f32 * 0.02).min(0.2);
        }
        
        // 5. 反應數量
        if let Some(reactions) = metadata.reaction_count {
            score += (reactions as f32 * 0.01).min(0.1);
        }
        
        // 6. 是否包含骰子結果
        if content.contains("d20") || content.contains("d100") || content.contains("擲骰") {
            score += 0.05;
        }
        
        // 7. 是否有引用其他訊息 (表示延續性)
        if metadata.has_reference {
            score += 0.05;
        }
        
        // 確保分數在 0.0 - 1.0 範圍內
        score.clamp(0.0, 1.0)
    }
    
    /// 自動生成標籤
    #[allow(dead_code)]
    pub fn auto_generate_tags(&self, content: &str, content_type: &str) -> Vec<String> {
        let mut tags = vec![content_type.to_string()];
        
        // 骰子相關
        if content.contains("d20") || content.contains("d100") {
            tags.push("骰子".to_string());
        }
        
        // 戰鬥相關
        if content.contains("攻擊") || content.contains("傷害") || content.contains("HP") {
            tags.push("戰鬥".to_string());
        }
        
        // 角色相關
        if content.contains("角色") || content.contains("技能") || content.contains("屬性") {
            tags.push("角色".to_string());
        }
        
        // 劇情相關
        if content.contains("劇情") || content.contains("NPC") || content.contains("任務") {
            tags.push("劇情".to_string());
        }
        
        // 規則相關
        if content.contains("規則") || content.contains("判定") || content.contains("檢定") {
            tags.push("規則".to_string());
        }
        
        tags
    }
    
    /// 計算記憶衰減因子 (基於時間)
    #[allow(dead_code)]
    pub fn calculate_decay_factor(&self, created_timestamp: u64) -> f32 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        let age_in_days = (now - created_timestamp) as f32 / 86400.0;
        
        // 使用指數衰減: factor = e^(-λt)
        // λ = 0.01 表示約 69 天後重要性減半
        let lambda = 0.01;
        (-lambda * age_in_days).exp()
    }
}



// 簡單的文本標記化函數
fn simple_tokenize(text: &str) -> Vec<String> {
    // 轉換為小寫並分割文本
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

// 將字符串哈希為f32值的輔助函數
fn hash_str_to_f32(s: &str) -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();
    
    // 將哈希值轉換為-1到1之間的f32
    let hash_u32 = hash as u32;
    (hash_u32 as f32) / (u32::MAX as f32) * 2.0 - 1.0
}

// 將最大結果數轉換為 i32
fn max_results_to_i32(max_results: usize) -> i32 {
    max_results as i32
}

// 轉義 SQL 字符串，防止注入
#[allow(dead_code)]  // 在當前實現中暫時未使用，保留以備將來擴充
fn escape_sql_string(input: &str) -> String {
    input.replace("'", "''")
}

// 處理行結果的輔助函數
fn process_row_result(row: &rusqlite::Row) -> std::result::Result<MemoryEntry, rusqlite::Error> {
    let id = row.get(0)?;
    let user_id = row.get(1)?;
    let guild_id = row.get(2)?;
    let channel_id = row.get(3)?;
    let content = row.get(4)?;
    let content_type = row.get(5)?;
    let importance_score = row.get(6)?;
    let tags = row.get(7)?;
    let enabled_i32: i32 = row.get(8)?;
    let enabled = enabled_i32 != 0;
    let created_at: String = row.get(9)?;
    let last_accessed: String = row.get(10)?;
    
    // 檢查並獲得嵌入向量
    let embedding_bytes: Vec<u8> = row.get(11)?;
    let embedding_result = deserialize_embedding(&embedding_bytes);
    
    let embedding_vector = embedding_result.unwrap_or_default();

    Ok(MemoryEntry {
        id,
        user_id,
        guild_id,
        channel_id,
        content,
        content_type,
        importance_score,
        tags,
        enabled,
        created_at,
        last_accessed,
        embedding_vector,
    })
}

fn get_current_timestamp() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_seconds = since_the_epoch.as_secs();
    format!("{}", in_seconds)
}

// 定義對話消息結構
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChatMessage {
    pub user_id: String,
    pub message: String,
    pub timestamp: String,
    // 添加 content 字段(與 message 相同)
    pub content: String,
    // 添加 username 字段
    pub username: String,
}