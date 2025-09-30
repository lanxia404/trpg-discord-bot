# TRPG Discord Bot (Rust Version)

這是一個用 Rust 編寫的 Discord 機器人，專為桌上角色扮演游戏(TRPG)設計。

## 功能特性

- **擲骰系統**：支援 D&D 和 CoC 7e 擲骰
- **日誌系統**：即時串流和批次日誌記錄
- **管理功能**：重啟機器人、管理開發者等
- **配置管理**：JSON 格式的持久化配置

## 技術特點

- 使用 Rust 編程語言確保內存安全和高性能
- 基於 Serenity 框架構建
- 模塊化設計便於擴展
- 支持指令前綴 `rpg!`

## 編譯和運行

```bash
# 克隆項目
git clone <repository-url>
cd trpg-discord-bot

# 設置環境變量
echo "DISCORD_TOKEN=your_discord_token_here" > .env

# 編譯
cargo build --release

# 運行
cargo run --release
```

## 指令列表

### 擲骰指令
- `rpg!dnd <骰子表達式>` - D&D 擲骰
- `rpg!roll <骰子表達式>` - 通用擲骰
- `rpg!cc <技能值>` - CoC 7e 擲骰

### 日誌指令
- `rpg!log stream <set/off/mode/throttle> [...]` - 配置日誌串流
- `rpg!log level <INFO|DEBUG|WARN|ERROR>` - 設置日誌級別
- `rpg!log crit <set/off> [...]` - 設置大成功/大失敗記錄頻道

### 管理指令
- `rpg!admin restart` - 重啟機器人
- `rpg!admin dev <add/remove/list> [...]` - 管理開發者
- `rpg!admin rcfg <mode/service/show> [...]` - 配置重啟設置
- `rpg!admin gstream` - 全局串流設置

### 幫助指令
- `rpg!help` - 顯示幫助信息

## 當前狀態

機器人功能完整且可正常運行。編譯時會產生棄用警告，這是因為項目使用了 Serenity 庫的 StandardFramework，該框架在 0.12.x 版本中被標記為棄用，並會在 0.13 版本中移除。

## 未來計劃

1. **遷移到 Poise 框架**：計劃將項目遷移到推薦的 [poise](https://github.com/serenity-rs/poise) 框架，以消除棄用警告並獲得更好的功能支持。
2. **功能擴展**：添加更多 TRPG 系統支持。
3. **性能優化**：進一步優化資源使用。

## 貢獻

歡迎提交 Issue 和 Pull Request 來改進這個項目！

## 許可證

MIT License