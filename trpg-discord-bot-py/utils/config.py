import json
import os
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from pathlib import Path


@dataclass
class GlobalConfig:
    """全局配置"""
    developers: List[int]
    restart_mode: str
    restart_service: Optional[str]
    global_stream_enabled: bool
    global_stream_channel: Optional[int]

    def __post_init__(self):
        if self.developers is None:
            self.developers = []
        if self.restart_service is None:
            self.restart_service = None
        if self.global_stream_channel is None:
            self.global_stream_channel = None


@dataclass
class GuildConfig:
    """公會配置"""
    log_channel: Optional[int] = None
    stream_mode: str = "Batch"  # "Live" or "Batch"
    stream_throttle: int = 1000  # milliseconds
    crit_success_channel: Optional[int] = None
    crit_fail_channel: Optional[int] = None
    
    # D&D 规则配置
    dnd_critical_success: int = 20
    dnd_critical_fail: int = 1
    dnd_max_dice_count: int = 50
    dnd_max_dice_sides: int = 1000
    
    # CoC 规则配置
    coc_critical_success: int = 1
    coc_critical_fail: int = 100
    coc_skill_divisor_hard: int = 2
    coc_skill_divisor_extreme: int = 5


class ConfigManager:
    """配置管理器"""
    def __init__(self, config_path: str = "config.json"):
        self.config_path = config_path
        self.global_config = GlobalConfig(
            developers=[],
            restart_mode="execv",
            restart_service=None,
            global_stream_enabled=False,
            global_stream_channel=None
        )
        self.guild_configs: Dict[int, GuildConfig] = {}
        self.load_config()
    
    def load_config(self):
        """加載配置"""
        if os.path.exists(self.config_path):
            with open(self.config_path, 'r', encoding='utf-8') as f:
                data = json.load(f)
                
            # 加載全局配置
            global_data = data.get('global', {})
            self.global_config = GlobalConfig(
                developers=global_data.get('developers', []),
                restart_mode=global_data.get('restart_mode', 'execv'),
                restart_service=global_data.get('restart_service'),
                global_stream_enabled=global_data.get('global_stream_enabled', False),
                global_stream_channel=global_data.get('global_stream_channel')
            )
            
            # 加載公會配置
            guild_data = data.get('guilds', {})
            for guild_id, cfg in guild_data.items():
                self.guild_configs[int(guild_id)] = GuildConfig(
                    log_channel=cfg.get('log_channel'),
                    stream_mode=cfg.get('stream_mode', 'Batch'),
                    stream_throttle=cfg.get('stream_throttle', 1000),
                    crit_success_channel=cfg.get('crit_success_channel'),
                    crit_fail_channel=cfg.get('crit_fail_channel'),
                    dnd_critical_success=cfg.get('dnd_critical_success', 20),
                    dnd_critical_fail=cfg.get('dnd_critical_fail', 1),
                    dnd_max_dice_count=cfg.get('dnd_max_dice_count', 50),
                    dnd_max_dice_sides=cfg.get('dnd_max_dice_sides', 1000),
                    coc_critical_success=cfg.get('coc_critical_success', 1),
                    coc_critical_fail=cfg.get('coc_critical_fail', 100),
                    coc_skill_divisor_hard=cfg.get('coc_skill_divisor_hard', 2),
                    coc_skill_divisor_extreme=cfg.get('coc_skill_divisor_extreme', 5)
                )
        else:
            # 如果配置文件不存在，創建默認配置
            self.save_config()
    
    def save_config(self):
        """保存配置"""
        data = {
            'global': asdict(self.global_config),
            'guilds': {str(guild_id): asdict(config) 
                      for guild_id, config in self.guild_configs.items()}
        }
        
        with open(self.config_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
    
    def get_guild_config(self, guild_id: int) -> GuildConfig:
        """獲取公會配置"""
        return self.guild_configs.get(guild_id, GuildConfig())
    
    def set_guild_config(self, guild_id: int, config: GuildConfig):
        """設置公會配置"""
        self.guild_configs[guild_id] = config
        self.save_config()
    
    def is_developer(self, user_id: int) -> bool:
        """檢查是否為開發者"""
        return user_id in self.global_config.developers
    
    def add_developer(self, user_id: int) -> bool:
        """添加開發者"""
        if user_id in self.global_config.developers:
            return False
        self.global_config.developers.append(user_id)
        self.save_config()
        return True
    
    def remove_developer(self, user_id: int) -> bool:
        """移除開發者"""
        if user_id not in self.global_config.developers:
            return False
        self.global_config.developers.remove(user_id)
        self.save_config()
        return True