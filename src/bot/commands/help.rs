use serenity::all::{CreateCommand, Context};
use serenity::model::prelude::CommandDataOption;
use crate::utils::config::ConfigManager;

pub async fn register_help_command() -> CreateCommand {
    CreateCommand::new("help")
        .description("顯示說明資訊")
}

pub async fn handle_help_command(
    _ctx: &Context,
    _command_options: Vec<CommandDataOption>,
    _config_manager: &ConfigManager,
) -> String {
    "
# TRPG Discord 機器人說明

## 擲骰指令
- `/roll <骰子表達式>` - D&D 骰子指令
  - 例如: `2d20+5`, `d10`, `1d6>=15`
  - 支援連續擲骰: `+3 d6` (擲3次d6)
  
- `/coc <技能值>` - CoC 7e 闇黑咆哮指令
  - 例如: `/coc 65` (技能值65的判定)

## 日誌指令
- `/log-stream-set <頻道>` - 設定日誌串流頻道
- `/log-stream-off` - 關閉日誌串流
- `/log-stream-mode <模式>` - 設定串流模式 (live/batch)

## 管理指令 (僅開發者)
- `/admin restart` - 重啟機器人
- `/admin dev-add <用戶>` - 添加開發者
- `/admin dev-remove <用戶>` - 移除開發者
- `/admin dev-list` - 列出開發者

## 其他指令
- `/help` - 顯示此說明

### D&D 骰子系統
支援常見的骰子表達式格式：
- `2d6` - 擲2顆6面骰
- `d20+5` - 擲1顆20面骰+5
- `1d10>=15` - 擲1顆10面骰，與15比較

### CoC 7e 闇黑咆哮系統
- 大成功: 骰出1
- 極限成功: 骰出 ≤ 技能值/5
- 困難成功: 骰出 ≤ 技能值/2
- 普通成功: 骰出 ≤ 技能值
- 大失敗: 技能<50時96-100，技能≥50時100
    ".to_string()
}