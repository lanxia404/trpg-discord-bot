use crate::bot::data::BotData;

type Error = anyhow::Error;
type Context<'a> = poise::Context<'a, BotData, Error>;

/// 生成對話摘要
///
/// 使用 AI 自動總結最近的對話內容
#[poise::command(slash_command, guild_only)]
pub async fn summarize(
    ctx: Context<'_>,
    #[description = "要總結的訊息數量 (預設: 50)"]
    #[min = 10]
    #[max = 200]
    count: Option<usize>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow::anyhow!("此指令只能在伺服器中使用"))?
        .get();
    let channel_id = ctx.channel_id().get();

    let message_count = count.unwrap_or(50);

    ctx.defer().await?;

    log::info!(
        "開始為 guild_id={}, channel_id={} 生成摘要,訊息數={}",
        guild_id,
        channel_id,
        message_count
    );

    // 調用 ConversationManager 生成摘要
    match ctx
        .data()
        .conversation_manager
        .summarize_conversation(guild_id, channel_id, message_count)
        .await
    {
        Ok(summary) => {
            let response = format!(
                "📝 **對話摘要** (最近 {} 條訊息)\n\n{}",
                message_count, summary
            );

            ctx.say(response).await?;
            log::info!("摘要生成成功");
        }
        Err(e) => {
            log::error!("生成摘要失敗: {}", e);
            ctx.say(format!("❌ 生成摘要時發生錯誤: {}", e)).await?;
        }
    }

    Ok(())
}
