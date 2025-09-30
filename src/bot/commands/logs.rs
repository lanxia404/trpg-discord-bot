use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, Context};
use serenity::model::prelude::CommandDataOption;
use crate::utils::config::ConfigManager;

pub async fn register_log_stream_set_command() -> CreateCommand {
    CreateCommand::new("log-stream-set")
        .description("設定日誌串流頻道")
        .add_option(CreateCommandOption::new(
            CommandOptionType::Channel,
            "channel",
            "要設定為日誌串流的頻道",
        ).required(true))
}

pub async fn register_log_stream_off_command() -> CreateCommand {
    CreateCommand::new("log-stream-off")
        .description("關閉日誌串流")
}

pub async fn register_log_stream_mode_command() -> CreateCommand {
    CreateCommand::new("log-stream-mode")
        .description("設定串流模式")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "mode",
            "串流模式 (live 或 batch)",
        ).add_string_choice("live", "live")
         .add_string_choice("batch", "batch")
         .required(true))
}

pub async fn handle_log_stream_set_command(
    _ctx: &Context,
    mut command_options: Vec<CommandDataOption>,
    config_manager: &mut ConfigManager,
    guild_id: u64,
) -> String {
    if command_options.is_empty() {
        return "請提供頻道".to_string();
    }
    
    let option = command_options.remove(0);
    let channel_id = if let serenity::all::CommandDataOptionValue::Channel(channel) = option.value {
        channel
    } else {
        return "參數必須是頻道".to_string();
    };

    let mut guild_config = config_manager.get_guild_config(guild_id);
    guild_config.log_channel = Some(channel_id.get());
    config_manager.set_guild_config(guild_id, guild_config);
    
    // Save the updated config
    if let Err(e) = config_manager.save_config() {
        return format!("設定保存失敗: {}", e);
    }

    format!("日誌串流已設定到頻道: <#{}>", channel_id)
}

pub async fn handle_log_stream_off_command(
    _ctx: &Context,
    _command_options: Vec<CommandDataOption>,
    config_manager: &mut ConfigManager,
    guild_id: u64,
) -> String {
    let mut guild_config = config_manager.get_guild_config(guild_id);
    guild_config.log_channel = None;
    config_manager.set_guild_config(guild_id, guild_config);
    
    // Save the updated config
    if let Err(e) = config_manager.save_config() {
        return format!("設定保存失敗: {}", e);
    }

    "日誌串流已關閉".to_string()
}

pub async fn handle_log_stream_mode_command(
    _ctx: &Context,
    mut command_options: Vec<CommandDataOption>,
    config_manager: &mut ConfigManager,
    guild_id: u64,
) -> String {
    if command_options.is_empty() {
        return "請提供模式".to_string();
    }
    
    let option = command_options.remove(0);
    let mode = if let serenity::all::CommandDataOptionValue::String(m) = option.value {
        m
    } else {
        return "模式必須是字串".to_string();
    };

    let mut guild_config = config_manager.get_guild_config(guild_id);
    
    match mode.as_str() {
        "live" => guild_config.stream_mode = crate::models::types::StreamMode::Live,
        "batch" => guild_config.stream_mode = crate::models::types::StreamMode::Batch,
        _ => return "無效的模式，請使用 'live' 或 'batch'".to_string(),
    }
    
    config_manager.set_guild_config(guild_id, guild_config);
    
    // Save the updated config
    if let Err(e) = config_manager.save_config() {
        return format!("設定保存失敗: {}", e);
    }

    format!("串流模式已設定為: {}", mode)
}