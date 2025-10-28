#!/usr/bin/env python3
"""
TRPG Discord Bot - 項目根目錄管理腳本
用於檢查和管理根目錄的環境變量和數據庫文件
"""

import os
import json
from pathlib import Path


def find_project_root():
    """查找項目根目錄"""
    current_path = Path(__file__).resolve()  # __file__ 是 check_files.py 的路徑
    
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

def find_env_and_db_files():
    """查找項目根目錄的 .env 和數據庫文件"""
    root_dir = find_project_root()
    
    print("項目根目錄:", root_dir)
    
    # 查找 .env 文件
    env_files = list(root_dir.glob(".env*"))
    print("\n找到的環境變量文件:")
    if env_files:
        for env_file in env_files:
            print(f"  - {env_file}")
    else:
        print("  - 未找到 .env 文件")
    
    # 查找數據庫文件
    db_files = list(root_dir.glob("*.db")) + list(root_dir.glob("config.json"))
    print("\n找到的數據庫和配置文件:")
    if db_files:
        for db_file in db_files:
            print(f"  - {db_file}")
    else:
        print("  - 未找到數據庫或配置文件")
    
    return env_files, db_files


def print_setup_instructions():
    """打印設置說明"""
    print("\n設置說明:")
    print("如果這是第一次運行，請執行以下操作：")
    print()
    print("1. 創建環境文件:")
    print("   在項目根目錄創建 .env 文件，內容如下：")
    print("   DISCORD_TOKEN=your_discord_bot_token_here")
    print()
    print("2. 配置文件:")
    print("   config.json 文件將在機器人首次運行時自動創建")
    print("   skills.db 數據庫文件也將在首次使用技能功能時自動創建")
    print()
    print("運行機器人:")
    print("  cd trpg-discord-bot-py")
    print("  source venv/bin/activate  # 如果使用虛擬環境")
    print("  python main.py")


if __name__ == "__main__":
    print("TRPG Discord Bot - 根目錄文件管理")
    print("=" * 40)
    
    env_files, db_files = find_env_and_db_files()
    
    print(f"\n根目錄共找到 {len(env_files)} 個環境文件和 {len(db_files)} 個數據庫/配置文件")
    
    # 如果沒有找到環境文件，提供設置說明
    if not env_files:
        print("\n警告: 未找到環境變量文件 (.env)，機器人將無法運行。")
        print_setup_instructions()
    elif not db_files:
        print("\n提示: 未找到數據庫文件，這些文件將在機器人首次運行時自動創建。")