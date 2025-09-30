use serenity::Client;
use serenity::model::prelude::GatewayIntents;
use std::env;
use crate::bot::handler::BotHandler;
use crate::utils::config::ConfigManager;

mod models;
mod utils;
mod bot;

#[tokio::main]
async fn main() {
    // 初始化日誌
    if let Err(e) = utils::logger::DiscordLogger::init("bot.log") {
        println!("日誌初始化失敗: {:?}", e);
    }

    // 從環境變數獲取 Discord token
    let token = env::var("DISCORD_TOKEN")
        .expect("預期 DISCORD_TOKEN 環境變數，但找不到!");

    // 初始化設定管理器
    let config_manager = match ConfigManager::new("config.json") {
        Ok(config) => config,
        Err(e) => {
            println!("設定管理器初始化失敗: {:?}", e);
            ConfigManager::new("config.json").expect("設定管理器創建失敗")
        }
    };

    // 創建機器人處理器
    let handler = BotHandler::new(config_manager);

    // 創建客戶端
    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Err creating client");

    // 啟動機器人
    if let Err(e) = client.start().await {
        println!("機器人啟動失敗: {:?}", e);
    }
}