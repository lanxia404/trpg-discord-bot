import discord
from discord.ext import commands
from typing import Optional
import asyncio
from utils.config import GuildConfig


class SkillsCog(commands.Cog, name="Skills"):
    """技能相關指令"""
    def __init__(self, bot, config_manager, skills_db):
        self.bot = bot
        self.config_manager = config_manager
        self.skills_db = skills_db

    @commands.hybrid_command(name="skill", description="技能資料庫指令")
    async def skill_command(self, ctx, action: str, name: str, 
                           skill_type: Optional[str] = None, 
                           level: Optional[str] = None, 
                           effect: Optional[str] = None):
        """技能指令"""
        if not ctx.guild:
            embed = discord.Embed(
                title="錯誤",
                description="此指令僅能在服務器中使用",
                color=0xff0000
            )
            await ctx.send(embed=embed)
            return
        
        action = action.lower()
        
        if action == "add":
            if not all([skill_type, level, effect]):
                embed = discord.Embed(
                    title="錯誤",
                    description="添加技能時需要提供類型、等級和效果",
                    color=0xff0000
                )
                await ctx.send(embed=embed)
                return
            
            self.skills_db.add_skill(ctx.guild.id, ctx.author.id, name, skill_type, level, effect)
            
            embed = discord.Embed(
                title="技能已儲存",
                color=0x2ecc71
            )
            embed.add_field(name="名稱", value=f"`{name}`", inline=False)
            embed.add_field(name="類型", value=skill_type, inline=True)
            embed.add_field(name="等級", value=level, inline=True)
            embed.add_field(name="效果", value=effect, inline=False)
            
            await ctx.send(embed=embed)
        
        elif action == "show":
            skill = self.skills_db.find_skill_for_user(ctx.guild.id, ctx.author.id, name)
            
            if skill:
                embed = discord.Embed(
                    title=f"技能：<{skill.name}>",
                    color=0x7289da
                )
                embed.add_field(name="類型", value=skill.skill_type, inline=True)
                embed.add_field(name="等級", value=skill.level, inline=True)
                embed.add_field(name="效果", value=skill.effect, inline=False)
            else:
                embed = discord.Embed(
                    title=f"技能：<{name}>",
                    description=f"找不到 {ctx.author.mention} 的技能 `{name}`",
                    color=0xf39c12
                )
            
            await ctx.send(embed=embed)
        
        elif action == "delete":
            skill = self.skills_db.find_skill_in_guild(ctx.guild.id, name)
            
            if not skill:
                embed = discord.Embed(
                    title="錯誤",
                    description=f"找不到此服務器中的技能 `{name}`，無法刪除",
                    color=0xff0000
                )
                await ctx.send(embed=embed)
                return
            
            # 創建確認按鈕
            view = SkillDeleteView(ctx.author.id, ctx.guild.id, skill.normalized_name, 
                                   ctx.author.mention, skill.name, self.skills_db)
            
            embed = discord.Embed(
                title="確認刪除技能",
                description=f"目標技能：`{skill.name}`\n擁有者：<@{skill.user_id}>\n類型：{skill.skill_type}\n等級：{skill.level}\n效果：{skill.effect}",
                color=0xe74c3c
            )
            
            await ctx.send(embed=embed, view=view)
        
        else:
            embed = discord.Embed(
                title="錯誤",
                description="操作必須是 'add', 'show', 或 'delete'",
                color=0xff0000
            )
            await ctx.send(embed=embed)


class SkillDeleteView(discord.ui.View):
    """技能刪除確認視圖"""
    def __init__(self, author_id: int, guild_id: int, normalized_name: str, 
                 author_mention: str, skill_name: str, skills_db):
        super().__init__(timeout=30)
        self.author_id = author_id
        self.guild_id = guild_id
        self.normalized_name = normalized_name
        self.author_mention = author_mention
        self.skill_name = skill_name
        self.skills_db = skills_db

    @discord.ui.button(label="確認刪除", style=discord.ButtonStyle.danger, emoji="🗑️")
    async def confirm_delete(self, interaction: discord.Interaction, button: discord.ui.Button):
        if interaction.user.id != self.author_id:
            await interaction.response.send_message("只有執行此操作的用戶可以確認。", ephemeral=True)
            return
        
        # 刪除技能
        self.skills_db.delete_skill(self.guild_id, self.author_id, self.normalized_name)
        
        # 更新消息
        embed = discord.Embed(
            title="技能已刪除",
            description=f"{self.author_mention} 刪除了技能 `{self.skill_name}`",
            color=0x2ecc71
        )
        
        await interaction.response.edit_message(embed=embed, view=None)

    @discord.ui.button(label="取消", style=discord.ButtonStyle.secondary)
    async def cancel_delete(self, interaction: discord.Interaction, button: discord.ui.Button):
        if interaction.user.id != self.author_id:
            await interaction.response.send_message("只有執行此操作的用戶可以取消。", ephemeral=True)
            return
        
        embed = discord.Embed(
            title="操作已取消",
            description=f"{self.author_mention} 取消了刪除操作",
            color=0xf39c12
        )
        
        await interaction.response.edit_message(embed=embed, view=None)


async def setup(bot):
    """設置Cog"""
    await bot.add_cog(SkillsCog(bot, bot.config_manager, bot.skills_db))