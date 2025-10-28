use crate::bot::{Context, Error};
use poise::{
    serenity_prelude::{
        self as serenity, ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse,
        CreateInteractionResponseMessage, Mentionable,
    },
    ChoiceParameter, CreateReply,
};
use std::time::Duration;
use tokio_rusqlite::{params, OptionalExtension, Result as DbResult};

#[derive(ChoiceParameter, Clone, Copy, Debug)]
pub enum SkillAction {
    #[name = "add"]
    Add,
    #[name = "show"]
    Show,
    #[name = "delete"]
    Delete,
}

struct DbSkill {
    name: String,
    normalized_name: String,
    skill_type: String,
    level: String,
    effect: String,
}

/// 技能資料庫指令
#[poise::command(slash_command)]
pub async fn skill(
    ctx: Context<'_>,
    #[description = "操作 add、show 或 delete"] action: SkillAction,
    #[description = "技能名稱"] name: String,
    #[description = "技能類型 (add 必填)"] skill_type: Option<String>,
    #[description = "技能等級 (add 必填)"] level: Option<String>,
    #[description = "技能效果 (add 必填)"] effect: Option<String>,
) -> Result<(), Error> {
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

    match action {
        SkillAction::Add => {
            let Some(skill_type) = skill_type.filter(|s| !s.trim().is_empty()) else {
                let embed = serenity::CreateEmbed::default()
                    .colour(serenity::Colour::RED)
                    .description("請提供技能類型");
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            };
            let skill_type = skill_type.trim().to_string();

            let Some(level) = level.filter(|s| !s.trim().is_empty()) else {
                let embed = serenity::CreateEmbed::default()
                    .colour(serenity::Colour::RED)
                    .description("請提供技能等級");
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            };
            let level = level.trim().to_string();
            let Some(effect) = effect.filter(|s| !s.trim().is_empty()) else {
                let embed = serenity::CreateEmbed::default()
                    .colour(serenity::Colour::RED)
                    .description("請提供技能效果");
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            };
            let effect = effect.trim().to_string();

            add_skill(&ctx, guild_id, &name, &skill_type, &level, &effect).await?;

            let embed = serenity::CreateEmbed::default()
                .title("技能已儲存")
                .fields([
                    ("名稱", format!("`{}`", name), false),
                    ("類型", skill_type.clone(), true),
                    ("等級", level.clone(), true),
                    ("效果", effect.clone(), false),
                ])
                .colour(serenity::Colour::DARK_GREEN);
            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        SkillAction::Show => {
            let target = ctx.author().clone();
            let record = find_skill_for_user(&ctx, guild_id, &target, &name).await?;

            let search_term = name.trim();
            let embed = match record {
                Some(db_skill) => {
                    let DbSkill {
                        name,
                        normalized_name: _,
                        skill_type,
                        level,
                        effect,
                    } = db_skill;

                    serenity::CreateEmbed::default()
                        .title(format!("技能：<{}>", name))
                        .fields([
                            ("類型", skill_type, true),
                            ("等級", level, true),
                            ("效果", effect, false),
                        ])
                        .colour(serenity::Colour::BLURPLE)
                }
                None => serenity::CreateEmbed::default()
                    .title(format!("技能：<{}>", search_term))
                    .colour(serenity::Colour::ORANGE)
                    .description(format!("找不到技能 `{}`", name)),
            };

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        SkillAction::Delete => {
            let caller = ctx.author().clone();

            let Some(db_skill) = find_skill_in_guild(&ctx, guild_id, &name).await? else {
                let embed = serenity::CreateEmbed::default()
                    .colour(serenity::Colour::ORANGE)
                    .description(format!("找不到此伺服器中的技能 `{}`，無法刪除", name));
                ctx.send(CreateReply::default().embed(embed)).await?;
                return Ok(());
            };

            let confirm_id = format!(
                "skill_delete_confirm:{}:{}",
                guild_id, db_skill.normalized_name
            );
            let cancel_id = format!(
                "skill_delete_cancel:{}:{}",
                guild_id, db_skill.normalized_name
            );
            let components = vec![CreateActionRow::Buttons(vec![
                CreateButton::new(confirm_id.clone())
                    .label("確認刪除")
                    .style(ButtonStyle::Danger),
                CreateButton::new(cancel_id.clone())
                    .label("取消")
                    .style(ButtonStyle::Secondary),
            ])];

            let embed = serenity::CreateEmbed::default()
                .title("確認刪除技能")
                .description(format!(
                    "目標技能：`{}`\n類型：{}\n等級：{}\n效果：{}",
                    &db_skill.name,
                    &db_skill.skill_type,
                    &db_skill.level,
                    &db_skill.effect
                ))
                .colour(serenity::Colour::DARK_RED);

            let reply = CreateReply::default().embed(embed).components(components);

            let sent = ctx.send(reply).await?;
            let mut message = sent.into_message().await?;
            let ctx_clone = ctx.serenity_context().clone();
            let author_id = caller.id;

            let interaction = message
                .await_component_interaction(&ctx_clone)
                .author_id(author_id)
                .timeout(Duration::from_secs(30))
                .await;

            match interaction {
                Some(interaction) if interaction.data.custom_id == confirm_id => {
                    delete_skill(&ctx, guild_id, &db_skill.normalized_name)
                        .await?;

                    let summary = format!("{} 刪除了技能 `{}`", caller.mention(), db_skill.name);

                    let mut response = CreateInteractionResponseMessage::default();
                    response = response.content(summary).components(Vec::new());
                    interaction
                        .create_response(
                            &ctx_clone,
                            CreateInteractionResponse::UpdateMessage(response),
                        )
                        .await?;
                }
                Some(interaction) => {
                    let mut response = CreateInteractionResponseMessage::default();
                    response = response
                        .content(format!("{} 取消刪除操作", caller.mention()))
                        .components(Vec::new());
                    interaction
                        .create_response(
                            &ctx_clone,
                            CreateInteractionResponse::UpdateMessage(response),
                        )
                        .await?;
                }
                None => {
                    let edit = serenity::builder::EditMessage::new()
                        .content("操作逾時，未刪除任何技能")
                        .components(Vec::new());
                    let _ = message.edit(&ctx_clone.http, edit).await;
                }
            }
        }
    }

    Ok(())
}

async fn add_skill(
    ctx: &Context<'_>,
    guild_id: u64,
    name: &str,
    skill_type: &str,
    level: &str,
    effect: &str,
) -> Result<(), Error> {
    let skills_db = ctx.data().skills_db.clone();
    let normalized = name.to_lowercase();
    let name = name.to_string();
    let skill_type = skill_type.to_string();
    let level = level.to_string();
    let effect = effect.to_string();

    skills_db.call(move |conn| {
        conn.execute(
            "INSERT INTO skills (guild_id, name, normalized_name, skill_type, level, effect)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(guild_id, normalized_name)
            DO UPDATE SET name=excluded.name, skill_type=excluded.skill_type, level=excluded.level, effect=excluded.effect",
            params![
                guild_id as i64,
                name,
                normalized,
                skill_type,
                level,
                effect
            ],
        )?;
        Ok(())
    })
    .await?;

    Ok(())
}

async fn find_skill_for_user(
    ctx: &Context<'_>,
    guild_id: u64,
    _user: &serenity::User,
    name: &str,
) -> Result<Option<DbSkill>, Error> {
    let skills_db = ctx.data().skills_db.clone();
    let guild_id_i64 = guild_id as i64;
    let normalized = name.to_lowercase();

    let pattern = format!("%{}%", normalized);

    let result = skills_db
        .call(move |conn| -> DbResult<Option<DbSkill>> {
            let row = conn
                .query_row(
                    "SELECT name, normalized_name, skill_type, level, effect
                FROM skills
                WHERE guild_id = ?1 AND normalized_name LIKE ?2
                ORDER BY CASE WHEN normalized_name = ?3 THEN 0 ELSE 1 END,
                        ABS(LENGTH(normalized_name) - LENGTH(?3)),
                        normalized_name
                LIMIT 1",
                    params![guild_id_i64, pattern, normalized],
                    |row| {
                        Ok(DbSkill {
                            name: row.get(0)?,
                            normalized_name: row.get(1)?,
                            skill_type: row.get(2)?,
                            level: row.get(3)?,
                            effect: row.get(4)?,
                        })
                    },
                )
                .optional()?;
            Ok(row)
        })
        .await?;

    Ok(result)
}

async fn find_skill_in_guild(
    ctx: &Context<'_>,
    guild_id: u64,
    name: &str,
) -> Result<Option<DbSkill>, Error> {
    let skills_db = ctx.data().skills_db.clone();
    let guild_id_i64 = guild_id as i64;
    let normalized = name.to_lowercase();
    let pattern = format!("%{}%", normalized);

    let result = skills_db
        .call(move |conn| -> DbResult<Option<DbSkill>> {
            let row = conn
                .query_row(
                    "SELECT name, normalized_name, skill_type, level, effect
                FROM skills
                WHERE guild_id = ?1 AND normalized_name LIKE ?2
                ORDER BY CASE WHEN normalized_name = ?3 THEN 0 ELSE 1 END,
                        ABS(LENGTH(normalized_name) - LENGTH(?3)),
                        normalized_name
                LIMIT 1",
                    params![guild_id_i64, pattern, normalized],
                    |row| {
                        Ok(DbSkill {
                            name: row.get(0)?,
                            normalized_name: row.get(1)?,
                            skill_type: row.get(2)?,
                            level: row.get(3)?,
                            effect: row.get(4)?,
                        })
                    },
                )
                .optional()?;
            Ok(row)
        })
        .await?;

    Ok(result)
}

async fn delete_skill(
    ctx: &Context<'_>,
    guild_id: u64,
    normalized_name: &str,
) -> Result<(), Error> {
    let skills_db = ctx.data().skills_db.clone();
    let guild_id_i64 = guild_id as i64;
    let normalized = normalized_name.to_string();

    skills_db.call(move |conn| -> DbResult<()> {
        conn.execute(
            "DELETE FROM skills
            WHERE guild_id = ?1 AND normalized_name = ?2",
            params![guild_id_i64, normalized],
        )?;
        Ok(())
    })
    .await?;

    Ok(())
}
