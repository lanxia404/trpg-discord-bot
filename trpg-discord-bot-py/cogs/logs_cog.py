import discord
from discord.ext import commands
from typing import Optional
import asyncio
from utils.config import GuildConfig


class LogsCog(commands.Cog, name="Logs"):
    """日誌相關指令"""
    def __init__(self, bot, config_manager):
        self.bot = bot
        self.config_manager = config_manager

    @commands.hybrid_command(name="log_stream", description="控制日誌串流開關")
    async def log_stream(self, ctx, state: str, channel: Optional[discord.TextChannel] = None):
        """控制日誌串流開關"""
        if not ctx.guild:
            await ctx.send("此指令只能在服務器中使用")
            return
        
        if state.lower() not in ["on", "off"]:
            await ctx.send("狀態參數必須是 'on' 或 'off'")
            return
        
        guild_config = self.config_manager.get_guild_config(ctx.guild.id)
        
        if state.lower() == "on":
            if not channel:
                await ctx.send("請提供要啟用串流的文字頻道")
                return
            guild_config.log_channel = channel.id
            self.config_manager.set_guild_config(ctx.guild.id, guild_config)
            await ctx.send(f"日誌串流已開啟，使用頻道: {channel.mention}")
        else:  # off
            guild_config.log_channel = None
            self.config_manager.set_guild_config(ctx.guild.id, guild_config)
            await ctx.send("日誌串流已關閉")

    @commands.hybrid_command(name="log_stream_mode", description="設定日誌串流模式")
    async def log_stream_mode(self, ctx, mode: str):
        """設定日誌串流模式"""
        if not ctx.guild:
            await ctx.send("此指令只能在服務器中使用")
            return
        
        if mode.lower() not in ["live", "batch"]:
            await ctx.send("模式參數必須是 'live' 或 'batch'")
            return
        
        guild_config = self.config_manager.get_guild_config(ctx.guild.id)
        guild_config.stream_mode = mode.lower().capitalize()
        self.config_manager.set_guild_config(ctx.guild.id, guild_config)
        
        await ctx.send(f"串流模式已設定為: {mode.lower()}")
    
    @commands.hybrid_command(name="crit", description="設定大成功/大失敗紀錄頻道")
    async def crit(self, ctx, kind: str, channel: Optional[discord.TextChannel] = None):
        """設定大成功/大失敗紀錄頻道"""
        if not ctx.guild:
            embed = discord.Embed(
                title="錯誤",
                description="此指令僅能在服務器中使用",
                color=0xff0000
            )
            await ctx.send(embed=embed)
            return
        
        if kind.lower() not in ["success", "fail"]:
            embed = discord.Embed(
                title="錯誤",
                description="紀錄類型必須是 'success' 或 'fail'",
                color=0xff0000
            )
            await ctx.send(embed=embed)
            return
        
        guild_config = self.config_manager.get_guild_config(ctx.guild.id)
        
        if kind.lower() == "success":
            guild_config.crit_success_channel = channel.id if channel else None
            field_name = "大成功"
        else:  # fail
            guild_config.crit_fail_channel = channel.id if channel else None
            field_name = "大失敗"
        
        self.config_manager.set_guild_config(ctx.guild.id, guild_config)
        
        if channel:
            description = f"已設定{field_name}紀錄頻道為 {channel.mention}"
        else:
            description = f"已清除{field_name}紀錄頻道設定"
        
        embed = discord.Embed(
            title="紀錄頻道已更新",
            description=description,
            color=0x7289da
        )
        await ctx.send(embed=embed)


async def setup(bot):
    """設置Cog"""
    await bot.add_cog(LogsCog(bot, bot.config_manager))