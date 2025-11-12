use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub developers: Vec<u64>,
    pub restart_mode: String,
    pub restart_service: Option<String>,
    pub global_stream_enabled: bool,
    pub global_stream_channel: Option<u64>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            developers: Vec::new(),
            restart_mode: "execv".to_string(),
            restart_service: None,
            global_stream_enabled: false,
            global_stream_channel: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    pub log_channel: Option<u64>,
    pub stream_mode: StreamMode,
    pub stream_throttle: u64, // 毫秒
    pub crit_success_channel: Option<u64>,
    pub crit_fail_channel: Option<u64>,
    pub dnd_rules: DnDRules,
    pub coc_rules: CoCRules,
    #[serde(default)]
    pub api_configs: std::collections::HashMap<String, crate::utils::api::ApiConfig>,
    #[serde(default)]
    pub active_api: Option<String>, // 指定活動的API配置名稱
    // 為了向後兼容而保留，但不再使用
    #[serde(default)]
    pub api_config: Option<crate::utils::api::ApiConfig>,
    #[serde(default)]
    pub memory_enabled_users: std::collections::HashMap<String, bool>, // 記憶功能開關：使用者ID -> 是否啟用
    #[serde(default)]
    pub memory_vector_storage_method: VectorStorageMethod, // 向量儲存計算方式
    #[serde(default)]
    pub custom_system_prompt: Option<String>, // 自定義系統提示詞
    #[serde(default)]
    pub context_config: ContextConfig, // 上下文配置
}

// 記憶向量儲存方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum VectorStorageMethod {
    #[default]
    Local,          // 本地計算和儲存
    EmbeddingApi,   // 使用嵌入API
    VectorDatabase, // 使用向量資料庫
}

// 上下文配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub token_budget_ratio: f32,      // 輸入/輸出比例 (預設 0.75)
    pub max_memory_results: usize,    // 最大記憶檢索數 (預設 10)
    pub max_history_messages: usize,  // 最大歷史訊息數 (預設 30)
    pub min_memory_results: usize,    // 最小記憶檢索數 (預設 3)
    pub min_history_messages: usize,  // 最小歷史訊息數 (預設 5)
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            token_budget_ratio: 0.75,
            max_memory_results: 10,
            max_history_messages: 30,
            min_memory_results: 3,
            min_history_messages: 5,
        }
    }
}


impl Default for GuildConfig {
    fn default() -> Self {
        Self {
            log_channel: None,
            stream_mode: StreamMode::Batch,
            stream_throttle: 1000, // 1 秒
            crit_success_channel: None,
            crit_fail_channel: None,
            dnd_rules: DnDRules::default(),
            coc_rules: CoCRules::default(),
            api_configs: std::collections::HashMap::new(),
            active_api: None,
            api_config: None,
            memory_enabled_users: std::collections::HashMap::new(),
            memory_vector_storage_method: VectorStorageMethod::Local,
            custom_system_prompt: None,
            context_config: ContextConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamMode {
    Live,
    Batch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnDRules {
    pub critical_success: u8, // 通常 20
    pub critical_fail: u8,    // 通常 1
    pub max_dice_count: u8,   // 最大擲骰數
    pub max_dice_sides: u16,  // 最大骰子面數
}

impl Default for DnDRules {
    fn default() -> Self {
        Self {
            critical_success: 20,
            critical_fail: 1,
            max_dice_count: 50,
            max_dice_sides: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoCRules {
    pub critical_success: u8,      // 通常 1
    pub critical_fail: u8,         // 通常 100
    pub skill_divisor_hard: u8,    // 通常 2 (hard success is skill/2)
    pub skill_divisor_extreme: u8, // 通常 5 (extreme success is skill/5)
}

impl Default for CoCRules {
    fn default() -> Self {
        Self {
            critical_success: 1,
            critical_fail: 100,
            skill_divisor_hard: 2,
            skill_divisor_extreme: 5,
        }
    }
}

#[derive(Debug)]
pub struct RollResult {
    pub dice_expr: String,
    pub rolls: Vec<u16>,
    pub modifier: i32,
    pub total: i32,
    pub is_critical_success: bool,
    pub is_critical_fail: bool,
    pub comparison_result: Option<bool>, // Some(true) for success, Some(false) for failure, None for no comparison
}

#[derive(Debug)]
pub struct DiceRoll {
    pub count: u8,
    pub sides: u16,
    pub modifier: i32,
    pub comparison: Option<(String, i32)>, // (operator, value) e.g. (">=", 15)
}
