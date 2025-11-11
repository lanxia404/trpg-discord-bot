# TRPG Discord Bot (Rust Version)

這是一個使用 Rust 編寫的專業 TRPG（桌上角色扮演遊戲）Discord 機器人，旨在提升 TRPG 遊戲體驗。機器人整合了多種 TRPG 系統支持、AI 聊天功能、先進的 RAG 記憶系統和全面的遊戲管理功能。

## 功能特色

### 🎲 擲骰系統

- **D&D 擲骰**：支援各種 D&D 骰子表達式，如 `1d20+5`, `2d6`, `d100` 等
- **CoC 7e 擲骰**：支援克蘇魯的呼喚第 7 版技能判定，可進行 1-10 次連續判定
- **自訂技能管理**：玩家可以新增、查詢、刪除個人技能，支援職業和種族分類

### 🧠 AI 聊天與 RAG 記憶系統

- **多 LLM 支援**：支援 OpenAI、OpenRouter、Anthropic、Google 等多種 API 提供商
- **對話歷史**：智能對話上下文管理，被提及時自動載入 50 條初始歷史消息，使用 100 條消息作為 AI 回應上下文
- **RAG 記憶系統**：基於向量相似度的語意搜尋，實現檢索增強生成 (RAG)
  - 支援本地計算、嵌入 API 和向量數據庫三種向量計算方式
  - 可保存、搜尋、列出和刪除個人記憶條目
  - 支援標籤分類和管理
- **標籤分類**：支援標籤系統方便分類和檢索記憶

### 📚 資料庫與搜尋

- **基礎設定搜尋**：支援搜尋預載的 TRPG 資料庫，支援多表搜尋和模糊匹配
- **資料導入功能**：支援從 CSV、XLSX、JSON 等多種格式導入資料
- **SQLite 存儲**：高效能的本地數據庫支持

### 📝 日誌與追蹤

- **即時日誌串流**：支援即時和批次日誌記錄，自動抑制重複和噪音日誌
- **大成功/大失敗追蹤**：專門的關鍵結果記錄功能
- **劇情連貫性**：結合 RAG 記憶系統維持劇情的連貫性

### ⚙️ 管理與設定

- **斜線指令**：基於 poise 框架的直觀用戶界面
- **配置管理**：JSON 格式的持久化配置，支援全局和伺服器級設定
- **管理員功能**：重啟、關閉機器人，管理開發者列表

## 快速開始

### 環境要求

- Rust 1.85 或更高版本
- Cargo
- 有效的 Discord 機器人 Token

### 安裝與設定

1. **克隆項目**

   ```bash
   git clone <repository-url>
   cd trpg-discord-bot
   ```

2. **設定環境變數**

   ```bash
   cp .env.example .env
   # 編輯 .env 文件並填入您的 DISCORD_TOKEN 和其他 API 金鑰
   ```

3. **編譯項目**

   ```bash
   cargo build --release
   ```

4. **運行機器人**

   ```bash
   cargo run --release
   ```

### 環境變數配置

在 `.env` 文件中設定以下變數：

```bash
# 必需
DISCORD_TOKEN=your_discord_token_here

# 可選：API 金鑰
OPENROUTER_API_KEY=your_openrouter_api_key_here
OPENAI_API_KEY=your_openai_api_key_here
ANTHROPIC_API_KEY=your_anthropic_api_key_here
GOOGLE_API_KEY=your_google_api_key_here
CUSTOM_API_KEY=your_custom_api_key_here
```

## 機器人指令

### 擲骰指令

- `/roll <骰子表達式>` - D&D 擲骰，支援 `1d20+5`、`2d6`、`d100` 等格式
- `/coc <技能值> [次數]` - CoC 7e 擲骰，支援 1-10 次連續判定
- `/skill add <名稱> <類型> <等級> <效果> [職業] [種族]` - 新增或更新個人技能
- `/skill show <名稱>` - 搜尋技能（支援模糊搜尋）
- `/skill delete <名稱>` - 刪除技能（需確認）

### 記憶系統指令

- `/memory save content:"<內容>" [tags:"標籤1,標籤2"]` - 保存記憶條目
- `/memory search query:"<搜尋內容>" [max_results:5]` - 語意搜尋記憶
- `/memory list [page:1]` - 列出個人記憶條目
- `/memory delete id:<ID>` - 刪除特定記憶
- `/memory clear confirm:"confirm"` - 清除所有個人記憶
- `/memory toggle enabled:<true/false>` - 開啟/關閉記憶功能
- `/memory setvector method:<local/embeddingapi/vectordb>` - 設定向量計算方式
- `/memory setvector method:<true/false>` - 設定向量存儲方法（true表示EmbeddingApi，false表示Local）

