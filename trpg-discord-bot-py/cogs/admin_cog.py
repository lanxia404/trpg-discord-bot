import discord
from discord.ext import commands
import asyncio
from typing import Optional
import subprocess
import sys
import os


class AdminCog(commands.Cog, name="Admin"):
    """管理相關指令"""
    def __init__(self, bot, config_manager):
        self.bot = bot
        self.config_manager = config_manager

    @commands.hybrid_command(name="admin", description="管理指令")
    async def admin_command(self, ctx, action: str, user: Optional[discord.User] = None):
        """管理指令"""
        # 檢查權限
        if not self.config_manager.is_developer(ctx.author.id):
            await ctx.send("您沒有權限執行此操作！")
            return
        
        action = action.lower()
        
        if action == "restart":
            # 確認重啟
            view = ConfirmView(ctx.author.id, "restart", ctx, self)
            embed = discord.Embed(
                title="確認重啟",
                description="確認執行重啟操作？",
                color=0xf39c12
            )
            await ctx.send(embed=embed, view=view, ephemeral=True)
        
        elif action == "shutdown":
            # 確認關閉
            view = ConfirmView(ctx.author.id, "shutdown", ctx, self)
            embed = discord.Embed(
                title="確認關閉",
                description="確認關閉機器人？",
                color=0xe74c3c
            )
            await ctx.send(embed=embed, view=view, ephemeral=True)
        
        elif action == "dev_add":
            if not user:
                await ctx.send("請指定要添加的用戶！")
                return
            
            # 確認添加開發者
            view = ConfirmView(ctx.author.id, "dev_add", ctx, self, target_user=user)
            embed = discord.Embed(
                title="確認添加開發者",
                description=f"確認將 {user.mention} 新增為開發者？",
                color=0x3498db
            )
            await ctx.send(embed=embed, view=view, ephemeral=True)
        
        elif action == "dev_remove":
            if not user:
                await ctx.send("請指定要移除的用戶！")
                return
            
            # 確認移除開發者
            view = ConfirmView(ctx.author.id, "dev_remove", ctx, self, target_user=user)
            embed = discord.Embed(
                title="確認移除開發者",
                description=f"確認將 {user.mention} 從開發者列表移除？",
                color=0xe74c3c
            )
            await ctx.send(embed=embed, view=view, ephemeral=True)
        
        elif action == "dev_list":
            developers = self.config_manager.global_config.developers
            if not developers:
                await ctx.send("目前沒有開發者")
            else:
                dev_list = []
                for dev_id in developers:
                    user = self.bot.get_user(dev_id)
                    if user:
                        dev_list.append(f"{user.mention}")
                    else:
                        dev_list.append(f"<@{dev_id}> (未知用戶)")
                
                embed = discord.Embed(
                    title="開發者列表",
                    description="\n".join(dev_list),
                    color=0x2ecc71
                )
                await ctx.send(embed=embed)
        
        else:
            await ctx.send("無效的管理操作。支持的操作：restart, shutdown, dev_add, dev_remove, dev_list")

    async def execute_restart(self, ctx):
        """執行重啟操作"""
        await ctx.send("已確認，機器人即將重新啟動……")
        
        # 根據配置決定重啟方式
        restart_mode = self.config_manager.global_config.restart_mode
        
        if restart_mode == "service":
            service_name = self.config_manager.global_config.restart_service
            if service_name:
                # 使用系統服務重啟
                try:
                    if sys.platform.startswith('win'):
                        subprocess.run(["sc", "stop", service_name], check=True)
                        subprocess.run(["sc", "start", service_name], check=True)
                    else:
                        subprocess.run(["sudo", "systemctl", "restart", service_name], check=True)
                except subprocess.CalledProcessError as e:
                    await ctx.send(f"服務重啟失敗: {e}")
                    sys.exit(1)
            else:
                await ctx.send("restart_mode 為 service 時，必須設定 restart_service")
                return
        else:
            # 使用execv重啟
            try:
                # 在新線程中重啟，讓回應能發送出去
                import threading
                def restart_bot():
                    # 在 Python 中沒有直接的 execv，我們重啟進程
                    os.execv(sys.executable, [sys.executable] + sys.argv)
                
                # 關閉當前機器人並重啟
                await self.bot.close()
                # 等待一段時間確保機器人關閉
                import time
                time.sleep(1)
                
                # 重新執行腳本
                os.execl(sys.executable, sys.executable, *sys.argv)
            except Exception as e:
                await ctx.send(f"重啟失敗: {e}")
        
        sys.exit(0)

    async def execute_shutdown(self, ctx):
        """執行關閉操作"""
        await ctx.send("已確認，機器人即將關閉……")
        
        # 根據配置決定關閉方式
        restart_mode = self.config_manager.global_config.restart_mode
        
        if restart_mode == "service":
            service_name = self.config_manager.global_config.restart_service
            if service_name:
                # 使用系統服務關閉
                try:
                    if sys.platform.startswith('win'):
                        subprocess.run(["sc", "stop", service_name], check=True)
                    else:
                        subprocess.run(["sudo", "systemctl", "stop", service_name], check=True)
                except subprocess.CalledProcessError as e:
                    await ctx.send(f"服務停止失敗: {e}")
                
                sys.exit(0)
            else:
                await ctx.send("restart_mode 為 service 時，必須設定 restart_service")
                return
        else:
            # 簡單退出
            await self.bot.close()
            sys.exit(0)

    async def execute_dev_add(self, ctx, user: discord.User):
        """執行添加開發者"""
        if self.config_manager.add_developer(user.id):
            await ctx.send(f"用戶 {user.mention} 已添加到開發者列表")
        else:
            await ctx.send(f"用戶 {user.mention} 已經是開發者")

    async def execute_dev_remove(self, ctx, user: discord.User):
        """執行移除開發者"""
        if self.config_manager.remove_developer(user.id):
            await ctx.send(f"用戶 {user.mention} 已從開發者列表移除")
        else:
            await ctx.send(f"用戶 {user.mention} 不在開發者列表中")


