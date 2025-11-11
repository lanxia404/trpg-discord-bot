use crate::bot::{Context, Error};
use crate::utils::api::{ApiConfig, ApiProvider, ChatCompletionRequest, ChatMessage};
use poise::{ChoiceParameter, CreateReply, serenity_prelude as serenity};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;

#[derive(ChoiceParameter, Clone, Copy, Debug)]
pub enum ApiAction {
    #[name = "add"]
    Add,
    #[name = "remove"]
    Remove,
    #[name = "toggle"]
    Toggle,
    #[name = "list-models"]
    ListModels,
    #[name = "list"]
    List,
    #[name = "switch"]
    Switch,
}

/// API 設定指令
#[poise::command(slash_command)]
pub async fn chat(
    ctx: Context<'_>,
    #[description = "操作 add、remove、toggle、list、switch 或 list-models"] action: ApiAction,
    #[description = "API URL"] api_url: Option<String>,
    #[description = "API 金鑰"] api_key: Option<String>,
    #[description = "模型名稱"] model: Option<String>,
    #[description = "API設定名稱"] name: Option<String>,
) -> Result<(), Error> {
    log::info!("執行 API 指令: {:?} for guild {:?}", action, ctx.guild_id());

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            let embed = serenity::CreateEmbed::default()
                .colour(serenity::Colour::RED)
                .description("此指令僅能在伺服器中使用");
            ctx.send(CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let data = ctx.data();
    let api_manager = &data.api_manager;

    match action {
        ApiAction::Add => {
            let api_url = if let Some(url) = api_url {
                url
            } else {
                let embed = serenity::CreateEmbed::default()
                    .colour(serenity::Colour::RED)
                    .description("請提供 API URL");
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            };

            let test_provider = determine_provider_from_url(&api_url);
            // 選擇適合提供者的默認模型
            let default_model = crate::utils::api::get_default_model_for_provider(&test_provider);

            // 優先使用傳入的 API 金鑰，如果沒有則嘗試從環境變數獲取
            let effective_api_key = api_key
                .clone()
                .or_else(|| crate::utils::api::get_api_key_from_env(&test_provider));

            // 驗證 API 連線
            let test_request = ChatCompletionRequest {
                model: model.clone().unwrap_or_else(|| default_model.clone()),
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: "測試".to_string(),
                }],
                temperature: None,
                max_tokens: Some(10),
            };

            // 記錄 API 測試參數，方便調試
            log::info!(
                "API 測試: URL={} Model={} Key(Present)={}",
                api_url,
                model.clone().unwrap_or_else(|| default_model.clone()),
                effective_api_key.is_some()
            );

            let call_result = timeout(
                Duration::from_secs(10),
                crate::utils::api::call_llm_api(
                    &api_url,
                    effective_api_key.as_deref(),
                    &test_request,
                    &test_provider,
                ),
            )
            .await;

            match call_result {
                Ok(Ok(_)) => {
                    // API 連線成功
                    let provider = determine_provider_from_url(&api_url);
                    // 使用測試成功的模型或根據提供者選擇默認模型
                    let selected_model = model.unwrap_or_else(|| {
                        crate::utils::api::get_default_model_for_provider(&provider)
                    });

                    // 檢查是否有通過命令提供金鑰，以及環境變數中是否有金鑰
                    let has_command_key = api_key.is_some();
                    let has_env_key = crate::utils::api::get_api_key_from_env(&provider).is_some();

                    // 如果通過命令提供了金鑰，則將其保存到 .env 文件中
                    if let Some(ref key) = api_key {
                        save_api_key_to_env(&provider, key).await;
                    }

                    // 儲存到設定檔時，不保存金鑰（只保存 None，表示使用環境變數）
                    // 預設使用API URL作為名稱，如果有重複則添加序號
                    let mut api_name = api_url.clone();
                    let all_configs = api_manager.get_guild_configs(guild_id).await;
                    if all_configs.contains_key(&api_name) {
                        let mut counter = 1;
                        while all_configs.contains_key(&format!("{}{}", api_name, counter)) {
                            counter += 1;
                        }
                        api_name = format!("{}{}", api_name, counter);
                    }

                    let api_config = ApiConfig {
                        name: api_name,
                        api_url,
                        api_key: None, // 不再保存金鑰到設定檔中
                        model: selected_model,
                        enabled: true,
                        provider: provider.clone(), // Clone to avoid move
                    };

                    api_manager.add_guild_config(guild_id, api_config).await;

                    // 提供適當的反饋信息
                    let feedback_msg = if has_command_key {
                        "API 連線測試成功，已儲存設定（API 金鑰已保存到 .env 文件中）"
                    } else if has_env_key {
                        "API 連線測試成功，已儲存設定（將使用 .env 文件中的 API 金鑰）"
                    } else {
                        "API 連線測試成功，但沒有提供 API 金鑰。請在 .env 文件中設置相應的 API 金鑰環境變數。"
                    };

                    let embed = serenity::CreateEmbed::default()
                        .title("API 設定成功")
                        .description(feedback_msg)
                        .colour(serenity::Colour::DARK_GREEN);
                    ctx.send(CreateReply::default().embed(embed)).await?;
                }
                Ok(Err(e)) => {
                    // 選擇正確的模型名稱用於日誌
                    let log_model = model.clone().unwrap_or_else(|| {
                        let provider = determine_provider_from_url(&api_url);
                        crate::utils::api::get_default_model_for_provider(&provider)
                    });

                    // 記錄詳細錯誤信息
                    log::error!(
                        "API 測試失敗: URL={}, Model={}, Error={}",
                        api_url,
                        log_model,
                        e
                    );

                    let embed = serenity::CreateEmbed::default()
                        .title("API 設定失敗")
                        .description(format!("API 連線測試失敗: {}", e))
                        .colour(serenity::Colour::RED);
                    ctx.send(CreateReply::default().embed(embed)).await?;
                }
                Err(_) => {
                    // 選擇正確的模型名稱用於日誌
                    let log_model = model.clone().unwrap_or_else(|| {
                        let provider = determine_provider_from_url(&api_url);
                        crate::utils::api::get_default_model_for_provider(&provider)
                    });

                    log::warn!("API 測試超時: URL={}, Model={}", api_url, log_model);

                    let embed = serenity::CreateEmbed::default()
                        .title("API 設定失敗")
                        .description("API 連線測試超時")
                        .colour(serenity::Colour::RED);
                    ctx.send(CreateReply::default().embed(embed)).await?;
                }
            }
        }
        ApiAction::Remove => {
            let all_configs = api_manager.get_guild_configs(guild_id).await;

            // 如果沒有指定要刪除的名稱，且有多個配置，則提示用戶指定名稱
            if name.is_none() && all_configs.len() > 1 {
                let embed = serenity::CreateEmbed::default()
                    .title("多個API設定")
                    .description("此伺服器有多個API設定。請使用 `/chat list` 查看所有設定，並指定要刪除的設定名稱。\n範例: /chat remove name:設定名稱")
                    .colour(serenity::Colour::ORANGE);
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            }

            let api_name_to_remove = if let Some(ref specified_name) = name {
                specified_name.clone()
            } else {
                // 如果只有一個配置，使用活動API名稱
                let active_config = api_manager.get_guild_config(guild_id).await;
                active_config.name
            };

            let success = api_manager
                .remove_guild_config(guild_id, &api_name_to_remove)
                .await;

            if success {
                let embed = serenity::CreateEmbed::default()
                    .title("API 設定已移除")
                    .description(format!(
                        "已清除此伺服器的 '{}' API 設定",
                        api_name_to_remove
                    ))
                    .colour(serenity::Colour::DARK_GREEN);
                ctx.send(CreateReply::default().embed(embed)).await?;
            } else {
                let embed = serenity::CreateEmbed::default()
                    .title("API 設定移除失敗")
                    .description(format!("沒有找到名為 '{}' 的 API 設定", api_name_to_remove))
                    .colour(serenity::Colour::RED);
                ctx.send(CreateReply::default().embed(embed)).await?;
            }
        }
        ApiAction::Toggle => {
            let all_configs = api_manager.get_guild_configs(guild_id).await;

            if all_configs.is_empty() {
                let embed = serenity::CreateEmbed::default()
                    .title("錯誤")
                    .description("此伺服器沒有設定任何API配置")
                    .colour(serenity::Colour::RED);
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            }

            // 如果沒有指定要切換的名稱，則使用活動API配置
            let target_name = if let Some(ref specified_name) = name {
                specified_name.clone()
            } else {
                // 使用活動API配置
                let active_config = api_manager.get_guild_config(guild_id).await;
                active_config.name
            };

            if let Some(mut config) = all_configs.get(&target_name).cloned() {
                let was_enabled = config.enabled;
                config.enabled = !was_enabled;

                // 將更新後的配置重新添加到存儲
                api_manager.add_guild_config(guild_id, config).await;

                let status = if !was_enabled {
                    "已啟用"
                } else {
                    "已停用"
                };
                let embed = serenity::CreateEmbed::default()
                    .title("API 狀態切換")
                    .description(format!("API '{}' 已{}", target_name, status))
                    .colour(serenity::Colour::BLURPLE);
                ctx.send(CreateReply::default().embed(embed)).await?;
            } else {
                let embed = serenity::CreateEmbed::default()
                    .title("錯誤")
                    .description(format!(
                        "找不到名為 '{}' 的API設定。請使用 `/chat list` 查看可用設定。",
                        target_name
                    ))
                    .colour(serenity::Colour::RED);
                ctx.send(CreateReply::default().embed(embed)).await?;
            }
        }
        ApiAction::ListModels => {
            // 獲取當前伺服器的配置
            let current_config = api_manager.get_guild_config(guild_id).await;

            // 檢查是否有環境變數中的API金鑰
            let effective_api_key = current_config
                .api_key
                .clone()
                .or_else(|| crate::utils::api::get_api_key_from_env(&current_config.provider));

            if effective_api_key.is_none() {
                let embed = serenity::CreateEmbed::default()
                    .title("模型列表")
                    .description("此伺服器尚未設定 API 金鑰，無法獲取模型列表。")
                    .colour(serenity::Colour::RED);
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            }

            let api_key = effective_api_key.as_ref().unwrap(); // 已確認不為 None

            match crate::utils::api::get_models_list(
                &current_config.api_url,
                Some(api_key),
                &current_config.provider,
            )
            .await
            {
                Ok(models_list) => {
                    if !models_list.is_empty() {
                        // 限制模型顯示數量，避免 Discord 消息長度限制
                        let models_to_show = if models_list.len() > 50 {
                            format!("顯示前 50 個模型（共 {} 個）：\n", models_list.len())
                        } else {
                            String::new()
                        };

                        let models_str = models_list
                            .iter()
                            .take(50) // 限制顯示前 50 個模型
                            .map(|model| format!("- {}", model))
                            .collect::<Vec<_>>()
                            .join("\n");

                        let full_description = format!("{}{}", models_to_show, models_str);

                        let embed = serenity::CreateEmbed::default()
                            .title("可用模型列表")
                            .description(full_description)
                            .colour(serenity::Colour::BLURPLE);
                        ctx.send(CreateReply::default().embed(embed)).await?;
                    } else {
                        let embed = serenity::CreateEmbed::default()
                            .title("模型列表")
                            .description("API 回應中沒有模型數據。")
                            .colour(serenity::Colour::ORANGE);
                        ctx.send(CreateReply::default().embed(embed)).await?;
                    }
                }
                Err(_) => {
                    // 如果獲取模型列表失敗，顯示當前配置的模型
                    let embed = serenity::CreateEmbed::default()
                        .title("可用模型")
                        .description(format!(
                            "無法從 API 獲取模型列表。\n當前設定的模型: {}",
                            current_config.model
                        ))
                        .colour(serenity::Colour::ORANGE);
                    ctx.send(CreateReply::default().embed(embed)).await?;
                }
            }
        }
        ApiAction::List => {
            // 獲取當前伺服器的所有API配置
            let all_configs = api_manager.get_guild_configs(guild_id).await;

            // 獲取活動API配置名稱
            let data = ctx.data();
            let config_guard = data.config.lock().await;
            let guilds_read = config_guard.guilds.read().await;
            let active_api = if let Some(guild_config) = guilds_read.get(&guild_id) {
                guild_config.active_api.clone().unwrap_or_default()
            } else {
                String::new()
            };
            drop(guilds_read); // 釋放對guilds的借用
            drop(config_guard); // 釋放對config的鎖

            if all_configs.is_empty() {
                let embed = serenity::CreateEmbed::default()
                    .title("API設定列表")
                    .description("此伺服器尚未設定任何API。")
                    .colour(serenity::Colour::ORANGE);
                ctx.send(CreateReply::default().embed(embed)).await?;
            } else {
                let mut description = String::new();
                for (name, config) in &all_configs {
                    let status = if config.enabled { "✅" } else { "❌" };
                    let active_marker = if name == &active_api { " 🌟" } else { "" };
                    let provider_debug = format!("{:?}", config.provider);
                    description.push_str(&format!(
                        "{} **{}**{} - {} ({})\n",
                        status, name, active_marker, config.model, provider_debug
                    ));
                }

                let embed = serenity::CreateEmbed::default()
                    .title("API設定列表")
                    .description(description)
                    .colour(serenity::Colour::BLURPLE);
                ctx.send(CreateReply::default().embed(embed)).await?;
            }
        }
        ApiAction::Switch => {
            let all_configs = api_manager.get_guild_configs(guild_id).await;

            if all_configs.is_empty() {
                let embed = serenity::CreateEmbed::default()
                    .title("錯誤")
                    .description("此伺服器沒有設定任何API配置")
                    .colour(serenity::Colour::RED);
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            }

            if let Some(ref target_name) = name {
                if all_configs.contains_key(target_name) {
                    // 切換到指定的API配置
                    let success = api_manager.set_active_api(guild_id, target_name).await;

                    if success {
                        let embed = serenity::CreateEmbed::default()
                            .title("API 切換成功")
                            .description(format!("已切換到 '{}' API 設定", target_name))
                            .colour(serenity::Colour::DARK_GREEN);
                        ctx.send(CreateReply::default().embed(embed)).await?;
                    } else {
                        let embed = serenity::CreateEmbed::default()
                            .title("API 切換失敗")
                            .description(format!("無法切換到 '{}' API 設定", target_name))
                            .colour(serenity::Colour::RED);
                        ctx.send(CreateReply::default().embed(embed)).await?;
                    }
                } else {
                    let embed = serenity::CreateEmbed::default()
                        .title("錯誤")
                        .description(format!(
                            "找不到名為 '{}' 的API設定。請使用 `/chat list` 查看可用設定。",
                            target_name
                        ))
                        .colour(serenity::Colour::RED);
                    ctx.send(CreateReply::default().embed(embed)).await?;
                }
            } else {
                // 顯示可用的配置列表，讓用戶知道可以選擇什麼
                let mut description = String::new();
                for (name, config) in &all_configs {
                    let status = if config.enabled { "✅" } else { "❌" };
                    let provider_debug = format!("{:?}", config.provider);
                    description.push_str(&format!(
                        "{} **{}** - {} ({})\n",
                        status, name, config.model, provider_debug
                    ));
                }

                let embed = serenity::CreateEmbed::default()
                    .title("可用的API設定")
                    .description(
                        "請使用指令指定要切換到的API設定名稱。\n範例: /chat switch name:設定名稱",
                    )
                    .field("設定列表", description, false)
                    .colour(serenity::Colour::BLURPLE);
                ctx.send(CreateReply::default().embed(embed)).await?;
            }
        }
    }

    Ok(())
}

