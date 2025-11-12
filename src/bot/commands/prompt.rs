use crate::bot::{Context, Error};

/// 系統提示詞管理指令
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("set", "reset", "view", "context")
)]
pub async fn prompt(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("請使用子指令：set, reset, view, context").await?;
    Ok(())
}

/// 設置自定義系統提示詞
#[poise::command(prefix_command, slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "自定義系統提示詞"] prompt: String,
) -> Result<(), Error> {
    log::info!(
        "設置自定義提示詞 for guild {:?}, user={}",
        ctx.guild_id(),
        ctx.author().id
    );

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令僅能在伺服器中使用").await?;
            return Ok(());
        }
    };

    // 獲取並更新配置
    let config = ctx.data().config.lock().await;
    let mut guild_config = config.get_guild_config(guild_id).await;
    guild_config.custom_system_prompt = Some(prompt.clone());
    
    config.set_guild_config(guild_id, guild_config).await?;
    drop(config);

    ctx.say(format!(
        "✅ 已設置自定義系統提示詞\n\n預覽:\n```\n{}\n```\n\n使用 `/prompt reset` 可恢復預設提示詞",
        &prompt[..prompt.len().min(200)]
    ))
    .await?;
    
    Ok(())
}

/// 重置為預設系統提示詞
#[poise::command(prefix_command, slash_command)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    log::info!(
        "重置系統提示詞 for guild {:?}, user={}",
        ctx.guild_id(),
        ctx.author().id
    );

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令僅能在伺服器中使用").await?;
            return Ok(());
        }
    };

    // 獲取並更新配置
    let config = ctx.data().config.lock().await;
    let mut guild_config = config.get_guild_config(guild_id).await;
    guild_config.custom_system_prompt = None;
    
    config.set_guild_config(guild_id, guild_config).await?;
    drop(config);

    ctx.say("✅ 已重置為預設 TRPG 助手提示詞").await?;
    
    Ok(())
}

/// 查看當前系統提示詞
#[poise::command(prefix_command, slash_command)]
pub async fn view(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令僅能在伺服器中使用").await?;
            return Ok(());
        }
    };

    let config = ctx.data().config.lock().await;
    let guild_config = config.get_guild_config(guild_id).await;
    drop(config);

    let prompt = if let Some(custom) = &guild_config.custom_system_prompt {
        format!("**自定義系統提示詞:**\n```\n{}\n```", custom)
    } else {
        "**使用預設 TRPG 助手提示詞**\n\n```\n你是一個專業的 TRPG (桌上角色扮演遊戲) 助手。\n你的任務是幫助玩家和 GM (遊戲主持人) 進行遊戲。\n...\n```".to_string()
    };

    // 添加 D&D 規則資訊
    let rules_info = format!(
        "\n\n**伺服器規則:**\n• 大成功: {}\n• 大失敗: {}",
        guild_config.dnd_rules.critical_success,
        guild_config.dnd_rules.critical_fail
    );

    ctx.say(format!("{}{}", prompt, rules_info)).await?;
    
    Ok(())
}

/// 配置上下文參數
#[poise::command(prefix_command, slash_command)]
pub async fn context(
    ctx: Context<'_>,
    #[description = "Token 預算比例 (0.5-0.9)"] ratio: Option<f32>,
    #[description = "最大記憶檢索數 (3-20)"] max_memory: Option<usize>,
    #[description = "最大歷史訊息數 (5-50)"] max_history: Option<usize>,
) -> Result<(), Error> {
    log::info!(
        "配置上下文參數 for guild {:?}, user={}",
        ctx.guild_id(),
        ctx.author().id
    );

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令僅能在伺服器中使用").await?;
            return Ok(());
        }
    };

    // 獲取並更新配置
    let config = ctx.data().config.lock().await;
    let mut guild_config = config.get_guild_config(guild_id).await;
    
    let mut changes = Vec::new();
    
    if let Some(r) = ratio {
        let clamped = r.clamp(0.5, 0.9);
        guild_config.context_config.token_budget_ratio = clamped;
        changes.push(format!("• Token 預算比例: {:.2}", clamped));
    }
    
    if let Some(m) = max_memory {
        let clamped = m.clamp(3, 20);
        guild_config.context_config.max_memory_results = clamped;
        changes.push(format!("• 最大記憶檢索數: {}", clamped));
    }
    
    if let Some(h) = max_history {
        let clamped = h.clamp(5, 50);
        guild_config.context_config.max_history_messages = clamped;
        changes.push(format!("• 最大歷史訊息數: {}", clamped));
    }
    
    if changes.is_empty() {
        // 顯示當前配置
        let cfg = &guild_config.context_config;
        ctx.say(format!(
            "**當前上下文配置:**\n\
             • Token 預算比例: {:.2}\n\
             • 記憶檢索範圍: {}-{} 條\n\
             • 歷史訊息範圍: {}-{} 條",
            cfg.token_budget_ratio,
            cfg.min_memory_results,
            cfg.max_memory_results,
            cfg.min_history_messages,
            cfg.max_history_messages
        )).await?;
    } else {
        config.set_guild_config(guild_id, guild_config.clone()).await?;
        
        ctx.say(format!(
            "✅ 已更新上下文配置:\n{}\n\n當前完整配置:\n\
             • Token 預算比例: {:.2}\n\
             • 記憶檢索範圍: {}-{} 條\n\
             • 歷史訊息範圍: {}-{} 條",
            changes.join("\n"),
            guild_config.context_config.token_budget_ratio,
            guild_config.context_config.min_memory_results,
            guild_config.context_config.max_memory_results,
            guild_config.context_config.min_history_messages,
            guild_config.context_config.max_history_messages
        )).await?;
    }
    
    drop(config);
    Ok(())
}
