import discord
from discord.ext import commands
import asyncio
from typing import Optional
import random

from utils.dice import roll_multiple_dice, format_roll_result, format_multiple_roll_results
from utils.coc import roll_coc_multi, format_coc_result, format_coc_multi_results, determine_success_level, format_success_level
from utils.config import GuildConfig


class DiceCog(commands.Cog, name="Dice"):
    """骰子相關指令"""
    def __init__(self, bot, config_manager, skills_db):
        self.bot = bot
        self.config_manager = config_manager
        self.skills_db = skills_db

    @commands.hybrid_command(name="roll", description="D&D 擲骰子")
    async def roll_command(self, ctx, expression: str):
        """D&D 骰子指令 - 擲骰子"""
        # 獲取公會配置
        guild_id = ctx.guild.id if ctx.guild else None
        if guild_id:
            rules = self.config_manager.get_guild_config(guild_id)
        else:
            rules = GuildConfig()  # 使用默認配置
        
        try:
            results = roll_multiple_dice(expression, rules.dnd_max_dice_count, rules)
            
            if len(results) == 1:
                message = format_roll_result(results[0])
                embed = discord.Embed(
                    title="D&D 擲骰結果",
                    description=message,
                    color=0x7289da
                )
            else:
                message = format_multiple_roll_results(results)
                embed = discord.Embed(
                    title="D&D 連續擲骰結果",
                    description=message,
                    color=0x7289da
                )
            
            await ctx.send(embed=embed)
            
        except ValueError as e:
            embed = discord.Embed(
                title="D&D 擲骰錯誤",
                description=f"錯誤: {str(e)}",
                color=0xff0000
            )
            await ctx.send(embed=embed)

    @commands.hybrid_command(name="coc", description="CoC 7e 擲骰")
    async def coc_command(self, ctx, skill: commands.Range[int, 1, 100], times: Optional[commands.Range[int, 1, 10]] = 1):
        """CoC 7e 指令"""
        if not ctx.guild:
            await ctx.send("此指令只能在服務器中使用")
            return
        
        rules = self.config_manager.get_guild_config(ctx.guild.id)
        
        results = roll_coc_multi(skill, times, rules)
        
        if len(results) == 1:
            result = results[0]
            success_level = determine_success_level(result.total, skill, rules)
            success_text = format_success_level(success_level)
            
            message = format_coc_result(result, skill)
            embed = discord.Embed(
                title="CoC 7e 擲骰結果",
                description=message,
                color=0x7289da
            )
        else:
            message = format_coc_multi_results(results, skill)
            embed = discord.Embed(
                title="CoC 7e 連續擲骰結果",
                description=message,
                color=0x7289da
            )
        
        await ctx.send(embed=embed)


async def setup(bot):
    """設置Cog"""
    await bot.add_cog(DiceCog(bot, bot.config_manager, bot.skills_db))