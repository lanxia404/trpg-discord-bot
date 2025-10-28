#!/usr/bin/env python3
"""
TRPG Discord Bot - Python版
專為TRPG設計的Discord機器人
"""

import os
import sys
from dotenv import load_dotenv

# 加載環境變量
load_dotenv()

# 確保路徑正確
sys.path.insert(0, os.path.dirname(__file__))

from bot import TRPGBot
from utils.logger import get_logger


def main():
    """主函數"""
    logger = get_logger()
    
    logger.info("正在啟動TRPG Discord Bot...")
    
    # 創建並啟動機器人（機器人會自己查找環境變量）
    try:
        bot = TRPGBot()
    except ValueError as e:
        logger.error(f"錯誤：{e}")
        print(f"錯誤：{e}")
        sys.exit(1)
    
    try:
        # 運行機器人
        import asyncio
        asyncio.run(bot.start())
    except KeyboardInterrupt:
        logger.info("收到中斷信號，正在關閉機器人...")
    except Exception as e:
        logger.error(f"機器人運行時出現錯誤: {e}")
        print(f"機器人運行時出現錯誤: {e}")
    finally:
        # 關閉機器人
        try:
            asyncio.run(bot.close())
        except:
            pass  # 如果機器人已經關閉，則跳過
        logger.info("機器人已關閉")


if __name__ == "__main__":
    main()