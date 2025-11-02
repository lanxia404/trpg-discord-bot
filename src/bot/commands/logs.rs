use crate::bot::{Context, Error};
use crate::models::types::StreamMode;
use poise::{ChoiceParameter, CreateReply, serenity_prelude as serenity};

#[derive(Clone, Copy, Debug, ChoiceParameter)]
pub enum StreamToggle {
    #[name = "on"]
    On,
    #[name = "off"]
    Off,
}

#[derive(Clone, Copy, Debug, ChoiceParameter)]
pub enum StreamModeChoice {
    #[name = "live"]
    Live,
    #[name = "batch"]
    Batch,
}

impl From<StreamModeChoice> for StreamMode {
    fn from(choice: StreamModeChoice) -> Self {
        match choice {
            StreamModeChoice::Live => StreamMode::Live,
            StreamModeChoice::Batch => StreamMode::Batch,
        }
    }
}

#[derive(Clone, Copy, Debug, ChoiceParameter)]
pub enum CritChannelKind {
    #[name = "success"]
    Success,
    #[name = "fail"]
    Fail,
}

/// 控制日誌串流開關
#[poise::command(slash_command)]
pub async fn log_stream(
    ctx: Context<'_>,
    #[description = "開關狀態"] state: StreamToggle,
    #[description = "當 state=on 時指定串流頻道"]
    #[channel_types("Text")]
    channel: Option<serenity::ChannelId>,
) -> Result<(), Error> {
    log::info!("執行日誌串流指令: {:?} for guild {:?}", state, ctx.guild_id());
    
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令只能在伺服器中使用").await?;
            return Ok(());
        }
    };

    let config_manager = ctx.data().config.lock().await;
    let mut guild_config = futures::executor::block_on(config_manager.get_guild_config(guild_id));

    match state {
        StreamToggle::On => {
            let channel = match channel {
                Some(ch) => ch,
                None => {
                    ctx.say("請提供要啟用串流的文字頻道").await?;
                    return Ok(());
                }
            };
            guild_config.log_channel = Some(channel.get());
            match futures::executor::block_on(config_manager.set_guild_config(guild_id, guild_config)) {
                Ok(_) => {
                    log::info!("日誌串流已設定到頻道: {}", channel.get());
                    ctx.say(format!("日誌串流已開啟，使用頻道: <#{}>", channel))
                        .await?;
                }
                Err(e) => {
                    log::error!("設定日誌串流設定失敗: {:?}", e);
                    ctx.say("設定日誌串流時發生錯誤").await?;
                    return Err(e.into());
                }
            }
        }
        StreamToggle::Off => {
            guild_config.log_channel = None;
            match futures::executor::block_on(config_manager.set_guild_config(guild_id, guild_config)) {
                Ok(_) => {
                    log::info!("日誌串流已關閉 for guild {}", guild_id);
                    ctx.say("日誌串流已關閉").await?;
                }
                Err(e) => {
                    log::error!("關閉日誌串流設定失敗: {:?}", e);
                    ctx.say("關閉日誌串流時發生錯誤").await?;
                    return Err(e.into());
                }
            }
        }
    }

    Ok(())
}

/// 設定日誌串流模式
#[poise::command(slash_command)]
pub async fn log_stream_mode(
    ctx: Context<'_>,
    #[description = "串流模式 (live 或 batch)"] mode: StreamModeChoice,
) -> Result<(), Error> {
    log::info!("執行日誌串流模式指令: {:?} for guild {:?}", mode, ctx.guild_id());
    
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令只能在伺服器中使用").await?;
            return Ok(());
        }
    };

    let config_manager = ctx.data().config.lock().await;
    let mut guild_config = futures::executor::block_on(config_manager.get_guild_config(guild_id));
    guild_config.stream_mode = mode.into();
    
    match futures::executor::block_on(config_manager.set_guild_config(guild_id, guild_config)) {
        Ok(_) => {
            let mode_text = match mode {
                StreamModeChoice::Live => "live",
                StreamModeChoice::Batch => "batch",
            };
            log::info!("串流模式已設定為: {} for guild {}", mode_text, guild_id);
            ctx.say(format!("串流模式已設定為: {}", mode_text)).await?;
        }
        Err(e) => {
            log::error!("設定串流模式失敗: {:?}", e);
            ctx.say("設定串流模式時發生錯誤").await?;
            return Err(e.into());
        }
    }
    
    Ok(())
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
    log::info!("執行 crit 設定指令: {:?} for guild {:?}", kind, ctx.guild_id());
    
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