### 聊天與 AI 指令

- `/chat add api_url:<API_URL> api_key:<API_KEY> model:<MODEL_NAME>` - 添加 API 配置
- `/chat remove` - 移除當前 API 配置
- `/chat toggle` - 開啟/關閉 API 功能
- `/chat list-models` - 列出可用模型

### 日誌指令

- `/log-stream <on|off> [頻道]` - 控制日誌串流開關
- `/log-stream-mode <live|batch>` - 切換串流模式
- `/crit <success|fail> [頻道]` - 設定大成功/大失敗記錄頻道

### 管理指令

- `/admin restart` - 重啟機器人
- `/admin shutdown` - 關閉機器人
- `/admin dev-add <用戶>` - 添加開發者
- `/admin dev-remove <用戶>` - 移除開發者
- `/admin dev-list` - 列出開發者

### 其他指令

- `/bs-search [搜尋關鍵字]` - 搜尋基礎設定資料庫
- `/help [summary|detailed]` - 顯示指令說明

## API 整合

機器人支援多種 API 提供商：

### OpenAI

```txt
/chat add api_url:https://api.openai.com/v1/chat/completions api_key:<your_key> model:gpt-4o
```

### OpenRouter

```txt
/chat add api_url:https://openrouter.ai/api/v1/chat/completions api_key:<your_key> model:openai/gpt-4o
```

### 其他提供商

支援任何兼容 OpenAI API 格式的服務，只需提供正確的 API URL、密鑰和模型名稱。

## 記憶系統詳解

機器人的記憶系統是其核心功能之一，專為 TRPG 場景設計：

### 向量計算方式

1. **本地計算（Local）**：使用簡化 TF-IDF 算法，保護隱私，適合日常使用
2. **嵌入 API（Embedding API）**：使用專業 API，提供更高精度的語意理解
3. **向量數據庫（Vector DB）**：支援專業向量數據庫，適用大規模數據場景

### RAG (Retrieval-Augmented Generation) 實現

機器人採用檢索增強生成 (RAG) 架構，將記憶系統與 AI 回應結合：

1. **記憶檢索**：使用向量相似度搜尋相關記憶
2. **上下文增強**：將檢索到的記憶條目與對話歷史作為 AI 的額外上下文
3. **增強回應**：AI 基於原始提示和檢索到的上下文生成更準確、更連貫的回應

### TRPG 應用場景

- **角色關係追蹤**：記錄複雜的角色關係和互動
- **劇情連貫性**：維持劇情的連貫性和一致性
- **世界觀設定**：保存和管理遊戲世界的詳細設定
- **任務追蹤**：記錄任務進度和相關細節

## 系統服務整合

若機器人以系統服務管理，可在 `config.json` 中設定：

```json
{
  "global": {
    "restart_mode": "service",
    "restart_service": "trpg-bot"
  }
}
```

- `restart_mode`：`execv`（默認）或 `service`
- `restart_service`：服務名稱（當 `restart_mode` 為 `service` 時必填）

根據平台自動使用對應服務管理工具：

- **Linux**：`systemctl`
- **Windows**：`sc`

## 開發與貢獻

### 架構概覽

- **框架**：使用 [poise](https://github.com/serenity-rs/poise) 框架提供斜線指令支持
- **語言**：使用 Rust 確保內存安全和高性能
- **非同步**：基於 Tokio 的非同步架構，支援高並發操作
- **數據庫**：SQLite 用於數據持久化（skills.db, base_settings.db, memory.db）
- **API**：支援多種 LLM 提供商的 OpenAI 兼容接口

### 貢獻指南

1. Fork 項目
2. 創建功能分支
3. 提交更改
4. 發起 Pull Request

### 開發進度

- ✅ 核心功能穩定
- ✅ 記憶系統實現
- ✅ 多 API 提供商支援
- 🔄 測試套件補強
- 🔄 性能優化

## 許可證

MIT License

## 支援與反饋

如遇到問題或需要幫助，請：

- 提交 Issue
- 查閱 Wiki 文檔
- 參與社區討論

---

由 Rust 驅動，為 TRPG 愛好者打造的全方位遊戲助手。無論您是主持人還是玩家，都能在這裡找到提升遊戲體驗的工具。
