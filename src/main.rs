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

    let db = tokio_rusqlite::Connection::open("skills.db")
        .await
        .map_err(|e| anyhow!("開啟資料庫失敗: {}", e))?;
    db.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS skills (
                guild_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                normalized_name TEXT NOT NULL,
                skill_type TEXT NOT NULL,
                level TEXT NOT NULL,
                effect TEXT NOT NULL,
                UNIQUE(guild_id, user_id, normalized_name)
            )",
            [],
        )?;

        // 既有資料表的結構升級：缺少欄位時補上；如果存在舊的 description 欄位則重建資料表。
        let mut has_description = false;
        {
            let mut stmt = conn.prepare("PRAGMA table_info(skills)")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let column_name: String = row.get(1)?;
                if column_name == "description" {
                    has_description = true;
                }
            }
        }

        if has_description {
            conn.execute_batch(
                "BEGIN;
                 DROP TABLE IF EXISTS skills_tmp;
                 CREATE TABLE skills_tmp (
                    guild_id INTEGER NOT NULL,
                    user_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    normalized_name TEXT NOT NULL,
                    skill_type TEXT NOT NULL,
                    level TEXT NOT NULL,
                    effect TEXT NOT NULL,
                    UNIQUE(guild_id, user_id, normalized_name)
                 );
                 INSERT INTO skills_tmp (guild_id, user_id, name, normalized_name, skill_type, level, effect)
                 SELECT guild_id, user_id, name, normalized_name, COALESCE(skill_type, ''), COALESCE(level, ''), COALESCE(effect, '')
                 FROM skills;
                 DROP TABLE skills;
                 ALTER TABLE skills_tmp RENAME TO skills;
                 COMMIT;",
            )?;
        } else {
            let _ = conn.execute(
                "ALTER TABLE skills ADD COLUMN skill_type TEXT NOT NULL DEFAULT ''",
                [],
            );
            let _ = conn.execute(
                "ALTER TABLE skills ADD COLUMN level TEXT NOT NULL DEFAULT ''",
                [],
            );
        }

        Ok(())
    })
    .await
    .map_err(|e| anyhow!("初始化資料庫失敗: {}", e))?;

    let intents = serenity::GatewayIntents::GUILDS;

    let setup_config = Arc::clone(&shared_config);
    let setup_db = db.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: crate::bot::commands(),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let config = Arc::clone(&setup_config);
            let db = setup_db.clone();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("{} 已經上線!", ready.user.name);
                Ok(BotData { config, db })
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
