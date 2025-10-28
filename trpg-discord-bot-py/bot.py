import discord
from discord.ext import commands
import asyncio
import os
from typing import Optional

from utils.config import ConfigManager
from models.database import SkillsDB


class TRPGBot:
    """TRPG機器人類"""
    def __init__(self):
        # 遍歷查找環境變量和數據庫文件
        root_dir = self.find_project_root()
        
        # 查找環境變量文件
        env_file = self.find_env_file(root_dir)
        if env_file:
            from dotenv import load_dotenv
            load_dotenv(dotenv_path=env_file)
        
        # 從環境變量獲取token
        token = os.getenv("DISCORD_TOKEN")
        if not token:
            print("錯誤：未找到 DISCORD_TOKEN 環境變量")
            print("請執行以下操作之一：")
            print("1. 在項目根目錄創建 .env 文件，並添加 DISCORD_TOKEN=your_token_here")
            print("2. 在終端中設置環境變量：export DISCORD_TOKEN=your_token_here")
            raise ValueError("未找到 DISCORD_TOKEN 環境變量")
        
        self.token = token
        
        # 查找配置和數據庫文件
        config_path = self.find_config_file(root_dir)
        db_path = self.find_db_file(root_dir)
        
        self.config_manager = ConfigManager(config_path=config_path)
        self.skills_db = SkillsDB(db_path=db_path)
        
        # 設置機器人
        intents = discord.Intents.default()
        intents.message_content = True  # 需要讀取消息內容
        intents.guilds = True  # 需要訪問服務器信息
        intents.members = True  # 需要訪問成員信息
        
        self.bot = commands.Bot(
            command_prefix="!",
            intents=intents,
            description="專為TRPG設計的機器人"
        )
        
        self.setup_events()
    
    def find_project_root(self) -> 'Path':
        """查找項目根目錄"""
        from pathlib import Path
        current_path = Path(__file__).resolve()
        
        # 搜索包含 .git 目錄的父目錄，這是更可靠的項目根目錄標誌
        for parent in current_path.parents:
            if (parent / '.git').exists():
                return parent
        
        # 如果沒找到 .git 目錄，嘗試使用 README.md (但排除 Python 子目錄中的 README)
        for parent in current_path.parents:
            if (parent / 'README.md').exists():
                # 確保不是在子目錄中的 README
                if not str(parent).endswith('trpg-discord-bot-py'):
                    return parent
        
        # 如果還是沒找到，返回當前目錄的父目錄（trpg-discord-bot-py的父目錄）
        return current_path.parent.parent
    
    def find_env_file(self, root_dir: 'Path') -> 'Optional[Path]':
        """遍歷查找環境變量文件"""
        from pathlib import Path
        # 搜索模式：查找所有 .env 相關文件，排除target目錄
        search_patterns = ['.env', '.env.*', '*.env']
        
        for pattern in search_patterns:
            for env_file in root_dir.rglob(pattern):
                if env_file.is_file() and 'target' not in env_file.parts:
                    print(f"找到環境變量文件: {env_file}")
                    return env_file
        
        print(f"在 {root_dir} 及其子目錄中未找到環境變量文件")
        print("請在項目根目錄創建 .env 文件，並添加 DISCORD_TOKEN=your_token_here")
        return None
    
    def find_config_file(self, root_dir: 'Path') -> str:
        """遍歷查找配置文件"""
        config_file = root_dir / "config.json"
        if config_file.exists():
            return str(config_file)
        else:
            # 如果不存在，返回默認路徑
            return str(root_dir / "config.json")
    
    def find_db_file(self, root_dir: 'Path') -> str:
        """遍歷查找數據庫文件"""
        db_file = root_dir / "skills.db"
        if db_file.exists():
            return str(db_file)
        else:
            # 如果不存在，返回默認路徑
            return str(root_dir / "skills.db")
    
    def setup_events(self):
        """設置事件處理器"""
        @self.bot.event
        async def on_ready():
            print(f'{self.bot.user} 已經上線!')
            print(f'機器人ID: {self.bot.user.id}')
            print(f'已連接到 {len(self.bot.guilds)} 個服務器')
            print(f'已服務 {len(self.bot.users)} 個用戶')
            
            # 同步應用命令
            try:
                await self.bot.tree.sync()
                print("應用命令已同步")
            except Exception as e:
                print(f"同步應用命令時出錯: {e}")
        
        @self.bot.event
        async def on_guild_join(guild):
            """當機器人加入服務器時的處理"""
            print(f'加入了服務器: {guild.name} (ID: {guild.id})')
    
    def add_cogs(self):
        """添加Cog模塊"""
        # 這裡我們會動態添加Cog模塊
        from cogs.dice_cog import DiceCog
        from cogs.skills_cog import SkillsCog
        from cogs.logs_cog import LogsCog
        from cogs.admin_cog import AdminCog
        from cogs.help_cog import HelpCog
        
        self.bot.add_cog(DiceCog(self.bot, self.config_manager, self.skills_db))
        self.bot.add_cog(SkillsCog(self.bot, self.config_manager, self.skills_db))
        self.bot.add_cog(LogsCog(self.bot, self.config_manager))
        self.bot.add_cog(AdminCog(self.bot, self.config_manager))
        self.bot.add_cog(HelpCog(self.bot, self.config_manager))
    
    async def start(self):
        """啟動機器人"""
        self.add_cogs()
        await self.bot.start(self.token)
    
    async def close(self):
        """關閉機器人"""
        await self.bot.close()