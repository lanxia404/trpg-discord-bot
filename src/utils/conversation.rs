use anyhow::Result;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::utils::api::ApiManager;
use crate::utils::config::ConfigManager;
use crate::utils::memory::MemoryManager;

/// 對話上下文構建策略
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ContextStrategy {
    /// 最近訊息優先
    RecentFirst,
    /// 重要性優先
    ImportanceFirst,
    /// 混合策略 (最近 + 重要)
    Hybrid,
}

/// 對話訊息結構
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConversationMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub timestamp: Option<String>,
    pub importance: f32,
}

/// 對話上下文
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConversationContext {
    pub system_prompt: String,
    pub messages: Vec<ConversationMessage>,
    pub total_tokens: usize,
    pub retrieved_memories: Vec<String>,
}

/// 對話管理器 - 核心組件
#[allow(dead_code)]
#[derive(Debug)]
pub struct ConversationManager {
    memory_manager: Arc<MemoryManager>,
    config: Arc<Mutex<ConfigManager>>,
    api_manager: Arc<ApiManager>,
}

#[allow(dead_code)]
impl ConversationManager {
    pub fn new(
        memory_manager: Arc<MemoryManager>,
        config: Arc<Mutex<ConfigManager>>,
        api_manager: Arc<ApiManager>,
    ) -> Self {
        Self {
            memory_manager,
            config,
            api_manager,
        }
    }

    /// 構建完整的對話上下文
    pub async fn build_context(
        &self,
        guild_id: u64,
        channel_id: u64,
        user_id: u64,
        user_message: &str,
        strategy: ContextStrategy,
    ) -> Result<ConversationContext> {
        // 1. 獲取 API 配置以確定模型的上下文窗口
        let api_config = self.api_manager.get_guild_config(guild_id).await;
        let max_context_tokens = self.get_model_context_window(&api_config.model);

        // 預留給回應的 token (通常是上下文的 1/4)
        let available_tokens = (max_context_tokens as f32 * 0.75) as usize;

        log::info!(
            "構建對話上下文: guild_id={}, channel_id={}, max_tokens={}, available_tokens={}",
            guild_id,
            channel_id,
            max_context_tokens,
            available_tokens
        );

        // 2. 獲取系統提示詞
        let system_prompt = self.build_system_prompt(guild_id).await?;
        let mut used_tokens = self.estimate_tokens(&system_prompt);

        // 3. 為當前訊息預留空間
        let current_message_tokens = self.estimate_tokens(user_message);
        used_tokens += current_message_tokens;

        // 4. 使用 RAG 檢索相關記憶
        let retrieved_memories = self
            .retrieve_relevant_memories(
                guild_id,
                channel_id,
                user_id,
                user_message,
                (available_tokens - used_tokens) / 4, // 分配 25% 給記憶
            )
            .await?;

        let memories_text = retrieved_memories.join("\n");
        used_tokens += self.estimate_tokens(&memories_text);

        // 5. 獲取對話歷史
        let remaining_tokens = available_tokens.saturating_sub(used_tokens);
        let conversation_history = self
            .get_conversation_history(guild_id, channel_id, remaining_tokens, strategy)
            .await?;

        // 6. 構建最終上下文
        let mut messages = Vec::new();

        // 系統提示詞
        messages.push(ConversationMessage {
            role: "system".to_string(),
            content: system_prompt.clone(),
            timestamp: None,
            importance: 1.0,
        });

        // 添加記憶上下文 (如果有)
        if !retrieved_memories.is_empty() {
            let memory_context = format!("相關記憶與設定:\n{}", retrieved_memories.join("\n---\n"));
            messages.push(ConversationMessage {
                role: "system".to_string(),
                content: memory_context,
                timestamp: None,
                importance: 0.8,
            });
        }

        // 對話歷史
        messages.extend(conversation_history);

        // 當前使用者訊息
        messages.push(ConversationMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            timestamp: Some(Self::get_current_timestamp()),
            importance: 1.0,
        });

        let total_tokens = self.calculate_total_tokens(&messages);

        log::info!(
            "對話上下文構建完成: messages={}, total_tokens={}, memories={}",
            messages.len(),
            total_tokens,
            retrieved_memories.len()
        );

