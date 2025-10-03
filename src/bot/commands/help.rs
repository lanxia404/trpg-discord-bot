use crate::bot::{Context, Error};
use poise::ChoiceParameter;

#[derive(Clone, Copy, Debug, ChoiceParameter)]
pub enum HelpMode {
    #[name = "summary"]
    Summary,
    #[name = "detailed"]
    Detailed,
}

/// 顯示指令快速說明
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "顯示模式"] mode: Option<HelpMode>,
) -> Result<(), Error> {
    match mode.unwrap_or(HelpMode::Summary) {
        HelpMode::Summary => {
            ctx.say(
                "TRPG Discord Bot 指令速覽:\n\
\n\
/roll <表達式> — 擲 D&D 骰子，支援比較與批次擲骰。\n\
/coc <技能值> — CoC 7e 判定並顯示成功等級。\n\
/log-stream <on|off> [頻道] — 控制日誌串流開關。\n\
/log-stream-mode <live|batch> — 切換串流輸出模式。\n\
/admin <restart|dev-add|dev-remove|dev-list> — 開發者專用管理指令。\n\
/help [summary|detailed] — 顯示這份簡表或詳細版。",
            )
            .await?;
        }
        HelpMode::Detailed => {
            ctx.say(
                r#"
# TRPG Discord Bot 說明

## 擲骰
- `/roll <骰子表達式>`：一般 D&D 擲骰，支援：
  - 數量/面數：`2d6`、`d20`
  - 修正值：`1d20+5`
  - 比較：`1d10>=15`
  - 批次：`+3 d6`（連續擲 3 次）
- `/coc <技能值>`：CoC 7e 判定，自動回報成功等級、極限/困難成功與大成/失敗。

## 日誌控制
- `/log-stream on <頻道>`：啟用串流並綁定文字頻道。
- `/log-stream off`：關閉串流輸出。
- `/log-stream-mode <live|batch>`：切換即時或批次模式。

## 管理（僅開發者）
- `/admin restart`：發出重啟指令（目前為提示）。
- `/admin dev-add <用戶>` / `/admin dev-remove <用戶>`：維護開發者名單。
- `/admin dev-list`：列出所有已註冊開發者。

## 其他
- `/help [summary|detailed]`：切換本說明的摘要或完整內容。
                "#,
            )
            .await?;
        }
    }

    Ok(())
}
