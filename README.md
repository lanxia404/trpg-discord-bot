# TRPG Discord Bot

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

一個功能完整的 TRPG Discord 機器人，使用 Rust 編寫。支援 D&D/CoC 擲骰、AI 對話、RAG 記憶系統和技能管理。

**專案規模**：8.4k 行代碼 · 27 個模組 · 16+ 指令

## 功能特色

### 🎲 擲骰系統

- **D&D**：`1d20+5`, `1d20adv`, `1d20>=15` 等表達式
- **CoC 7e**：技能判定（大成功、極難、困難、普通、失敗、大失敗）
- **技能管理**：個人技能數據庫，支援模糊搜尋

### 🧠 AI 對話與記憶

- **多 LLM 支援**：OpenAI、Anthropic、Google 等
- **RAG 記憶系統**：向量語義搜尋（837 行實現）
- **自定義提示詞**：完全可配置的 AI 角色
- **智能上下文**：可調整記憶檢索（3-20 條）、歷史範圍（5-50 條）、Token 預算（50%-90%）

### 📚 數據管理

- **多格式導入**：CSV、Excel、JSON（1,043 行實現）
- **資料庫搜尋**：多表聯合搜尋、模糊匹配
- **異步存儲**：三個 SQLite 數據庫，自動權限檢查

### 📝 其他功能

- 智能日誌系統、大成功/失敗追蹤
- 對話總結、效果查詢
- 配置管理、熱重載

## 快速開始

### 環境要求

- Rust 1.85+
- Discord Bot Token

### 安裝

```bash
# 克隆專案
git clone <repository-url>
cd trpg-discord-bot

# 設定環境變數
cp .env.example .env
# 編輯 .env 填入 DISCORD_TOKEN

# 編譯運行
cargo build --release
cargo run --release
```

### 環境變數

```bash
DISCORD_TOKEN=your_discord_token        # 必需
OPENAI_API_KEY=your_key                 # 可選
ANTHROPIC_API_KEY=your_key              # 可選
GOOGLE_API_KEY=your_key                 # 可選
```

## 指令列表

### 核心指令

| 指令      | 功能      | 範例                                |
| --------- | --------- | ----------------------------------- |
| `/dice`   | 擲骰系統  | `/dice roll 1d20+5`                 |
| `/skill`  | 技能管理  | `/skill show 劍術`                  |
| `/memory` | 記憶系統  | `/memory action:save content:"..."` |
| `/prompt` | AI 提示詞 | `/prompt set prompt:"..."`          |
| `/chat`   | API 配置  | `/chat add name:openai ...`         |

### 完整指令參考

#### 指令詳細列表

##### 擲骰

- `/dice roll <表達式>` - D&D 擲骰
- `/dice coc <技能值> [次數]` - CoC 擲骰

##### 技能管理

- `/skill add` - 新增技能
- `/skill show <名稱>` - 搜尋技能
- `/skill delete <名稱>` - 刪除技能

##### 記憶系統

- `/memory action:save content:"..." [tags:"..."]` - 保存記憶
- `/memory action:search content:"..." [max_results:5]` - 搜尋記憶
- `/memory action:list [page:1]` - 列出記憶
- `/memory action:delete id:<ID>` - 刪除記憶
- `/memory action:toggle enabled:<true/false>` - 開關記憶
- `/memory action:vector method:<api/local>` - 設定向量計算

##### AI 提示詞

- `/prompt set prompt:"..."` - 設置自定義提示詞
- `/prompt reset` - 重置為預設
- `/prompt view` - 查看當前提示詞
- `/prompt context [ratio] [max_memory] [max_history]` - 配置上下文

##### API 管理

- `/chat add name:<名稱> api_url:<URL> model:<模型>` - 添加 API
- `/chat remove name:<名稱>` - 移除 API
- `/chat set_active name:<名稱>` - 設置活躍 API
- `/chat list` - 列出所有 API

##### 其他

