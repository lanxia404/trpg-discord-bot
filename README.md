# TRPG Discord Bot (Rust Version)

這是一個用 Rust 編寫的 Discord 機器人，專為TRPG設計。

## 功能特性

- **擲骰系統**：支援 D&D 和 CoC 7e 擲骰
- **日誌系統**：即時串流和批次日誌記錄，自動抑制重複和噪音日誌
- **管理功能**：重啟機器人、管理開發者等
- **配置管理**：JSON 格式的持久化配置
- **資料庫搜尋**：支援搜尋基礎設定資料庫，並可查看特定資料表內容
- **技能系統**：支援新增、查詢、刪除個人技能
- **資料導入**：支援從多種格式（CSV、XLSX、JSON 等）導入資料

## 特點

- 使用 Rust 編程語言確保內存安全和高性能
- 基於 [poise](https://github.com/serenity-rs/poise) 框架構建，提供斜線指令支援
- 模塊化設計利於擴展
- 透過 `.env` 管理敏感設定，並內建 JSON 配置持久化，~~不像某個只會用AI的國中生，還說什麼.json比.env安全~~
- 內建 SQLite 資料庫支援，可用於技能和基礎設定管理
- 非同步架構支援高並發操作
- 豐富的 UI 互動（按鈕、選單）提升使用者體驗

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

若機器人以服務管理器啟動，可在 `config.json` 的 `global` 區段設定：

```json
{
  "restart_mode": "service",
  "restart_service": "trpg-bot"
}
```

- `restart_mode`：`execv`（預設）會嘗試重新啟動程序（Unix 使用 execv，Windows 會啟動新實例後退出）；`service` 會透過平台特定的服務管理工具控制指定服務。
- `restart_service`：當 `restart_mode` 為 `service` 時必填，為服務名稱。

根據平台不同，機器人會使用對應的服務管理工具：

- **Linux**：使用 `systemctl restart/stop` 控制服務
- **Windows**：使用 `sc stop/start` 控制服務

完成設定後，`/admin restart` 與 `/admin shutdown` 將呼叫對應的平台服務管理命令，確保服務受系統管理。

## 指令列表

### 擲骰指令

- `/roll <骰子表達式>` - D&D 擲骰
- `/coc <技能值> [次數]` - CoC 7e 擲骰，支援 1-10 次連續判定
- `/skill add <名稱> <類型> <等級> <效果> [職業] [種族]` - 新增或更新個人技能
- `/skill show <名稱>` - 支援模糊搜尋技能名稱、類型、等級、職業、種族，查詢自己的技能
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

### 資料庫指令

- `/bs-search [搜尋關鍵字]` - 搜尋基礎設定資料庫，會顯示資料表選單讓您選擇，支援在選定的資料表中搜尋關鍵字。若搜尋結果只有一筆會強調顯示，多筆結果支援翻頁瀏覽，並可點擊按鈕查看單筆資料的詳細內容。

## 當前狀態

機器人功能完整且可正常運行。

## 未來計劃

1. **更多指令與系統支援**：擴充更多 TRPG 系統與自訂化功能。
2. **測試補強**：加入整合測試確保核心指令穩定性。
3. **性能優化**：進一步優化資源使用。
4. **擴充資料庫功能**：提供更多資料表操作指令。
5. **增強搜尋功能**：支援更複雜的搜尋和過濾條件。

## 貢獻

歡迎提交 Issue 和 Pull Request 來改進這個項目！

## 許可證

MIT License
