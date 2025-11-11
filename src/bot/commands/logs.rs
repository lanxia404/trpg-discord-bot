use crate::bot::{Context, Error};
use poise::{ChoiceParameter, CreateReply, serenity_prelude as serenity};

#[derive(Clone, Copy, Debug, ChoiceParameter)]
pub enum CritChannelKind {
    #[name = "success"]
    Success,
    #[name = "fail"]
    Fail,
}

/// 設定大成功/大失敗紀錄頻道
#[poise::command(slash_command)]
pub async fn crit(
    ctx: Context<'_>,
    #[description = "紀錄類型"] kind: CritChannelKind,
    #[description = "要紀錄的頻道，留空則清除"]
    #[channel_types("Text")]
    channel: Option<serenity::ChannelId>,
) -> Result<(), Error> {
    log::info!(
        "執行 crit 設定指令: {:?} for guild {:?}",
        kind,
        ctx.guild_id()
    );

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

    let manager = ctx.data().config.lock().await;
    let mut guild_config = futures::executor::block_on(manager.get_guild_config(guild_id));

    let (label, field) = match kind {
        CritChannelKind::Success => ("大成功", &mut guild_config.crit_success_channel),
        CritChannelKind::Fail => ("大失敗", &mut guild_config.crit_fail_channel),
    };

    *field = channel.map(|ch| ch.get());

    match futures::executor::block_on(manager.set_guild_config(guild_id, guild_config)) {
        Ok(_) => {
            drop(manager);
            let description = match channel {
                Some(ch) => format!("已設定{}紀錄頻道為 <#{}>", label, ch),
                None => format!("已清除{}紀錄頻道設定", label),
            };

            log::info!("Crit 頻道設定更新: {}", description);

            let embed = serenity::CreateEmbed::default()
                .title("紀錄頻道已更新")
                .description(description)
                .colour(serenity::Colour::BLURPLE);
            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            log::error!("設定 crit 頻道失敗: {:?}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
