import discord
from discord.ext import commands


class HelpCog(commands.Cog, name="Help"):
    """幫助相關指令"""
    def __init__(self, bot, config_manager):
        self.bot = bot
        self.config_manager = config_manager

    @commands.hybrid_command(name="help", description="顯示指令說明")
    async def help_command(self, ctx):
        """顯示幫助信息"""
        embed = discord.Embed(
            title="TRPG Discord Bot 指令說明",
            description="請點擊下方按鈕查看各指令的詳細說明。\n支援 `/roll`、`/coc`、`/skill`、`/log_stream`、`/log_stream_mode`、`/crit`、`/admin`。",
            color=0x1abc9c
        )
        
        view = HelpView()
        await ctx.send(embed=embed, view=view)


class HelpView(discord.ui.View):
    """幫助視圖"""
    def __init__(self):
        super().__init__(timeout=120)  # 2分鐘後超時
        
        # 添加按鈕
        self.add_item(discord.ui.Button(
            label="D&D 擲骰",
            style=discord.ButtonStyle.primary,
            custom_id="help_roll"
        ))
        self.add_item(discord.ui.Button(
            label="CoC 擲骰",
            style=discord.ButtonStyle.primary,
            custom_id="help_coc"
        ))
        self.add_item(discord.ui.Button(
            label="技能指令",
            style=discord.ButtonStyle.primary,
            custom_id="help_skill"
        ))
        self.add_item(discord.ui.Button(
            label="日誌指令",
            style=discord.ButtonStyle.secondary,
            custom_id="help_logs"
        ))
        self.add_item(discord.ui.Button(
            label="管理指令",
            style=discord.ButtonStyle.secondary,
            custom_id="help_admin"
        ))

    @discord.ui.button(label="查看詳細說明", style=discord.ButtonStyle.green, emoji="ℹ️")
    async def show_help(self, interaction: discord.Interaction, button: discord.ui.Button):
        # 創建詳細說明
        details_embed = discord.Embed(
            title="指令詳細說明",
            color=0x1abc9c
        )
        
        details = (
            "**/roll <骰子表達式>**\n"
            "支援 `2d6`、`d20+5`、`1d10>=15`、`+3 d6` 等格式，解析骰數、面數、修正值與比較條件。預設最多 50 次擲骰。\n\n"
            
            "**/coc <技能值> [次數]**\n"
            "技能值 1-100，可設定 1-10 次連續擲骰。自動判斷普通/困難/極限成功、大成功（1）與大失敗（技能<50 時 96-100，否則 100）。\n\n"
            
            "**技能指令**\n"
            "`/skill add <名稱> <類型> <等級> <效果>`：新增或更新技能紀錄。\n"
            "`/skill show <名稱>`：支援模糊搜尋技能名稱，查詢技能。\n"
            "`/skill delete <名稱>`：刪除此服務器中的技能。\n\n"
            
            "**日誌相關指令**\n"
            "`/log_stream on <頻道>`：啟用串流並綁定頻道。\n"
            "`/log_stream off`：關閉串流。\n"
            "`/log_stream_mode <live|batch>`：切換即時或批次。\n"
            "`/crit <success|fail> [頻道]`：設定大成功/大失敗紀錄頻道，留空則清除設定。\n\n"
            
            "**管理指令（需開發者）**\n"
            "`/admin restart`：確認後重新啟動機器人。\n"
            "`/admin shutdown`：確認後關閉機器人。\n"
            "`/admin dev_add <用戶>` / `/admin dev_remove <用戶>`：維護開發者名單。\n"
            "`/admin dev_list`：列出所有已註冊開發者。"
        )
        
        details_embed.description = details
        await interaction.response.send_message(embed=details_embed, ephemeral=True)


async def setup(bot):
    """設置Cog"""
    await bot.add_cog(HelpCog(bot, bot.config_manager))