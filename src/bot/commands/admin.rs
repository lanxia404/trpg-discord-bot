use crate::bot::{Context, Error};
use poise::ChoiceParameter;
use poise::serenity_prelude as serenity;

#[derive(Clone, Copy, Debug, ChoiceParameter)]
pub enum AdminAction {
    #[name = "restart"]
    Restart,
    #[name = "dev-add"]
    DevAdd,
    #[name = "dev-remove"]
    DevRemove,
    #[name = "dev-list"]
    DevList,
}

/// 管理指令
#[poise::command(slash_command)]
pub async fn admin(
    ctx: Context<'_>,
    #[description = "管理操作"] action: AdminAction,
    #[description = "要添加或移除的開發者"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let caller_id = ctx.author().id.get();

    let mut config_manager = ctx.data().config.lock().await;
    if !config_manager.is_developer(caller_id) {
        ctx.say("您沒有權限執行此操作！").await?;
        return Ok(());
    }

    match action {
        AdminAction::Restart => {
            ctx.say("機器人重啟功能已觸發（實際上不會重啟）").await?;
        }
        AdminAction::DevAdd => {
            let user = match user {
                Some(u) => u,
                None => {
                    ctx.say("請指定要添加的用戶！").await?;
                    return Ok(());
                }
            };

            if config_manager.add_developer(user.id.get())? {
                ctx.say(format!("用戶 <@{}> 已添加到開發者列表", user.id))
                    .await?;
            } else {
                ctx.say(format!("用戶 <@{}> 已經是開發者", user.id)).await?;
            }
        }
        AdminAction::DevRemove => {
            let user = match user {
                Some(u) => u,
                None => {
                    ctx.say("請指定要移除的用戶！").await?;
                    return Ok(());
                }
            };

            if config_manager.remove_developer(user.id.get())? {
                ctx.say(format!("用戶 <@{}> 已從開發者列表移除", user.id))
                    .await?;
            } else {
                ctx.say(format!("用戶 <@{}> 不在開發者列表中", user.id))
                    .await?;
            }
        }
        AdminAction::DevList => {
            let developers = &config_manager.global.developers;
            if developers.is_empty() {
                ctx.say("目前沒有開發者").await?;
            } else {
                let mut list = String::from("開發者列表:\n");
                for dev_id in developers {
                    list.push_str(&format!("<@{}>\n", dev_id));
                }
                ctx.say(list).await?;
            }
        }
    }

    Ok(())
}
