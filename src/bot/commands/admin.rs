use crate::bot::{Context, Error};
use crate::models::types::GlobalConfig;
use poise::{
    serenity_prelude::{
        self as serenity, ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    ChoiceParameter, CreateReply,
};
use rand::random;
use std::ffi::OsString;
use std::time::Duration;
use tokio::{process::Command as TokioCommand, time::sleep};

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
            let control = match process_control_from_config(&ctx).await {
                Ok(control) => control,
                Err(msg) => {
                    ctx.say(msg).await?;
                    return Ok(());
                }
            };
            ctx.say("已確認，機器人即將重新啟動……").await?;
            schedule_restart(control).await?;
        }
        AdminAction::Shutdown => {
            if !confirm_action(&ctx, "確認關閉機器人？").await? {
                return Ok(());
            }
            let control = match process_control_from_config(&ctx).await {
                Ok(control) => control,
                Err(msg) => {
                    ctx.say(msg).await?;
                    return Ok(());
                }
            };
            ctx.say("已確認，機器人即將關閉……").await?;
            schedule_shutdown(control).await?;
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

async fn schedule_restart(control: ProcessControl) -> Result<(), Error> {
    tokio::spawn(async move {
        sleep(Duration::from_millis(500)).await;
        if let Err(err) = perform_restart(control).await {
            eprintln!("Restart failed: {}", err);
        }
    });
    Ok(())
}

async fn schedule_shutdown(control: ProcessControl) -> Result<(), Error> {
    tokio::spawn(async move {
        sleep(Duration::from_millis(500)).await;
        if let Err(err) = perform_shutdown(control).await {
            eprintln!("Shutdown failed: {}", err);
        }
    });
    Ok(())
}

async fn process_control_from_config(ctx: &Context<'_>) -> Result<ProcessControl, String> {
    let config_manager = ctx.data().config.lock().await;
    map_global_to_control(&config_manager.global)
}

fn map_global_to_control(global: &GlobalConfig) -> Result<ProcessControl, String> {
    match global.restart_mode.as_str() {
        "spawn" => Ok(ProcessControl::Spawn),
        "service" => {
            let name = global
                .restart_service
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string());
            if let Some(name) = name {
                Ok(ProcessControl::Service { name })
            } else {
                Err("設定錯誤：restart_service 尚未設定，無法操作服務".to_string())
            }
        }
        _ => Ok(ProcessControl::Execv),
    }
}

async fn perform_restart(control: ProcessControl) -> Result<(), String> {
    match control {
        ProcessControl::Execv => restart_with_exec(),
        ProcessControl::Spawn => restart_with_spawn(),
        ProcessControl::Service { name } => {
            control_service(&name, ServiceAction::Restart).await?;
            std::process::exit(0);
            #[allow(unreachable_code)]
            Ok(())
        }
    }
}

async fn perform_shutdown(control: ProcessControl) -> Result<(), String> {
    match control {
        ProcessControl::Service { name } => {
            control_service(&name, ServiceAction::Stop).await?;
            std::process::exit(0);
            #[allow(unreachable_code)]
            Ok(())
        }
        ProcessControl::Execv | ProcessControl::Spawn => {
            std::process::exit(0);
            #[allow(unreachable_code)]
            Ok(())
        }
    }
}

fn restart_with_exec() -> Result<(), String> {
    #[cfg(target_family = "unix")]
    {
        let (exe, args) = current_command()?;
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(exe).args(args).exec();
        Err(format!("process replacement failed: {}", err))
    }

    #[cfg(not(target_family = "unix"))]
    {
        restart_with_spawn()
    }
}

fn restart_with_spawn() -> Result<(), String> {
    let (exe, args) = current_command()?;
    std::process::Command::new(exe)
        .args(args)
        .spawn()
        .map_err(|err| format!("failed to spawn replacement process: {}", err))?;
    std::process::exit(0);
    #[allow(unreachable_code)]
    Ok(())
}

fn current_command() -> Result<(std::path::PathBuf, Vec<OsString>), String> {
    let exe = std::env::current_exe()
        .map_err(|err| format!("unable to resolve executable path: {}", err))?;
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    Ok((exe, args))
}

#[derive(Clone)]
enum ProcessControl {
    Execv,
    Spawn,
    Service { name: String },
}

#[derive(Clone, Copy)]
enum ServiceAction {
    Restart,
    Stop,
}

async fn control_service(name: &str, action: ServiceAction) -> Result<(), String> {
    #[cfg(target_family = "windows")]
    {
        async fn run_sc(subcommand: &str, name: &str) -> Result<(), String> {
            let status = TokioCommand::new("sc")
                .arg(subcommand)
                .arg(name)
                .status()
                .await
                .map_err(|err| format!("failed to run sc {} {}: {}", subcommand, name, err))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "sc {} {} exited with status {:?}",
                    subcommand, name, status
                ))
            }
        }

        match action {
            ServiceAction::Restart => {
                run_sc("stop", name).await?;
                run_sc("start", name).await?;
            }
            ServiceAction::Stop => {
                run_sc("stop", name).await?;
            }
        }
        return Ok(());
    }

    #[cfg(target_family = "unix")]
    {
        let verb = match action {
            ServiceAction::Restart => "restart",
            ServiceAction::Stop => "stop",
        };

        let status = TokioCommand::new("systemctl")
            .arg(verb)
            .arg(name)
            .status()
            .await
            .map_err(|err| format!("failed to execute systemctl {} {}: {}", verb, name, err))?;

        if status.success() {
            return Ok(());
        }

        return Err(format!(
            "systemctl {} {} exited with status {:?}",
            verb, name, status
        ));
    }

    #[cfg(not(any(target_family = "unix", target_family = "windows")))]
    {
        let _ = name;
        let _ = action;
        Err("service control is not supported on this platform".to_string())
    }
}
