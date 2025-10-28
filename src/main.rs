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

    let skills_db = tokio_rusqlite::Connection::open("skills.db")
        .await
        .map_err(|e| anyhow!("開啟技能資料庫失敗: {}", e))?;
    skills_db.call(|conn| {
        // 檢查是否需要遷移：檢查是否存在 user_id 欄位
        let mut has_user_id = false;
        {
            let mut stmt = conn.prepare("PRAGMA table_info(skills)")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let column_name: String = row.get(1)?;
                if column_name == "user_id" {
                    has_user_id = true;
                    break;
                }
            }
        }

        // 如果存在 user_id 欄位，則重建表格（移除 user_id 欄位）
        if has_user_id {
            conn.execute_batch(
                "BEGIN;
                DROP TABLE IF EXISTS skills_tmp;
                CREATE TABLE skills_tmp (
                    guild_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    normalized_name TEXT NOT NULL,
                    skill_type TEXT NOT NULL,
                    level TEXT NOT NULL,
                    effect TEXT NOT NULL,
                    UNIQUE(guild_id, normalized_name)
                );
                INSERT INTO skills_tmp (guild_id, name, normalized_name, skill_type, level, effect)
                SELECT guild_id, name, normalized_name, skill_type, level, effect
                FROM skills;
                DROP TABLE skills;
                ALTER TABLE skills_tmp RENAME TO skills;
                COMMIT;",
            )?;
        } else {
            // 如果沒有 user_id 欄位，則按照新結構創建表
            conn.execute(
                "CREATE TABLE IF NOT EXISTS skills (
                    guild_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    normalized_name TEXT NOT NULL,
                    skill_type TEXT NOT NULL,
                    level TEXT NOT NULL,
                    effect TEXT NOT NULL,
                    UNIQUE(guild_id, normalized_name)
                )",
                [],
            )?;
        }

        Ok(())
    })
    .await
    .map_err(|e| anyhow!("初始化技能資料庫失敗: {}", e))?;

    let base_settings_db = tokio_rusqlite::Connection::open("base_settings.db")
        .await
        .map_err(|e| anyhow!("開啟基本設定資料庫失敗: {}", e))?;
    base_settings_db
        .call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS base_settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
            )?;
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
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let config = Arc::clone(&setup_config);
            let skills_db = setup_skills_db.clone();
            let base_settings_db = setup_base_settings_db.clone();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("{} 已經上線!", ready.user.name);
                Ok(BotData { config, skills_db, base_settings_db })
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
