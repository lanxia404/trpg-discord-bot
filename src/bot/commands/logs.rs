use crate::bot::{Context, Error};
use crate::models::types::StreamMode;
use poise::ChoiceParameter;
use poise::serenity_prelude as serenity;

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

/// 控制日誌串流開關
#[poise::command(slash_command)]
pub async fn log_stream(
    ctx: Context<'_>,
    #[description = "開關狀態"] state: StreamToggle,
    #[description = "當 state=on 時指定串流頻道"]
    #[channel_types("Text")]
    channel: Option<serenity::ChannelId>,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令只能在伺服器中使用").await?;
            return Ok(());
        }
    };

    let mut config_manager = ctx.data().config.lock().await;
    let mut guild_config = config_manager.get_guild_config(guild_id);

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
            config_manager.set_guild_config(guild_id, guild_config)?;
            ctx.say(format!("日誌串流已開啟，使用頻道: <#{}>", channel))
                .await?;
        }
        StreamToggle::Off => {
            guild_config.log_channel = None;
            config_manager.set_guild_config(guild_id, guild_config)?;
            ctx.say("日誌串流已關閉").await?;
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
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令只能在伺服器中使用").await?;
            return Ok(());
        }
    };

    let mut config_manager = ctx.data().config.lock().await;
    let mut guild_config = config_manager.get_guild_config(guild_id);
    guild_config.stream_mode = mode.into();
    config_manager.set_guild_config(guild_id, guild_config)?;

    let mode_text = match mode {
        StreamModeChoice::Live => "live",
        StreamModeChoice::Batch => "batch",
    };
    ctx.say(format!("串流模式已設定為: {}", mode_text)).await?;
    Ok(())
}