- `/bs-search [query]` - 搜尋資料庫
- `/effect keyword:<關鍵字>` - 搜尋效果
- `/import_data type:<格式>` - 導入數據
- `/crit kind:<success/fail>` - 設定大成功/失敗頻道
- `/admin` - 管理功能
- `/help` - 幫助
- `/summarize [limit]` - 總結對話

## 配置範例

### API 配置

```bash
# 添加 OpenAI
/chat add name:openai api_url:https://api.openai.com/v1/chat/completions model:gpt-4o

# 添加 OpenRouter
/chat add name:openrouter api_url:https://openrouter.ai/api/v1/chat/completions model:openai/gpt-4o
```

### 系統提示詞

```bash
# 設置自定義提示詞
/prompt set prompt:"你是一位經驗豐富的 D&D GM，擅長營造氛圍..."

# 調整上下文
/prompt context ratio:0.8 max_memory:15 max_history:40
```

### config.json 結構

```json
{
  "guilds": {
    "YOUR_GUILD_ID": {
      "custom_system_prompt": "你是...",
      "context_config": {
        "token_budget_ratio": 0.75,
        "max_memory_results": 10,
        "max_history_messages": 30
      },
      "dnd_rules": {
        "critical_success": 20,
        "critical_fail": 1
      },
      "active_api": "openai"
    }
  }
}
```

## 記憶系統與 RAG

### 工作原理

```txt
用戶提問 → 向量搜尋相關記憶 → 載入對話歷史 → 構建上下文 → AI 回應
```

### 向量計算方式

1. **Local**：本地 384 維向量，保護隱私
2. **API**：使用外部 API，更高精度

### 上下文構建

```txt
1. 系統提示詞（可自定義）
2. 記憶（3-20 條，向量相似度）
3. 歷史（5-50 條，時間排序）
4. 當前訊息
```

### TRPG 應用

- 角色關係追蹤
- 劇情連貫性維持
- 世界觀設定管理
- 任務進度記錄

## 架構與開發

### 專案結構

```bash
src/
├── bot/commands/      # 指令層 (12 模組)
├── utils/             # 核心邏輯 (8 模組)
└── models/            # 數據模型
```

### 核心模組

| 模組            | 行數  | 功能               |
| --------------- | ----- | ------------------ |
| memory.rs       | 837   | 記憶管理與向量搜尋 |
| skills.rs       | 732   | 技能管理系統       |
| api.rs          | 661   | API 管理與調用     |
| conversation.rs | 531   | 對話上下文構建     |
| import.rs       | 1,043 | 數據導入核心       |

### 技術棧

- **poise 0.6.1** - Discord 斜線指令框架
- **tokio 1.48** - 異步運行時
- **tokio-rusqlite 0.6** - 異步 SQLite
- **reqwest 0.12** - HTTP 客戶端（rustls）
- **calamine 0.31** - Excel 處理

### 開發

```bash
# 開發模式
RUST_LOG=debug cargo run

# 測試
cargo test

# 檢查
cargo clippy
cargo fmt --check
```

## 更新日誌

### v0.2.0 (2024-11-12)

#### 新功能

- ✨ 自定義系統提示詞（193 行新模組）
- ✨ 上下文配置管理（Token 預算、記憶/歷史範圍可調）
- ✨ `/prompt` 指令（4 個子指令）
- 🐛 資料庫權限修復（自動檢查與創建）

#### 改進

- 🔧 修正 ConfigManager 死鎖問題
- 🔧 修正 `created_at` 欄位類型兼容
- 📝 記憶指令重構（使用 action 枚舉）
- 📊 詳細日誌輸出

#### 技術

- 移除 `futures::executor::block_on`
- 優化異步/await 使用
- 代碼淨增長：~550 行

---

## 許可證

MIT License - Copyright (c) 2024

## 支援

- 📖 使用 `/help` 查看指令說明
- 🐛 [提交 Issue](https://github.com/lanxia404/trpg-discord-bot/issues)
- 💬 [參與討論](https://github.com/lanxia404/trpg-discord-bot/discussions)

---

### 由 Rust 🦀 驅動的 TRPG 助手

⭐ 如果有幫助，請給個星標！
