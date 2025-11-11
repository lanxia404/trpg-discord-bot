use crate::bot::{Context, Error};
use crate::utils::memory::{MemoryEntry, SearchOptions};
use chrono;
use poise::serenity_prelude as serenity;
use serenity::UserId;

/// 記憶功能整合指令
#[poise::command(prefix_command, slash_command)]
#[allow(clippy::too_many_arguments)]
pub async fn memory(
    ctx: Context<'_>,
    #[description = "選擇記憶操作"] action: MemoryAction,
    #[description = "內容"] content: Option<String>,
    #[description = "標籤"] tags: Option<String>,
    #[description = "搜尋查詢"] query: Option<String>,
    #[description = "最大結果數量"] max_results: Option<i32>,
    #[description = "頁碼"] page: Option<i32>,
    #[description = "記憶ID"] id: Option<i32>,
    #[description = "確認清除"] confirm: Option<String>,
    #[description = "啟用或禁用"] enabled: Option<bool>,
) -> Result<(), Error> {
    match action {
        MemoryAction::Save => {
            // 檢查必要參數
            let content = match content {
                Some(c) => c,
                None => {
                    ctx.say("請提供要保存的內容。").await?;
                    return Ok(());
                }
            };

            let guild_id = ctx
                .guild_id()
                .map(|id| id.get().to_string())
                .unwrap_or_else(|| "dm".to_string());
            let channel_id = ctx.channel_id().get().to_string();
            let user_id = ctx.author().id.get().to_string();

            // 檢查記憶功能是否已啟用
            let memory_enabled = {
                let config = ctx.data().config.lock().await;
                config
                    .get_memory_enabled_for_user(&user_id, &guild_id)
                    .await
            };

            if !memory_enabled {
                ctx.say("記憶功能對您已被禁用。請聯繫管理員啟用。").await?;
                return Ok(());
            }

            let memory_entry = MemoryEntry {
                id: 0, // ID 將由數據庫自動生成
                user_id: user_id.clone(),
                guild_id: guild_id.clone(),
                channel_id: channel_id.clone(),
                content: content.clone(),
                content_type: "message".to_string(),
                importance_score: 0.0,
                tags: tags.unwrap_or_default(),
                enabled: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                last_accessed: chrono::Utc::now().to_rfc3339(),
                embedding_vector: None,
            };

            let memory_manager = &ctx.data().memory_manager;
            let entry_id = memory_manager.save_memory(memory_entry).await?;

            ctx.say(format!("記憶已保存！ID: {}", entry_id)).await?;
        }
        MemoryAction::Search => {
            // 檢查必要參數
            let query = match query {
                Some(q) => q,
                None => {
                    ctx.say("請提供搜尋查詢。").await?;
                    return Ok(());
                }
            };

            let guild_id = ctx
                .guild_id()
                .map(|id| id.get().to_string())
                .unwrap_or_else(|| "dm".to_string());
            let user_id = ctx.author().id.get().to_string();

            // 檢查記憶功能是否已啟用
            let memory_enabled = {
                let config = ctx.data().config.lock().await;
                config
                    .get_memory_enabled_for_user(&user_id, &guild_id)
                    .await
            };

            if !memory_enabled {
                ctx.say("記憶功能對您已被禁用。請聯繫管理員啟用。").await?;
                return Ok(());
            }

            let max_results = max_results.unwrap_or(5).clamp(1, 20) as usize;
            let options = SearchOptions {
                max_results,
                guild_id: Some(guild_id),
                user_id: Some(user_id.clone()),
                tags: None,
            };

            let memory_manager = &ctx.data().memory_manager;
            let results = memory_manager.search_memory(&query, &options).await?;

            if results.is_empty() {
                ctx.say("未找到相關記憶。").await?;
                return Ok(());
            }

            let mut response = format!("找到 {} 個相關記憶：\n", results.len());
            for entry in results {
                response.push_str(&format!(
                    "**ID:** {} | **內容:** {} | **時間:** {}\n",
                    entry.id,
                    entry.content.chars().take(100).collect::<String>(),
                    entry.created_at
                ));
            }

            ctx.say(response).await?;
        }
        MemoryAction::List => {
            let guild_id = ctx
                .guild_id()
                .map(|id| id.get().to_string())
                .unwrap_or_else(|| "dm".to_string());
            let user_id = ctx.author().id.get().to_string();

            // 檢查記憶功能是否已啟用
            let memory_enabled = {
                let config = ctx.data().config.lock().await;
                config
                    .get_memory_enabled_for_user(&user_id, &guild_id)
                    .await
            };

            if !memory_enabled {
                ctx.say("記憶功能對您已被禁用。請聯繫管理員啟用。").await?;
                return Ok(());
            }

            let page = page.unwrap_or(1).max(1);
            let page_size = 10;
            let offset = (page - 1) * page_size;

            let memory_manager = &ctx.data().memory_manager;
            let results = memory_manager
                .list_memory(&user_id, &guild_id, offset, page_size)
                .await?;

            if results.is_empty() {
                ctx.say("您沒有任何記憶記錄。").await?;
                return Ok(());
            }

            let mut response = format!("您的第 {} 頁記憶（共 {} 條）：\n", page, results.len());
            for entry in &results {
                response.push_str(&format!(
                    "**ID:** {} | **內容:** {} | **標籤:** {}\n",
                    entry.id,
                    entry.content.chars().take(80).collect::<String>(),
                    entry.tags
                ));
            }

            // 添加分頁提示
            if results.len() as i32 == page_size {
                response.push_str(&format!(
                    "\n要查看下一頁，請使用 `/memory list page:{}`",
                    page + 1
                ));
            }

            ctx.say(response).await?;
        }
        MemoryAction::Delete => {
            // 檢查必要參數
            let id = match id {
                Some(i) => i,
                None => {
                    ctx.say("請提供要刪除的記憶ID。").await?;
                    return Ok(());
                }
            };

            let user_id = ctx.author().id.get().to_string();
            let guild_id = ctx
                .guild_id()
                .map(|id| id.get().to_string())
                .unwrap_or_else(|| "dm".to_string());

            // 檢查記憶功能是否已啟用
            let memory_enabled = {
                let config = ctx.data().config.lock().await;
                config
                    .get_memory_enabled_for_user(&user_id, &guild_id)
                    .await
            };

            if !memory_enabled {
                ctx.say("記憶功能對您已被禁用。請聯繫管理員啟用。").await?;
                return Ok(());
            }

            let memory_manager = &ctx.data().memory_manager;
            let deleted = memory_manager
                .delete_memory(id, &user_id, &guild_id)
                .await?;

            if deleted {
                ctx.say(format!("記憶 ID {} 已被刪除。", id)).await?;
            } else {
                ctx.say(format!("找不到 ID 為 {} 的記憶，或您沒有權限刪除它。", id))
                    .await?;
            }
        }
        MemoryAction::Clear => {
            let user_id = ctx.author().id.get().to_string();
            let guild_id = ctx
                .guild_id()
                .map(|id| id.get().to_string())
                .unwrap_or_else(|| "dm".to_string());

            // 檢查記憶功能是否已啟用
            let memory_enabled = {
                let config = ctx.data().config.lock().await;
                config
                    .get_memory_enabled_for_user(&user_id, &guild_id)
                    .await
            };

            if !memory_enabled {
                ctx.say("記憶功能對您已被禁用。請聯繫管理員啟用。").await?;
                return Ok(());
            }

            if confirm.as_deref() != Some("confirm") {
                ctx.say("請輸入 `/memory clear confirm` 以確認清除所有記憶。此操作不可逆！")
                    .await?;
                return Ok(());
            }

            let memory_manager = &ctx.data().memory_manager;
            let count = memory_manager.clear_memory(&user_id, &guild_id).await?;

            ctx.say(format!("已清除 {} 條記憶。", count)).await?;
        }
        MemoryAction::Toggle => {
            // 檢查必要參數
            let enabled = match enabled {
                Some(e) => e,
                None => {
                    ctx.say("請提供要設定的狀態（true/false）。").await?;
                    return Ok(());
                }
            };

            let user_id = ctx.author().id.get().to_string();
            let guild_id = ctx
                .guild_id()
                .map(|id| id.get().to_string())
                .unwrap_or_else(|| "dm".to_string());

            // 管理員才能為其他用戶切換功能
            let is_admin = is_user_admin(ctx, ctx.author().id).await?;
            if ctx.author().id.get() != user_id.parse().unwrap_or(0) && !is_admin {
                ctx.say("您沒有權限為其他用戶切換記憶功能。").await?;
                return Ok(());
            }

            {
                let config = ctx.data().config.lock().await;
                config
                    .set_memory_enabled_for_user(&user_id, &guild_id, enabled)
                    .await;
                // 保存配置到文件
                config.save_config().await?;
            }

            if enabled {
                ctx.say("記憶功能已啟用！").await?;
            } else {
                ctx.say("記憶功能已禁用。").await?;
            }
        }
        MemoryAction::SetVectorMethod => {
            // 檢查必要參數
            let enabled = match enabled {
                Some(e) => e,
                None => {
                    ctx.say("請提供要設定的向量存儲方法（true表示EmbeddingApi，false表示Local）。")
                        .await?;
                    return Ok(());
                }
            };

            let user_id = ctx.author().id.get().to_string();
            let guild_id = ctx
                .guild_id()
                .map(|id| id.get().to_string())
                .unwrap_or_else(|| "dm".to_string());

            // 管理員才能為其他用戶切換功能
            let is_admin = is_user_admin(ctx, ctx.author().id).await?;
            if ctx.author().id.get() != user_id.parse().unwrap_or(0) && !is_admin {
                ctx.say("您沒有權限為其他用戶切換向量存儲方法。").await?;
                return Ok(());
            }

            // 根據用戶選擇設置向量存儲方法
            use crate::models::types::VectorStorageMethod;
            let vector_method = if enabled {
                VectorStorageMethod::EmbeddingApi
            } else {
                VectorStorageMethod::Local
            };

            // 更新配置中的向量存儲方法
            {
                let config = ctx.data().config.lock().await;
                let current_guild_config =
                    config.get_guild_config(guild_id.parse().unwrap_or(0)).await;
                let mut new_guild_config = current_guild_config.clone();
                new_guild_config.memory_vector_storage_method = vector_method;
                config
                    .set_guild_config(guild_id.parse().unwrap_or(0), new_guild_config)
                    .await?;
            }

            // 保存配置到文件
            {
                let config_ref = ctx.data().config.lock().await;
                config_ref.save_config().await?;
            }

            if enabled {
                ctx.say("向量存儲方法已設定為使用嵌入API！").await?;
            } else {
                ctx.say("向量存儲方法已設定為本地計算。").await?;
            }
        }
    }

    Ok(())
}

// 定義記憶操作枚舉
#[derive(Debug, poise::ChoiceParameter)]
pub enum MemoryAction {
    #[name = "保存記憶"]
    Save,
    #[name = "搜尋記憶"]
    Search,
    #[name = "列出記憶"]
    List,
    #[name = "刪除記憶"]
    Delete,
    #[name = "清除記憶"]
    Clear,
    #[name = "開關功能"]
    Toggle,
    #[name = "設置向量計算方式"]
    SetVectorMethod,
}

// 檢查用戶是否為管理員的輔助函數
async fn is_user_admin(ctx: Context<'_>, user_id: UserId) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        if let Ok(member) = guild_id.member(&ctx.discord(), user_id).await {
            return Ok(member
                .permissions(ctx.discord())
                .map(|perms| perms.administrator())
                .unwrap_or(false));
        }
    }
    // 在 DM 中，假設機器人擁有者是管理員
    Ok(ctx.framework().bot_id.get() == ctx.author().id.get())
}
