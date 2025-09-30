use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::utils::config::ConfigManager;

pub struct BotHandler {
    pub config_manager: Arc<Mutex<ConfigManager>>,
}

impl BotHandler {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self {
            config_manager: Arc::new(Mutex::new(config_manager)),
        }
    }
}

#[async_trait]
impl EventHandler for BotHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} 已經上線!", ready.user.name);
        
        // 建立機器人指令
        let commands = vec![
            // 擲骰指令
            crate::bot::commands::dice::register_dice_command().await,
            crate::bot::commands::dice::register_coc_command().await,
            
            // 日誌指令
            crate::bot::commands::logs::register_log_stream_set_command().await,
            crate::bot::commands::logs::register_log_stream_off_command().await,
            crate::bot::commands::logs::register_log_stream_mode_command().await,
            
            // 管理指令
            crate::bot::commands::admin::register_admin_command().await,
            
            // 說明指令
            crate::bot::commands::help::register_help_command().await,
        ];
        
        // 註冊全域指令
        if let Err(e) = ctx.http.create_global_commands(&commands).await {
            println!("註冊指令失敗: {:?}", e);
        } else {
            println!("已註冊 {} 個指令", commands.len());
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(mut command) = interaction {
            let options = std::mem::take(&mut command.data.options);
            let content = match command.data.name.as_str() {
                "roll" => {
                    let config_manager = self.config_manager.lock().await;
                    crate::bot::commands::dice::handle_dice_command(&ctx, options, &config_manager).await
                },
                "coc" => {
                    let config_manager = self.config_manager.lock().await;
                    crate::bot::commands::dice::handle_coc_command(&ctx, options, &config_manager).await
                },
                "log-stream-set" => {
                    if let Some(guild_id) = command.guild_id {
                        let mut config_manager = self.config_manager.lock().await;
                        crate::bot::commands::logs::handle_log_stream_set_command(&ctx, options, &mut config_manager, guild_id.get()).await
                    } else {
                        "此指令只能在伺服器中使用".to_string()
                    }
                },
                "log-stream-off" => {
                    if let Some(guild_id) = command.guild_id {
                        let mut config_manager = self.config_manager.lock().await;
                        crate::bot::commands::logs::handle_log_stream_off_command(&ctx, options, &mut config_manager, guild_id.get()).await
                    } else {
                        "此指令只能在伺服器中使用".to_string()
                    }
                },
                "log-stream-mode" => {
                    if let Some(guild_id) = command.guild_id {
                        let mut config_manager = self.config_manager.lock().await;
                        crate::bot::commands::logs::handle_log_stream_mode_command(&ctx, options, &mut config_manager, guild_id.get()).await
                    } else {
                        "此指令只能在伺服器中使用".to_string()
                    }
                },
                "admin" => {
                    let mut config_manager = self.config_manager.lock().await;
                    crate::bot::commands::admin::handle_admin_command(&ctx, options, &mut config_manager, command.user.id.get()).await
                },
                "help" => {
                    let config_manager = self.config_manager.lock().await;
                    crate::bot::commands::help::handle_help_command(&ctx, options, &config_manager).await
                },
                _ => "未知的指令".to_string(),
            };

            if let Err(e) = command.create_response(&ctx.http, serenity::builder::CreateInteractionResponse::Message(
                serenity::builder::CreateInteractionResponseMessage::new()
                    .content(content)
            )).await {
                println!("回應指令時發生錯誤: {:?}", e);
            }
        }
    }
}