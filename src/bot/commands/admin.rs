use crate::bot::{Context, Error};
use poise::{
    ChoiceParameter, CreateReply,
    serenity_prelude::{
        self as serenity, ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
};
use rand::random;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone, Copy, Debug, ChoiceParameter)]
pub enum AdminAction {
    #[name = "restart"]
    Restart,
    #[name = "shutdown"]
    Shutdown,
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

    let has_permission = {
        let config_manager = ctx.data().config.lock().await;
        config_manager.is_developer(caller_id)
    };

    if !has_permission {
        ctx.say("您沒有權限執行此操作！").await?;
        return Ok(());
    }

    match action {
        AdminAction::Restart => {
            if !confirm_action(&ctx, "確認執行重啟操作？").await? {
                return Ok(());
            }
            ctx.say("已確認，機器人即將重新啟動……").await?;
            schedule_restart().await?;
        }
        AdminAction::Shutdown => {
            if !confirm_action(&ctx, "確認關閉機器人？").await? {
                return Ok(());
            }
            ctx.say("已確認，機器人即將關閉……").await?;
            schedule_shutdown().await?;
        }
        AdminAction::DevAdd => {
            let user = match user {
                Some(u) => u,
                None => {
                    ctx.say("請指定要添加的用戶！").await?;
                    return Ok(());
                }
            };

            if !confirm_action(&ctx, format!("確認將 <@{}> 新增為開發者？", user.id)).await?
            {
                return Ok(());
            }

            let mut config_manager = ctx.data().config.lock().await;
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

            if !confirm_action(&ctx, format!("確認將 <@{}> 從開發者列表移除？", user.id)).await?
            {
                return Ok(());
            }

            let mut config_manager = ctx.data().config.lock().await;
            if config_manager.remove_developer(user.id.get())? {
                ctx.say(format!("用戶 <@{}> 已從開發者列表移除", user.id))
                    .await?;
            } else {
                ctx.say(format!("用戶 <@{}> 不在開發者列表中", user.id))
                    .await?;
            }
        }
        AdminAction::DevList => {
            let config_manager = ctx.data().config.lock().await;
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

async fn confirm_action(ctx: &Context<'_>, prompt: impl Into<String>) -> Result<bool, Error> {
    let prompt = prompt.into();
    let nonce: u64 = random();
    let confirm_id = format!("admin_confirm:{}:{}", ctx.author().id, nonce);
    let cancel_id = format!("admin_cancel:{}:{}", ctx.author().id, nonce);
    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(confirm_id.clone())
            .label("確認")
            .style(ButtonStyle::Primary),
        CreateButton::new(cancel_id.clone())
            .label("取消")
            .style(ButtonStyle::Secondary),
    ])];

    let reply = CreateReply::default()
        .content(prompt)
        .components(components)
        .ephemeral(true);
    let sent = ctx.send(reply).await?;
    let mut message = sent.into_message().await?;
    let ctx_clone = ctx.serenity_context().clone();
    let author_id = ctx.author().id;

    let interaction = message
        .await_component_interaction(&ctx_clone)
        .author_id(author_id)
        .timeout(Duration::from_secs(30))
        .await;

    match interaction {
        Some(interaction) if interaction.data.custom_id == confirm_id => {
            let mut response = CreateInteractionResponseMessage::default();
            response = response.content("已確認").components(Vec::new());
            interaction
                .create_response(
                    &ctx_clone,
                    CreateInteractionResponse::UpdateMessage(response),
                )
                .await?;
            Ok(true)
        }
        Some(interaction) => {
            let mut response = CreateInteractionResponseMessage::default();
            response = response.content("操作已取消").components(Vec::new());
            interaction
                .create_response(
                    &ctx_clone,
                    CreateInteractionResponse::UpdateMessage(response),
                )
                .await?;
            Ok(false)
        }
        None => {
            let edit = serenity::builder::EditMessage::new()
                .content("操作逾時，未執行任何變更")
                .components(Vec::new());
            let _ = message.edit(&ctx_clone.http, edit).await;
            Ok(false)
        }
    }
}

async fn schedule_restart() -> Result<(), Error> {
    tokio::spawn(async move {
        sleep(Duration::from_millis(500)).await;
        restart_process();
    });
    Ok(())
}

async fn schedule_shutdown() -> Result<(), Error> {
    tokio::spawn(async {
        sleep(Duration::from_millis(500)).await;
        std::process::exit(0);
    });
    Ok(())
}

fn restart_process() -> ! {
    use std::ffi::OsString;

    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Restart failed: unable to resolve executable path: {}", err);
            std::process::exit(1);
        }
    };
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let err = std::process::Command::new(&exe).args(&args).exec();
        eprintln!("Restart failed: {}", err);
        std::process::exit(1);
    }

    #[cfg(not(unix))]
    {
        match std::process::Command::new(&exe).args(&args).spawn() {
            Ok(_) => std::process::exit(0),
            Err(err) => {
                eprintln!("Restart failed: {}", err);
                std::process::exit(1);
            }
        }
    }
}
