mod bot;
mod models;
mod utils;

use std::env;
use std::sync::Arc;

use anyhow::anyhow;
use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;

use crate::bot::data::BotData;
use crate::utils::config::ConfigManager;

use crate::utils::memory::MemoryManager;

#[tokio::main]
async fn main() -> Result<(), bot::Error> {
    if let Err(e) = utils::logger::DiscordLogger::init(Some("bot.log")) {
        eprintln!("日誌初始化失敗: {}", e);
    }

    dotenvy::dotenv().ok();

    // 啟動 .env 熱載入監視器
    let _env_watcher = utils::env_watcher::EnvWatcher::new(".env")
        .map_err(|e| anyhow!("環境變數監視器初始化失敗: {}", e))?;

    let token =
        env::var("DISCORD_TOKEN").map_err(|_| anyhow!("預期 DISCORD_TOKEN 環境變數，但找不到!"))?;

    let config_manager = ConfigManager::new("config.json")
        .await
        .map_err(|e| anyhow!("設定管理器初始化失敗: {}", e))?;
    let shared_config = Arc::new(Mutex::new(config_manager));
    // 下面開始建立並初始化資料庫
    let skills_db = tokio_rusqlite::Connection::open("skills.db")
        .await
        .map_err(|e| anyhow!("開啟技能資料庫失敗: {}", e))?;
    skills_db
        .call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS skills (
                    guild_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    normalized_name TEXT NOT NULL,
                    skill_type TEXT NOT NULL,
                    level TEXT NOT NULL,
                    effect TEXT NOT NULL,
                    occupation TEXT DEFAULT '',
                    race TEXT DEFAULT '',
                    UNIQUE(guild_id, normalized_name)
                )",
                [],
            )?;

            Ok(())
        })
        .await
        .map_err(|e| anyhow!("初始化技能資料庫失敗: {}", e))?;

    let base_settings_db = tokio_rusqlite::Connection::open("base_settings.db")
        .await
        .map_err(|e| anyhow!("開啟基本設定資料庫失敗: {}", e))?;
    // base_settings.db 現在用於存儲導入的數據表，無需預設表結構
    // 導入功能將根據數據類型自動創建對應的表
    base_settings_db
        .call(|conn| {
            // 確保資料庫連接正常
            conn.execute("CREATE TABLE IF NOT EXISTS __temp_check (id INTEGER)", [])
                .map_err(tokio_rusqlite::Error::Rusqlite)?;
            conn.execute("DROP TABLE IF EXISTS __temp_check", [])
                .map_err(tokio_rusqlite::Error::Rusqlite)?;
            Ok(())
        })
        .await
        .map_err(|e| anyhow!("初始化基本設定資料庫失敗: {}", e))?;

    // 初始化記憶管理器（先用 None 和默認本地存儲方法，稍後再設置）
    use crate::models::types::VectorStorageMethod;
    let memory_manager = MemoryManager::new("memory.db", None, VectorStorageMethod::Local)
        .await
        .map_err(|e| anyhow!("記憶管理器初始化失敗: {}", e))?;
    let shared_memory_manager = Arc::new(memory_manager);
    let _setup_memory_manager = Arc::clone(&shared_memory_manager);
    let shared_chat_history_manager = Arc::clone(&shared_memory_manager);
    let _setup_chat_history_manager = Arc::clone(&shared_chat_history_manager);

    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MESSAGES;

    let api_manager = Arc::new(crate::utils::api::ApiManager::new(Arc::clone(
        &shared_config,
    )));
    let shared_api_manager = Arc::clone(&api_manager);
    let setup_config = Arc::clone(&shared_config);
    // 現在有了 api_manager，我們可以重新初始化記憶管理器以包含 api_manager
    let memory_manager = MemoryManager::new(
        "memory.db",
        Some(shared_api_manager.clone()),
        crate::models::types::VectorStorageMethod::Local,
    )
    .await
    .map_err(|e| anyhow!("記憶管理器初始化失敗: {}", e))?;
    let shared_memory_manager = Arc::new(memory_manager);
    let _setup_memory_manager = Arc::clone(&shared_memory_manager);
    let shared_chat_history_manager = Arc::clone(&shared_memory_manager);
    let _setup_chat_history_manager = Arc::clone(&shared_chat_history_manager);

    let setup_skills_db = skills_db.clone();
    let setup_base_settings_db = base_settings_db.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: crate::bot::commands(),
            on_error: |error| {
                Box::pin(async move {
                    log::error!("指令執行錯誤: {}", error);

                    // 嘗試獲取具體的錯誤資訊
                    let error_msg = format!("發生錯誤: {}", error);

                    // 如果有互動回應，向使用者發送錯誤訊息
                    if let poise::FrameworkError::Command { ctx, .. } = error {
                        if let Err(why) = ctx.say(error_msg).await {
                            log::error!("發送錯誤訊息失敗: {}", why);
                        }
                    }
                })
            },
            event_handler: |_ctx, event, _framework, _data| {
                Box::pin(async move {
                    // 在poise中，事件類型是FullEvent，需要使用適當的方法來獲取消息
                    use poise::serenity_prelude::FullEvent;

                    if let FullEvent::Message {
                        new_message: message,
                    } = event
                    {
                        // 只檢查是否標記機器人
                        let is_mentioned = message
                            .mentions
                            .iter()
                            .any(|user| user.id == _ctx.cache.current_user().id);

                        // 只在被提及時記錄日誌
                        if is_mentioned {
                            log::info!(
                                "訊息事件處理: is_mentioned=true, content='{}'",
                                message.content
                            );
                        }

                        // 如果被標記，並且該頻道尚未載入初始歷史，則載入少量歷史消息
                        if is_mentioned {
                            let channel_id = message.channel_id.get();
                            let mut loaded_channels = _data.initial_history_loaded.lock().await;

                            if !loaded_channels.contains(&channel_id) {
                                loaded_channels.insert(channel_id);
                                drop(loaded_channels); // 釋放鎖定，以免在 await 時保持鎖定

                                log::info!("載入頻道 {} 的初始歷史消息", channel_id);
                                // 獲取當前頻道的較多歷史消息（例如最近的 50 條）
                                match message
                                    .channel_id
                                    .messages(&_ctx.http, serenity::GetMessages::new().limit(50))
                                    .await
                                {
                                    Ok(history_messages) => {
                                        log::info!(
                                            "從頻道 {} 載入了 {} 條歷史消息",
                                            channel_id,
                                            history_messages.len()
                                        );

                                        for history_message in history_messages {
                                            // 只記錄非機器人的消息
                                            if history_message.author.id
                                                != _ctx.cache.current_user().id
                                            {
                                                if let Err(e) = _data
                                                    .memory_manager
                                                    .insert_message(
                                                        channel_id,
                                                        message.guild_id.map(|g| g.get()),
                                                        history_message.author.id.get(),
                                                        &history_message.author.name,
                                                        &history_message.content,
                                                    )
                                                    .await
                                                {
                                                    log::error!("記錄歷史消息失敗: {}", e);
                                                } else {
                                                    log::debug!(
                                                        "記錄歷史消息: {} - {}",
                                                        history_message.author.name,
                                                        history_message.content
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "獲取頻道 {} 的歷史消息失敗: {}",
                                            channel_id,
                                            e
                                        );
                                    }
                                }
                            } else {
                                // 釋放鎖定後再繼續
                                drop(loaded_channels);
                            }
                        }

                        // 記錄所有用戶消息到對話歷史（除了機器人自己的消息）
                        if message.author.id != _ctx.cache.current_user().id {
                            let channel_id = message.channel_id.get();
                            let guild_id = message.guild_id.map(|g| g.get());
                            let user_id = message.author.id.get();
                            let username = &message.author.name;
                            let content = &message.content;

                            if let Err(e) = _data
                                .memory_manager
                                .insert_message(channel_id, guild_id, user_id, username, content)
                                .await
                            {
                                log::error!("記錄對話歷史失敗: {}", e);
                            }
                        }

                        if is_mentioned {
                            // 處理與AI的對話
                            log::info!("觸發AI對話處理");
                            handle_message(_ctx, message, _data).await;
                        }
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let config = Arc::clone(&setup_config);
            let api_manager = Arc::clone(&shared_api_manager);
            let memory_manager = _setup_memory_manager;
            let _chat_history_manager = _setup_chat_history_manager; // 對於兼容性，保留此變量但標記為未使用
            let skills_db = setup_skills_db.clone();
            let base_settings_db = setup_base_settings_db.clone();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // 初始化 ConversationManager
                let conversation_manager =
                    Arc::new(crate::utils::conversation::ConversationManager::new(
                        memory_manager.clone(),
                        config.clone(),
                        api_manager.clone(),
                    ));

                println!("{} 已經上線!", ready.user.name);
                Ok(BotData {
                    config,
                    api_manager,
                    memory_manager,
                    conversation_manager,
                    initial_history_loaded: Arc::new(Mutex::new(std::collections::HashSet::new())),
                    skills_db,
                    base_settings_db,
                })
            })
        })
        .build();

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .await
        .map_err(|e| anyhow!("建立 Discord 客戶端失敗: {}", e))?;

    client
        .start()
        .await
        .map_err(|e| anyhow!("機器人啟動失敗: {}", e))?;

    Ok(())
}

async fn handle_message(ctx: &serenity::Context, msg: &serenity::Message, data: &BotData) {
    // 檢查此頻道是否在伺服器中（不處理私訊）
    if msg.guild_id.is_none() {
        if let Err(e) = msg
            .channel_id
            .say(&ctx.http, "抱歉，AI對話功能僅在伺服器中可用。")
            .await
        {
            log::error!("發送訊息失敗: {:?}", e);
        }
        return;
    }

    let guild_id = msg.guild_id.unwrap().get();
    let channel_id = msg.channel_id.get();
    let user_id = msg.author.id.get();

    log::info!(
        "收到訊息，Guild ID: {}, Channel ID: {}, Author: {}",
        guild_id,
        channel_id,
        msg.author.name
    );

    // 獲取該伺服器的API配置
    let api_config = data.api_manager.get_guild_config(guild_id).await;
    log::info!(
        "API Config for guild {}: enabled={}, has_api_key={}, provider={:?}",
        guild_id,
        api_config.enabled,
        api_config.api_key.is_some(),
        api_config.provider
    );

    if !api_config.enabled {
        log::info!("伺服器 {} 的AI功能未啟用", guild_id);
        if let Err(e) = msg
            .channel_id
            .say(
                &ctx.http,
                "此伺服器尚未啟用AI對話功能。請使用 `/chat add` 指令設定API。",
            )
            .await
        {
            log::error!("發送訊息失敗: {:?}", e);
        }
        return;
    }

    // 準備用戶消息內容
    let mut user_message = remove_bot_mention(&msg.content, ctx.cache.current_user().id);

    // 如果當前消息是對其他消息的回覆，將被回覆的消息內容加入上下文
    if let Some(referenced) = &msg.referenced_message {
        let replied_context = format!(
            "[回覆 {}: {}]\n{}",
            referenced.author.name, referenced.content, user_message
        );
        user_message = replied_context;
    }

    // 使用 ConversationManager 構建對話上下文
    let conversation_context = match data
        .conversation_manager
        .build_context(
            guild_id,
            channel_id,
            user_id,
            &user_message,
            crate::utils::conversation::ContextStrategy::Hybrid,
        )
        .await
    {
        Ok(ctx) => ctx,
        Err(e) => {
            log::error!("構建對話上下文失敗: {}", e);
            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, format!("處理對話時發生錯誤: {}", e))
                .await
            {
                log::error!("發送錯誤訊息失敗: {:?}", e);
            }
            return;
        }
    };

    log::info!(
        "對話上下文已構建: messages={}, tokens={}, memories={}",
        conversation_context.messages.len(),
        conversation_context.total_tokens,
        conversation_context.retrieved_memories.len()
    );

    // 將 ConversationContext 轉換為 API 請求格式
    let api_messages: Vec<crate::utils::api::ChatMessage> = conversation_context
        .messages
        .iter()
        .map(|msg| crate::utils::api::ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        })
        .collect();

    // 優先使用配置中的 API 金鑰，如果沒有則嘗試從環境變數獲取
    let effective_api_key = api_config
        .api_key
        .clone()
        .or_else(|| crate::utils::api::get_api_key_from_env(&api_config.provider));

    log::info!(
        "嘗試從環境變數獲取API金鑰，provider={:?}",
        api_config.provider
    );

    // 檢查是否找到有效的 API 金鑰
    if effective_api_key.is_none() {
        log::warn!("伺服器 {} 沒有有效的API金鑰", guild_id);
        if let Err(e) = msg
            .channel_id
            .say(
                &ctx.http,
                "錯誤：未找到 API 金鑰。請確保已在 .env 文件中設置相應的 API 金鑰環境變數。",
            )
            .await
        {
            log::error!("發送錯誤訊息失敗: {:?}", e);
        }
        return;
    } else {
        log::info!("成功獲取API金鑰，準備調用API");
    }

    // 創建對話請求
    let request = crate::utils::api::ChatCompletionRequest {
        model: api_config.model.clone(),
        messages: api_messages,
        temperature: Some(0.7),
        max_tokens: Some(1024),
    };

    log::info!(
        "API請求準備就緒: model={}, messages={}, tokens={}",
        api_config.model,
        request.messages.len(),
        conversation_context.total_tokens
    );

    // 發送 typing 指示器
    let _typing = msg.channel_id.start_typing(&ctx.http);
    log::info!("已開始顯示 typing 指示器");

    // 調用API
    log::info!(
        "正在調用API: URL={}, Provider={:?}",
        api_config.api_url,
        api_config.provider
    );
    match crate::utils::api::call_llm_api(
        &api_config.api_url,
        effective_api_key.as_deref(),
        &request,
        &api_config.provider,
    )
    .await
    {
        Ok(response) => {
            log::info!(
                "API回應成功，字節長度: {}, 字符長度: {}",
                response.len(),
                response.chars().count()
            );

            // 限制 AI 回應在 1000 中文字符內
            let limited_response = limit_chinese_chars(&response, 1000);

            log::info!("限制後的回應字符長度: {}", limited_response.chars().count());

            // 分割回應以防超出Discord字符限制
            const MAX_MESSAGE_LENGTH: usize = 2000;
            // 按字符進行分割
            let chars: Vec<char> = limited_response.chars().collect();
            let chunks: Vec<String> = chars
                .chunks(MAX_MESSAGE_LENGTH)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect();

            log::info!("回應分割為 {} 個部分", chunks.len());
            // 將機器人回應拼接起來以便記錄到歷史
            let full_bot_response = chunks.join("");

            // 在 await 之前先獲取機器人用戶資訊
            let channel_id = msg.channel_id.get();
            let guild_id = msg.guild_id.map(|g| g.get());
            let bot_user_id = ctx.cache.current_user().id.get();
            let bot_username = ctx.cache.current_user().name.clone();

            for (i, chunk) in chunks.iter().enumerate() {
                log::info!("發送回應部分 {}: 字符長度 {}", i + 1, chunk.chars().count());
                if let Err(e) = msg.channel_id.say(&ctx.http, chunk).await {
                    log::error!("發送訊息失敗: {:?}", e);
                }
            }

            // 記錄機器人的回應到對話歷史
            if let Err(e) = data
                .memory_manager
                .insert_message(
                    channel_id,
                    guild_id,
                    bot_user_id,
                    &bot_username,
                    &full_bot_response,
                )
                .await
            {
                log::error!("記錄機器人對話歷史失敗: {}", e);
            } else {
                log::debug!("記錄機器人回應: {}", full_bot_response);
            }
        }
        Err(e) => {
            log::error!("API調用失敗: {:?}", e);
            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, format!("API調用失敗: {:?}", e))
                .await
            {
                log::error!("發送錯誤訊息失敗: {:?}", e);
            }
        }
    }
}

