# TRPG Discord Bot (Rust Version)

這是一個用 Rust 編寫的 Discord 機器人，專為TRPG設計。

## 功能特性

- **擲骰系統**：支援 D&D 和 CoC 7e 擲骰
- **日誌系統**：即時串流和批次日誌記錄
- **管理功能**：重啟機器人、管理開發者等
- **配置管理**：JSON 格式的持久化配置

## 技術特點

- 使用 Rust 編程語言確保內存安全和高性能
- 基於 [poise](https://github.com/serenity-rs/poise) 框架構建，提供現代化的 Slash 指令體驗
- 模塊化設計便於擴展
- 透過 `.env` 管理敏感設定，並內建 JSON 配置持久化

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

### 與系統服務整合

若機器人以 systemd 等服務管理器啟動，可在 `config.json` 的 `global` 區段設定：

```json
{
  "restart_mode": "service",
  "restart_service": "trpg-bot"
}
```

- `restart_mode`：`execv`（預設）會直接以原參數重新啟動程序；`service` 會透過 `systemctl restart/stop` 控制指定服務。
- `restart_service`：當 `restart_mode` 為 `service` 時必填，為 systemd 服務名稱。

完成設定後，`/admin restart` 與 `/admin shutdown` 將呼叫對應的 `systemctl restart/stop`，確保服務受系統管理。

## 指令列表

### 擲骰指令

- `/roll <骰子表達式>` - D&D 擲骰
- `/coc <技能值> [次數]` - CoC 7e 擲骰，支援 1-10 次連續判定
- `/skill add <名稱> <類型> <等級> <效果>` - 新增或更新個人技能
- `/skill show <名稱>` - 支援模糊搜尋技能名稱，查詢自己的技能
- `/skill delete <名稱>` - 刪除此伺服器中符合的技能（含其他玩家），需要按鈕確認

### 日誌指令

- `/log-stream <on|off> [頻道]` - 控制日誌串流開關
- `/log-stream-mode <live|batch>` - 切換串流模式
- `/crit <success|fail> [頻道]` - 設定大成功/大失敗紀錄頻道，紀錄訊息會標註觸發頻道

### 管理指令

- `/admin restart` - 確認後重新啟動機器人
- `/admin shutdown` - 確認後關閉機器人
- `/admin dev-add <用戶>` - 添加開發者（需按鈕確認）
- `/admin dev-remove <用戶>` - 移除開發者（需按鈕確認）
- `/admin dev-list` - 展示開發者列表

### 幫助指令

- `/help [summary|detailed]` - 顯示簡表或完整說明

## 當前狀態

機器人功能完整且可正常運行。

## 未來計劃

1. **更多指令與系統支援**：擴充更多 TRPG 系統與自訂化功能。
2. **測試補強**：加入整合測試確保核心指令穩定性。
3. **性能優化**：進一步優化資源使用。

## 貢獻

歡迎提交 Issue 和 Pull Request 來改進這個項目！

## 許可證

MIT License
