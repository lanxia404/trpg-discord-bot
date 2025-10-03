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
    pub stream_throttle: u64, // milliseconds
    pub crit_success_channel: Option<u64>,
    pub crit_fail_channel: Option<u64>,
    pub dnd_rules: DnDRules,
    pub coc_rules: CoCRules,
}

impl Default for GuildConfig {
    fn default() -> Self {
        Self {
            log_channel: None,
            stream_mode: StreamMode::Batch,
            stream_throttle: 1000, // 1 second
            crit_success_channel: None,
            crit_fail_channel: None,
            dnd_rules: DnDRules::default(),
            coc_rules: CoCRules::default(),
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
    pub critical_success: u8, // usually 20
    pub critical_fail: u8,    // usually 1
    pub max_dice_count: u8,   // max dices in one roll
    pub max_dice_sides: u16,  // max sides on a dice
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
    pub critical_success: u8,      // usually 1
    pub critical_fail: u8,         // usually 100
    pub skill_divisor_hard: u8,    // usually 2 (hard success is skill/2)
    pub skill_divisor_extreme: u8, // usually 5 (extreme success is skill/5)
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
