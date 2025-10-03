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

#[tokio::main]
async fn main() -> Result<(), bot::Error> {
    if let Err(e) = utils::logger::DiscordLogger::init(Some("bot.log")) {
        eprintln!("日誌初始化失敗: {}", e);
    }

    dotenvy::dotenv().ok();

    let token =
        env::var("DISCORD_TOKEN").map_err(|_| anyhow!("預期 DISCORD_TOKEN 環境變數，但找不到!"))?;

    let config_manager =
        ConfigManager::new("config.json").map_err(|e| anyhow!("設定管理器初始化失敗: {}", e))?;
    let shared_config = Arc::new(Mutex::new(config_manager));

    let intents = serenity::GatewayIntents::GUILDS;

    let setup_config = Arc::clone(&shared_config);
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: crate::bot::commands(),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let config = Arc::clone(&setup_config);
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("{} 已經上線!", ready.user.name);
                Ok(BotData { config })
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