class ConfirmView(discord.ui.View):
    """確認視圖"""
    def __init__(self, author_id: int, action: str, ctx, admin_cog: AdminCog, target_user: discord.User = None):
        super().__init__(timeout=30)
        self.author_id = author_id
        self.action = action
        self.ctx = ctx
        self.admin_cog = admin_cog
        self.target_user = target_user

    @discord.ui.button(label="確認", style=discord.ButtonStyle.primary)
    async def confirm(self, interaction: discord.Interaction, button: discord.ui.Button):
        if interaction.user.id != self.author_id:
            await interaction.response.send_message("只有執行此操作的用戶可以確認。", ephemeral=True)
            return
        
        # 更新消息
        embed = discord.Embed(
            title="已確認",
            description="操作已確認",
            color=0x2ecc71
        )
        await interaction.response.edit_message(embed=embed, view=None)
        
        # 執行對應操作
        if self.action == "restart":
            await self.admin_cog.execute_restart(self.ctx)
        elif self.action == "shutdown":
            await self.admin_cog.execute_shutdown(self.ctx)
        elif self.action == "dev_add":
            await self.admin_cog.execute_dev_add(self.ctx, self.target_user)
        elif self.action == "dev_remove":
            await self.admin_cog.execute_dev_remove(self.ctx, self.target_user)

    @discord.ui.button(label="取消", style=discord.ButtonStyle.secondary)
    async def cancel(self, interaction: discord.Interaction, button: discord.ui.Button):
        if interaction.user.id != self.author_id:
            await interaction.response.send_message("只有執行此操作的用戶可以取消。", ephemeral=True)
            return
        
        embed = discord.Embed(
            title="操作已取消",
            description="操作已取消",
            color=0xf39c12
        )
        await interaction.response.edit_message(embed=embed, view=None)


async def setup(bot):
    """設置Cog"""
    await bot.add_cog(AdminCog(bot, bot.config_manager))