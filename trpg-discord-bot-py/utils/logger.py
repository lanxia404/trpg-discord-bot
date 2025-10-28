import logging
import os
from logging.handlers import RotatingFileHandler
from typing import Set


class DiscordLogger:
    """自定義日誌系統"""
    
    def __init__(self, log_file: str = "bot.log", level: int = logging.INFO):
        self.logger = logging.getLogger('TRPGBot')
        self.logger.setLevel(level)
        
        # 避免重複添加處理器
        if not self.logger.handlers:
            # 設置文件處理器（帶輪換）
            file_handler = RotatingFileHandler(
                log_file,
                maxBytes=1024*1024,  # 1MB
                backupCount=5,
                encoding='utf-8'
            )
            
            # 設置控制台處理器
            console_handler = logging.StreamHandler()
            
            # 設置格式
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
            file_handler.setFormatter(formatter)
            console_handler.setFormatter(formatter)
            
            # 添加處理器
            self.logger.addHandler(file_handler)
            self.logger.addHandler(console_handler)
    
    def info(self, message: str):
        """記錄信息級別日誌"""
        self.logger.info(message)
    
    def warning(self, message: str):
        """記錄警告級別日誌"""
        self.logger.warning(message)
    
    def error(self, message: str):
        """記錄錯誤級別日誌"""
        self.logger.error(message)
    
    def debug(self, message: str):
        """記錄調試級別日誌"""
        self.logger.debug(message)


# 創建全局日誌實例
logger = DiscordLogger()


def get_logger() -> DiscordLogger:
    """獲取日誌實例"""
    return logger