use crate::bot::{Context, Error};
use crate::models::types::RollResult;
use crate::utils::coc::{determine_success_level, format_success_level, roll_coc};
use crate::utils::dice::roll_multiple_dice;

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
                ctx.say(format_roll_result(&results[0])).await?;
            } else {
                ctx.say(format_multiple_roll_results(&results)).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("錯誤: {}", e)).await?;
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

    let result = roll_coc(skill, &rules);
    let success_level = determine_success_level(result.total as u16, skill, &rules);
    let success_text = format_success_level(success_level);

    ctx.say(format!(
        "🎯 CoC 7e 擲骰\n技能值: {}\n骰子結果: {}\n判定結果: {}{}",
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
    ))
    .await?;

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
