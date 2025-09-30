use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, Context};
use serenity::model::prelude::CommandDataOption;
use crate::utils::dice::roll_multiple_dice;
use crate::utils::coc::{roll_coc, format_success_level};
use crate::utils::config::ConfigManager;
use crate::models::types::RollResult;

pub async fn register_dice_command() -> CreateCommand {
    CreateCommand::new("roll")
        .description("D&D 骰子指令 - 擲骰子")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "expression",
            "骰子表達式 (例如: 2d20+5, d10, 1d6>=15)",
        ).required(true))
}

pub async fn register_coc_command() -> CreateCommand {
    CreateCommand::new("coc")
        .description("CoC 7e 闇黑咆哮指令")
        .add_option(CreateCommandOption::new(
            CommandOptionType::Integer,
            "skill",
            "技能值 (1-100)",
        ).required(true))
}

pub async fn handle_dice_command(
    _ctx: &Context,
    mut command_options: Vec<CommandDataOption>,
    _config_manager: &ConfigManager,
) -> String {
    if command_options.is_empty() {
        return "請提供骰子表達式".to_string();
    }
    
    let option = command_options.remove(0);
    let expression = if let serenity::all::CommandDataOptionValue::String(expr) = option.value {
        expr
    } else {
        return "骰子表達式必須是字串".to_string();
    };

    match roll_multiple_dice(&expression, 50) {
        Ok(results) => {
            if results.len() == 1 {
                format_roll_result(&results[0])
            } else {
                format_multiple_roll_results(&results)
            }
        },
        Err(e) => format!("錯誤: {}", e),
    }
}

pub async fn handle_coc_command(
    _ctx: &Context,
    mut command_options: Vec<CommandDataOption>,
    config_manager: &ConfigManager,
) -> String {
    if command_options.is_empty() {
        return "請提供技能值".to_string();
    }
    
    let option = command_options.remove(0);
    let skill_value = if let serenity::all::CommandDataOptionValue::Integer(skill) = option.value {
        skill as u8
    } else {
        return "技能值必須是整數".to_string();
    };

    if skill_value < 1 || skill_value > 100 {
        return "技能值必須在 1-100 之間".to_string();
    }

    let coc_rules = &config_manager.get_guild_config(0).coc_rules; // Use default rules for now
    let result = roll_coc(skill_value, coc_rules);
    
    let success_level = crate::utils::coc::determine_success_level(result.total as u16, skill_value, coc_rules);
    let success_text = format_success_level(success_level);

    format!(
        "🎯 CoC 7e 擲骰\n技能值: {}\n骰子結果: {}\n判定結果: {}\n{}", 
        skill_value,
        result.rolls[0],
        success_text,
        if result.is_critical_success { " ✨ 大成功!" } else if result.is_critical_fail { " 💥 大失敗!" } else { "" }
    )
}

fn format_roll_result(result: &RollResult) -> String {
    let rolls_str = result.rolls.iter()
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
    
    format!("🎲 D&D 擲骰: {} = {}{}{}", result.dice_expr, total_with_mod, crit_info, comparison_info)
}

fn format_multiple_roll_results(results: &[RollResult]) -> String {
    let mut output = "🎲 連續擲骰結果:\n".to_string();
    
    for (i, result) in results.iter().enumerate() {
        let rolls_str = result.rolls.iter()
            .map(|r| r.to_string())
            .collect::<Vec<String>>()
            .join(" + ");
        
        let total_with_mod = if result.modifier != 0 {
            format!("({}) + {} = {}", rolls_str, result.modifier, result.total)
        } else {
            format!("{} = {}", rolls_str, result.total)
        };
        
        output.push_str(&format!("{}. {} = {}\n", i + 1, result.dice_expr, total_with_mod));
    }
    
    output
}