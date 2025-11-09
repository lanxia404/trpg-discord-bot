use crate::bot::{Context, Error};
use poise::CreateReply;

/// 清除當前伺服器的API配置（僅限開發者）
#[poise::command(slash_command, rename = "clear-api", track_edits = true, 
    guild_only = true, required_bot_permissions = "SEND_MESSAGES")]
pub async fn clear_api(ctx: Context<'_>) -> Result<(), Error> {
    // 檢查是否為開發者
    let user_id = ctx.author().id.get();
    let data = ctx.data();
    let config_manager = data.config.lock().await;
    let is_dev = config_manager.is_developer(user_id).await;
    drop(config_manager);

    if !is_dev {
        let response = CreateReply::default().content("❌ 你沒有權限執行此指令！");
        ctx.send(response).await?;
        return Ok(());
    }

    // 獲取當前伺服器ID
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            let response = CreateReply::default().content("❌ 此指令只能在伺服器中執行！");
            ctx.send(response).await?;
            return Ok(());
        }
    };

    // 清除當前伺服器的所有API配置
    let api_manager = &data.api_manager;
    let all_configs = api_manager.get_guild_configs(guild_id).await;
    let mut count = 0;
    
    for (name, _) in all_configs {
        if api_manager.remove_guild_config(guild_id, &name).await {
            count += 1;
        }
    }

    log::info!("已清除伺服器 {} 的 {} 個API配置", guild_id, count);

    let response = CreateReply::default().content(format!("✅ 成功清除此伺服器的 {} 個API配置（已從設定檔中移除）！", count));
    ctx.send(response).await?;
    Ok(())
}