        Ok(ConversationContext {
            system_prompt,
            messages,
            total_tokens,
            retrieved_memories,
        })
    }

    /// 獲取模型的上下文窗口大小
    fn get_model_context_window(&self, model: &str) -> usize {
        match model {
            // OpenAI models
            m if m.contains("gpt-4o") => 128000,
            m if m.contains("gpt-4-turbo") => 128000,
            m if m.contains("gpt-4") => 8192,
            m if m.contains("gpt-3.5-turbo") => 16385,

            // Anthropic models
            m if m.contains("claude-3-opus") => 200000,
            m if m.contains("claude-3-sonnet") => 200000,
            m if m.contains("claude-3-haiku") => 200000,
            m if m.contains("claude-2") => 100000,

            // Google models
            m if m.contains("gemini-pro") => 32768,
            m if m.contains("gemini-1.5") => 1000000,

            // OpenRouter and others - default
            _ => 8192,
        }
    }

    /// 估算文本的 token 數量
    fn estimate_tokens(&self, text: &str) -> usize {
        // 簡化估算:
        // 英文: ~4 字元 = 1 token
        // 中文: ~1.5 字元 = 1 token
        let chinese_chars = text.chars().filter(|c| Self::is_cjk_char(*c)).count();
        let total_chars = text.len();
        let non_chinese_chars = total_chars.saturating_sub(chinese_chars);

        let chinese_tokens = (chinese_chars as f32 / 1.5) as usize;
        let english_tokens = non_chinese_chars / 4;

        chinese_tokens + english_tokens
    }

    /// 判斷是否為 CJK 字元
    fn is_cjk_char(c: char) -> bool {
        matches!(c,
            '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
            '\u{3400}'..='\u{4DBF}' |  // CJK Extension A
            '\u{20000}'..='\u{2A6DF}' | // CJK Extension B
            '\u{2A700}'..='\u{2B73F}' | // CJK Extension C
            '\u{2B740}'..='\u{2B81F}' | // CJK Extension D
            '\u{2B820}'..='\u{2CEAF}' | // CJK Extension E
            '\u{F900}'..='\u{FAFF}'    // CJK Compatibility Ideographs
        )
    }

    /// 計算所有訊息的總 token 數
    fn calculate_total_tokens(&self, messages: &[ConversationMessage]) -> usize {
        messages
            .iter()
            .map(|msg| self.estimate_tokens(&msg.content))
            .sum()
    }

    /// 構建系統提示詞
    async fn build_system_prompt(&self, guild_id: u64) -> Result<String> {
        let config = self.config.lock().await;
        let guild_config = config.get_guild_config(guild_id).await;
        drop(config);

        let mut prompt = String::from(
            "你是一個專業的 TRPG (桌上角色扮演遊戲) 助手。\n\
             你的任務是幫助玩家和 GM (遊戲主持人) 進行遊戲。\n\
             \n\
             重要指引:\n\
             1. 保持角色扮演的氛圍和沉浸感\n\
             2. 提供有用的遊戲建議和規則解釋\n\
             3. 協助推進劇情發展\n\
             4. 記住之前的對話和重要設定\n\
             5. 回應要簡潔明瞭,避免過於冗長\n\
             6. 使用繁體中文回應\n",
        );

        // 添加伺服器特定設定
        if let Some(dnd_rules) = Some(&guild_config.dnd_rules) {
            prompt.push_str(&format!(
                "\n伺服器 D&D 規則:\n\
                 - 大成功: {}\n\
                 - 大失敗: {}\n",
                dnd_rules.critical_success, dnd_rules.critical_fail
            ));
        }

        Ok(prompt)
    }

    /// 使用 RAG 檢索相關記憶
    async fn retrieve_relevant_memories(
        &self,
        guild_id: u64,
        channel_id: u64,
        user_id: u64,
        query: &str,
        max_tokens: usize,
    ) -> Result<Vec<String>> {
        use crate::utils::memory::SearchOptions;

        // 計算可以檢索多少條記憶
        let estimated_tokens_per_memory = 100; // 平均每條記憶約 100 tokens
        let max_results = (max_tokens / estimated_tokens_per_memory).clamp(3, 10);

        let options = SearchOptions {
            max_results,
            guild_id: Some(guild_id.to_string()),
            user_id: Some(user_id.to_string()),
            channel_id: Some(channel_id.to_string()),
            tags: None,
        };

        let memories = self.memory_manager.search_memory(query, &options).await?;

        let mut results = Vec::new();
        let mut total_tokens = 0;

        for memory in memories {
            let memory_text = format!("[{}] {}", memory.content_type, memory.content);
            let tokens = self.estimate_tokens(&memory_text);

            if total_tokens + tokens > max_tokens {
                break;
            }

            results.push(memory_text);
            total_tokens += tokens;
        }

        log::debug!(
            "檢索到 {} 條相關記憶 (共 {} tokens)",
            results.len(),
            total_tokens
        );
        Ok(results)
    }

    /// 獲取對話歷史
    async fn get_conversation_history(
        &self,
        guild_id: u64,
        channel_id: u64,
        max_tokens: usize,
        strategy: ContextStrategy,
    ) -> Result<Vec<ConversationMessage>> {
        // 獲取最近的對話記錄
        let history = self
            .memory_manager
            .get_recent_messages(guild_id, channel_id, 100)
            .await?;

        let mut messages = Vec::new();
        let mut total_tokens = 0;

        // 根據策略選擇訊息
        let sorted_history = match strategy {
            ContextStrategy::RecentFirst => {
                // 最近的訊息優先 (已經是時間倒序)
                history
            }
            ContextStrategy::ImportanceFirst => {
                // 按重要性排序 (需要在記憶中存儲重要性)
                let mut sorted = history;
                sorted.sort_by(|a, b| {
                    // 簡單啟發: 長訊息可能更重要
                    b.content.len().cmp(&a.content.len())
                });
                sorted
            }
            ContextStrategy::Hybrid => {
                // 混合: 保留最近 30% + 最重要 70%
                let recent_count = (history.len() as f32 * 0.3) as usize;
                let mut recent: Vec<_> = history.iter().take(recent_count).cloned().collect();

                let mut remaining: Vec<_> = history.iter().skip(recent_count).cloned().collect();
                remaining.sort_by(|a, b| b.content.len().cmp(&a.content.len()));

                recent.extend(remaining);
                recent
            }
        };

        for msg in sorted_history.iter().rev() {
            // 跳過機器人自己的訊息 (可選)
            // if msg.username.contains("Bot") { continue; }

            let role = if msg.username.contains("Bot") || msg.username == "Assistant" {
                "assistant"
            } else {
                "user"
            };

            let content = format!("{}: {}", msg.username, msg.content);
            let tokens = self.estimate_tokens(&content);

            if total_tokens + tokens > max_tokens {
                break;
            }

            messages.push(ConversationMessage {
                role: role.to_string(),
                content,
                timestamp: Some(msg.timestamp.clone()),
                importance: 0.5,
            });

            total_tokens += tokens;
        }

        log::debug!(
            "載入 {} 條對話歷史 (共 {} tokens)",
            messages.len(),
            total_tokens
        );
        Ok(messages)
    }

    /// 生成對話摘要
    pub async fn summarize_conversation(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_count: usize,
    ) -> Result<String> {
        let history = self
            .memory_manager
            .get_recent_messages(guild_id, channel_id, message_count)
            .await?;

        if history.is_empty() {
            return Ok("沒有對話記錄".to_string());
        }

        // 構建摘要提示
        let conversation_text = history
            .iter()
            .map(|msg| format!("{}: {}", msg.username, msg.content))
            .collect::<Vec<_>>()
            .join("\n");

        let summary_prompt = format!(
            "請總結以下 TRPG 遊戲對話的關鍵要點,包括重要劇情、角色互動和決策:\n\n{}",
            conversation_text
        );

        // 調用 LLM 生成摘要
        let api_config = self.api_manager.get_guild_config(guild_id).await;

        let request = crate::utils::api::ChatCompletionRequest {
            model: api_config.model.clone(),
            messages: vec![crate::utils::api::ChatMessage {
                role: "user".to_string(),
                content: summary_prompt,
            }],
            temperature: Some(0.5),
            max_tokens: Some(500),
        };

        let api_key = api_config
            .api_key
            .clone()
            .or_else(|| crate::utils::api::get_api_key_from_env(&api_config.provider));

        let summary = crate::utils::api::call_llm_api(
            &api_config.api_url,
            api_key.as_deref(),
            &request,
            &api_config.provider,
        )
        .await
        .map_err(|e| anyhow::anyhow!("調用 LLM API 失敗: {}", e))?;

        // 保存摘要為記憶
        let memory_entry = crate::utils::memory::MemoryEntry {
            id: 0,
            user_id: "system".to_string(),
            guild_id: guild_id.to_string(),
            channel_id: channel_id.to_string(),
            content: summary.clone(),
            content_type: "summary".to_string(),
            importance_score: 0.9,
            tags: "對話摘要".to_string(),
            enabled: true,
            created_at: Self::get_current_timestamp(),
            last_accessed: Self::get_current_timestamp(),
            embedding_vector: None,
        };

        self.memory_manager.save_memory(memory_entry).await?;

        Ok(summary)
    }

    fn get_current_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        format!("{}", since_the_epoch.as_secs())
    }
}