/// 將API金鑰保存到 .env 文件中
async fn save_api_key_to_env(provider: &ApiProvider, key: &str) {
    let env_path = Path::new(".env");

    // 讀取現有的 .env 內容
    let env_content = if env_path.exists() {
        std::fs::read_to_string(env_path).unwrap_or_default()
    } else {
        String::new()
    };

    // 確定要寫入的環境變數名稱
    let env_var_name = match provider {
        ApiProvider::OpenAI => "OPENAI_API_KEY",
        ApiProvider::OpenRouter => "OPENROUTER_API_KEY",
        ApiProvider::Anthropic => "ANTHROPIC_API_KEY",
        ApiProvider::Google => "GOOGLE_API_KEY",
        ApiProvider::Custom => "CUSTOM_API_KEY",
    };

    // 檢查環境變數是否已經存在
    let mut lines: Vec<String> = env_content.lines().map(|s| s.to_string()).collect();
    let mut found = false;

    for line in &mut lines {
        if line.starts_with(&format!("{}=", env_var_name)) {
            *line = format!("{}={}", env_var_name, key);
            found = true;
            break;
        }
    }

    // 如果環境變數不存在，則添加新行
    if !found {
        lines.push(format!("{}={}", env_var_name, key));
    }

    // 寫回 .env 文件
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(env_path)
    {
        let new_content = lines.join("\n");
        let _ = file.write_all(new_content.as_bytes());
    }
}

fn determine_provider_from_url(url: &str) -> ApiProvider {
    if url.contains("openrouter.ai") {
        ApiProvider::OpenRouter
    } else if url.contains("anthropic") {
        ApiProvider::Anthropic
    } else if url.contains("google") {
        ApiProvider::Google
    } else {
        ApiProvider::OpenAI // Default to OpenAI for compatibility
    }
}
