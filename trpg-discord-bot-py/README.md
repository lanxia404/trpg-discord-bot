# TRPG Discord Bot (Python版)

這是一個用 Python 編寫的 Discord 機器人，專為TRPG設計。

## 功能特性

- **擲骰系統**：支援 D&D 和 CoC 7e 擲骰
- **日誌系統**：即時串流和批次日誌記錄
- **管理功能**：重啟機器人、管理開發者等
- **配置管理**：JSON 格式的持久化配置

## 技術特點

- 使用 Python 編程語言
- 基於 discord.py 框架構建
- 模塊化設計便於擴展
- 透過 `.env` 管理敏感設定，並內建 JSON 配置持久化

## 編譯和運行

```bash
# 克隆項目
git clone <repository-url>
cd trpg-discord-bot-py

# 安裝依賴
pip install -r requirements.txt

# 設置環境變量
cp .env.example .env
# 編輯 .env 文件並添加你的 DISCORD_TOKEN

# 運行
python main.py
```

## 指令列表

### 擲骰指令

- `/roll <骰子表達式>` - D&D 擲骰
- `/coc <技能值> [次數]` - CoC 7e 擲骰，支援 1-10 次連續判定
- `/skill add <名稱> <類型> <等級> <效果>` - 新增或更新個人技能
- `/skill show <名稱>` - 支援模糊搜尋技能名稱，查詢自己的技能
- `/skill delete <名稱>` - 刪除此伺服器中符合的技能（含其他玩家），需要按鈕確認

### 日誌指令

- `/log_stream <on|off> [頻道]` - 控制日誌串流開關
- `/log_stream_mode <live|batch>` - 切換串流模式
- `/crit <success|fail> [頻道]` - 設定大成功/大失敗紀錄頻道，紀錄訊息會標註觸發頻道

### 管理指令

- `/admin restart` - 確認後重新啟動機器人
- `/admin shutdown` - 確認後關閉機器人
- `/admin dev_add <用戶>` - 添加開發者（需按鈕確認）
- `/admin dev_remove <用戶>` - 移除開發者（需按鈕確認）
- `/admin dev_list` - 展示開發者列表

### 幫助指令

- `/help` - 顯示指令說明

## 當前狀態

機器人功能完整且可正常運行。

## 貢獻

歡迎提交 Issue 和 Pull Request 來改進這個項目！

## 許可證

MIT License