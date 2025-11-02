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

    let config_manager = ConfigManager::new("config.json").await.map_err(|e| anyhow!("設定管理器初始化失敗: {}", e))?;
    let shared_config = Arc::new(Mutex::new(config_manager));
    // 下面開始建立並初始化資料庫
    let skills_db = tokio_rusqlite::Connection::open("skills.db")
        .await
        .map_err(|e| anyhow!("開啟技能資料庫失敗: {}", e))?;
    skills_db
        .call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS skills (
                    guild_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    normalized_name TEXT NOT NULL,
                    skill_type TEXT NOT NULL,
                    level TEXT NOT NULL,
                    effect TEXT NOT NULL,
                    occupation TEXT DEFAULT '',
                    race TEXT DEFAULT '',
                    UNIQUE(guild_id, normalized_name)
                )",
                [],
            )?;
            
            Ok(())
        })
        .await
        .map_err(|e| anyhow!("初始化技能資料庫失敗: {}", e))?;

    let base_settings_db = tokio_rusqlite::Connection::open("base_settings.db")
        .await
        .map_err(|e| anyhow!("開啟基本設定資料庫失敗: {}", e))?;
    // base_settings.db 現在用於存儲導入的數據表，無需預設表結構
    // 導入功能將根據數據類型自動創建對應的表
    base_settings_db
        .call(|conn| {
            // 確保資料庫連接正常
            conn.execute("CREATE TABLE IF NOT EXISTS __temp_check (id INTEGER)", [])
                .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;
            conn.execute("DROP TABLE IF EXISTS __temp_check", [])
                .map_err(|e| tokio_rusqlite::Error::Rusqlite(e))?;
            Ok(())
        })
        .await
        .map_err(|e| anyhow!("初始化基本設定資料庫失敗: {}", e))?;

    let intents = serenity::GatewayIntents::GUILDS;

    let setup_config = Arc::clone(&shared_config);
    let setup_skills_db = skills_db.clone();
    let setup_base_settings_db = base_settings_db.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: crate::bot::commands(),
            on_error: |error| {
                Box::pin(async move {
                    log::error!("指令執行錯誤: {}", error);
                    
                    // 嘗試獲取具體的錯誤資訊
                    let error_msg = format!("發生錯誤: {}", error);
                    
                    // 如果有互動回應，向使用者發送錯誤訊息
                    if let poise::FrameworkError::Command { ctx, .. } = error {
                        if let Err(why) = ctx.say(error_msg).await {
                            log::error!("發送錯誤訊息失敗: {}", why);
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let config = Arc::clone(&setup_config);
            let skills_db = setup_skills_db.clone();
            let base_settings_db = setup_base_settings_db.clone();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("{} 已經上線!", ready.user.name);
                Ok(BotData {
                    config,
                    skills_db,
                    base_settings_db,
                })
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
