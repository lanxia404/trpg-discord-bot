#!/usr/bin/env python3
"""
TRPG Discord Bot - 文件遍歷查找腳本
用於查找項目中所有的 .env 和數據庫文件
"""

import os
from pathlib import Path


def find_all_env_and_db_files(root_dir):
    """查找項目中所有的 .env 和數據庫文件"""
    root_path = Path(root_dir)
    
    print(f"項目根目錄: {root_path}")
    print("="*50)
    
    # 只查找真正的 .env 文件，排除 Rust target 目錄
    print("查找 .env 相關文件:")
    env_files = []
    
    for item in root_path.rglob('.env*'):
        if item.is_file() and 'target' not in item.parts and 'deps' not in item.parts:
            env_files.append(item)
    
    for item in root_path.rglob('*env*'):
        if item.is_file() and 'target' not in item.parts and 'deps' not in item.parts and item.name.startswith('.env'):
            if item not in env_files:  # 避免重複
                env_files.append(item)
    
    if env_files:
        for env_file in env_files:
            print(f"  - {env_file.relative_to(root_path)}")
    else:
        print("  - 未找到 .env 相關文件")
    
    print()
    
    # 查找所有數據庫文件，排除 Rust target 目錄
    print("查找數據庫文件:")
    db_files = []
    
    for item in root_path.rglob('*.db'):
        if item.is_file() and 'target' not in item.parts:
            db_files.append(item)
    
    for item in root_path.rglob('config.json'):
        if item.is_file() and 'target' not in item.parts:
            if item not in db_files:  # 避免重複
                db_files.append(item)
    
    if db_files:
        for db_file in db_files:
            print(f"  - {db_file.relative_to(root_path)}")
    else:
        print("  - 未找到數據庫文件")
    
    print()
    
    # 總結
    print(f"總結:")
    print(f"  - 找到 {len(env_files)} 個環境變量文件")
    print(f"  - 找到 {len(db_files)} 個數據庫/配置文件")
    
    return env_files, db_files


def find_project_root():
    """查找項目根目錄"""
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


def print_setup_instructions(root_dir):
    """打印設置說明"""
    root_path = Path(root_dir)
    
    env_files = list(root_path.glob(".env*"))
    db_files = list(root_path.glob("*.db")) + list(root_path.glob("config.json"))
    
    print()
    print("=" * 50)
    print("設置說明:")
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
    print("文件路徑參考:")
    print(f"  項目根目錄: {root_path}")
    print("  所有 Python 源代碼在: trpg-discord-bot-py/ 目錄")
    print()
    print("運行機器人:")
    print("  cd trpg-discord-bot-py")
    print("  source venv/bin/activate  # 如果使用虛擬環境")
    print("  python main.py")
    print("=" * 50)


def main():
    print("TRPG Discord Bot - 文件遍歷查找腳本")
    print("=" * 50)
    
    # 查找項目根目錄
    root_dir = find_project_root()
    print(f"檢測到項目根目錄: {root_dir}")
    print()
    
    # 查找所有 .env 和數據庫文件
    env_files, db_files = find_all_env_and_db_files(root_dir)
    
    # 如果沒有找到重要的配置文件，提供設置說明
    if not env_files and not db_files:
        print("\\n警告: 未找到任何配置或數據庫文件。")
        print_setup_instructions(root_dir)
    elif not env_files:
        print("\\n警告: 未找到環境變量文件 (.env)。")
        print_setup_instructions(root_dir)
    elif not db_files:
        print("\\n提示: 未找到數據庫文件，這些文件將在機器人首次運行時自動創建。")
        print_setup_instructions(root_dir)
    else:
        print("\\n所有必需文件都已找到！")
    
    print()
    print("遍歷查找完成！")


if __name__ == "__main__":
    main()