// 判斷字符是否為中文字符
fn is_chinese_char(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c) ||  // CJK統一表意文字
    ('\u{3400}'..='\u{4dbf}').contains(&c) ||  // CJK擴展A區
    ('\u{20000}'..='\u{2a6df}').contains(&c) || // CJK擴展B區
    ('\u{2a700}'..='\u{2b73f}').contains(&c) || // CJK擴展C區
    ('\u{2b740}'..='\u{2b81f}').contains(&c) || // CJK擴展D區
    ('\u{2b820}'..='\u{2ceaf}').contains(&c) || // CJK擴展E區
    ('\u{f900}'..='\u{faff}').contains(&c) // CJK相容表意文字
}

// 限制字符串中的中文字符數量
fn limit_chinese_chars(s: &str, max_count: usize) -> String {
    let mut result = String::new();
    let mut chinese_count = 0;

    for c in s.chars() {
        if is_chinese_char(c) {
            if chinese_count >= max_count {
                break;
            }
            chinese_count += 1;
        }
        result.push(c);
    }

    result
}

fn remove_bot_mention(content: &str, bot_id: serenity::UserId) -> String {
    let bot_mention = format!("<@{}>", bot_id);
    let bot_mention_nick = format!("<@!{}>", bot_id); // With nickname
    content
        .replace(&bot_mention, "")
        .replace(&bot_mention_nick, "")
        .trim_start()
        .to_string()
}
