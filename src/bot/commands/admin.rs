use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, Context};
use serenity::model::prelude::CommandDataOption;
use crate::utils::config::ConfigManager;

pub async fn register_admin_command() -> CreateCommand {
    CreateCommand::new("admin")
        .description("管理指令")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "action",
            "管理操作",
        )
        .add_string_choice("restart", "restart")
        .add_string_choice("dev-add", "dev-add")
        .add_string_choice("dev-remove", "dev-remove")
        .add_string_choice("dev-list", "dev-list")
        .required(true))
        .add_option(CreateCommandOption::new(
            CommandOptionType::User,
            "user",
            "要添加或移除的開發者",
        ))
}

pub async fn handle_admin_command(
    _ctx: &Context,
    mut command_options: Vec<CommandDataOption>,
    config_manager: &mut ConfigManager,
    user_id: u64,
) -> String {
    if command_options.is_empty() {
        return "請指定操作".to_string();
    }
    
    let action_option = command_options.remove(0);
    let action = if let serenity::all::CommandDataOptionValue::String(act) = action_option.value {
        act.to_lowercase()
    } else {
        return "操作必須是字串".to_string();
    };

    // Check if user is a developer
    if !config_manager.is_developer(user_id) {
        return "您沒有權限執行此操作！".to_string();
    }

    match action.as_str() {
        "restart" => {
            // In a real implementation, we would handle the restart logic here
            // For now, we just return a message
            "機器人重啟功能已觸發（實際上不會重啟）".to_string()
        },
        "dev-add" => {
            if command_options.is_empty() {
                return "請指定要添加的用戶！".to_string();
            }
            
            let user_option = command_options.remove(0);
            let user_id_value = if let serenity::all::CommandDataOptionValue::User(user_id) = user_option.value {
                user_id
            } else {
                return "參數必須是用戶".to_string();
            };
                
            config_manager.add_developer(user_id_value.get());
            
            if let Err(e) = config_manager.save_config() {
                return format!("開發者列表保存失敗: {}", e);
            }
            
            format!("用戶 <@{}> 已添加到開發者列表", user_id_value.get())
        },
        "dev-remove" => {
            if command_options.is_empty() {
                return "請指定要移除的用戶！".to_string();
            }
            
            let user_option = command_options.remove(0);
            let user_id_value = if let serenity::all::CommandDataOptionValue::User(user_id) = user_option.value {
                user_id
            } else {
                return "參數必須是用戶".to_string();
            };
                
            config_manager.remove_developer(user_id_value.get());
            
            if let Err(e) = config_manager.save_config() {
                return format!("開發者列表保存失敗: {}", e);
            }
            
            format!("用戶 <@{}> 已從開發者列表移除", user_id_value.get())
        },
        "dev-list" => {
            let developers = &config_manager.global.developers;
            if developers.is_empty() {
                "目前沒有開發者".to_string()
            } else {
                let mut list = "開發者列表:\n".to_string();
                for dev_id in developers {
                    // In a real implementation, we would fetch user names
                    list.push_str(&format!("<@{}>\n", dev_id));
                }
                list
            }
        },
        _ => "無效的管理操作！".to_string(),
    }
}