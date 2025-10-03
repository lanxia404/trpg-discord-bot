use crate::bot::{Context, Error};
use crate::models::types::RollResult;
use crate::utils::coc::{determine_success_level, format_success_level, roll_coc_multi};
use crate::utils::dice::roll_multiple_dice;
use poise::{CreateReply, serenity_prelude as serenity};

/// D&D 骰子指令 - 擲骰子
#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "骰子表達式 (例如: 2d20+5, d10, 1d6>=15)"] expression: String,
) -> Result<(), Error> {
    let rules = {
        let data = ctx.data();
        let config_handle = data.config.lock().await;
        let guild_id = ctx.guild_id().map(|id| id.get());
        let guild_config = guild_id
            .map(|id| config_handle.get_guild_config(id))
            .unwrap_or_default();
        guild_config.dnd_rules
    };

    match roll_multiple_dice(&expression, rules.max_dice_count, &rules) {
        Ok(results) => {
            if results.len() == 1 {
                send_embed(&ctx, "D&D 擲骰結果", format_roll_result(&results[0])).await?;
            } else {
                send_embed(
                    &ctx,
                    "D&D 連續擲骰結果",
                    format_multiple_roll_results(&results),
                )
                .await?;
            }
        }
        Err(e) => {
            send_embed(&ctx, "D&D 擲骰錯誤", format!("錯誤: {}", e)).await?;
        }
    }

    Ok(())
}

/// CoC 7e 闇黑咆哮指令
#[poise::command(slash_command)]
pub async fn coc(
    ctx: Context<'_>,
    #[description = "技能值 (1-100)"]
    #[min = 1]
    #[max = 100]
    skill: u8,
    #[description = "擲骰次數 (1-10)"]
    #[min = 1]
    #[max = 10]
    times: Option<u8>,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("此指令只能在伺服器中使用").await?;
            return Ok(());
        }
    };

    let rules = {
        let data = ctx.data();
        let config_handle = data.config.lock().await;
        config_handle.get_guild_config(guild_id).coc_rules
    };

    let times = times.unwrap_or(1);
    let results = roll_coc_multi(skill, times, &rules);

    if results.len() == 1 {
        let result = &results[0];
        let success_level = determine_success_level(result.total as u16, skill, &rules);
        let success_text = format_success_level(success_level);
        send_embed(
            &ctx,
            "CoC 7e 擲骰結果",
            format!(
                "技能值: {}\n骰子結果: {}\n判定結果: {}{}",
                skill,
                result.rolls[0],
                success_text,
                if result.is_critical_success {
                    " ✨ 大成功!"
                } else if result.is_critical_fail {
                    " 💥 大失敗!"
                } else {
                    ""
                }
            ),
        )
        .await?;
    } else {
        let mut message = format!("連續擲骰次數: {}\n技能值: {}\n", results.len(), skill);
        for (index, result) in results.iter().enumerate() {
            let success_level = determine_success_level(result.total as u16, skill, &rules);
            let success_text = format_success_level(success_level);
            let crit = if result.is_critical_success {
                " ✨"
            } else if result.is_critical_fail {
                " 💥"
            } else {
                ""
            };
            let status = match result.comparison_result {
                Some(true) => " ✅",
                Some(false) => " ❌",
                None => "",
            };
            message.push_str(&format!(
                "{}. {} → {}{}{}\n",
                index + 1,
                result.rolls[0],
                success_text,
                crit,
                status
            ));
        }
        send_embed(&ctx, "CoC 7e 連續擲骰結果", message).await?;
    }

    Ok(())
}

fn format_roll_result(result: &RollResult) -> String {
    let rolls_str = result
        .rolls
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<String>>()
        .join(" + ");

    let total_with_mod = if result.modifier != 0 {
        format!("({}) + {} = {}", rolls_str, result.modifier, result.total)
    } else {
        format!("{} = {}", rolls_str, result.total)
    };

    let crit_info = if result.is_critical_success {
        " ✨ 大成功!"
    } else if result.is_critical_fail {
        " 💥 大失敗!"
    } else {
        ""
    };

    let comparison_info = match result.comparison_result {
        Some(true) => "✅ 成功 ",
        Some(false) => "❌ 失敗 ",
        None => "",
    };

    format!(
        "🎲 D&D 擲骰: {} = {}{}{}",
        result.dice_expr, total_with_mod, crit_info, comparison_info
    )
}

fn format_multiple_roll_results(results: &[RollResult]) -> String {
    let mut output = String::from("🎲 連續擲骰結果:\n");

    for (i, result) in results.iter().enumerate() {
        let rolls_str = result
            .rolls
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<String>>()
            .join(" + ");

        let total_with_mod = if result.modifier != 0 {
            format!("({}) + {} = {}", rolls_str, result.modifier, result.total)
        } else {
            format!("{} = {}", rolls_str, result.total)
        };

        output.push_str(&format!(
            "{}. {} = {}\n",
            i + 1,
            result.dice_expr,
            total_with_mod
        ));
    }

    output
}

async fn send_embed(
    ctx: &Context<'_>,
    title: impl Into<String>,
    description: String,
) -> Result<(), Error> {
    let title = title.into();
    let embed = serenity::CreateEmbed::default()
        .title(title)
        .description(description)
        .colour(serenity::Colour::BLURPLE);
    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;
    Ok(())
}
