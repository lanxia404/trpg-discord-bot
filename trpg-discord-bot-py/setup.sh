#!/bin/bash
# 設置 Python 虛擬環境並安裝依賴
set -e

echo "設置 TRPG Discord Bot Python 開發環境..."

# 檢查 Python 是否已安裝
if ! command -v python3 &> /dev/null; then
    echo "錯誤：未找到 Python 3。請先安裝 Python 3。"
    exit 1
fi

# 檢查是否在項目根目錄的 trpg-discord-bot-py 子目錄中
if [ ! -f "requirements.txt" ]; then
    echo "錯誤：requirements.txt 不存在。請確保在 trpg-discord-bot-py 目錄中運行此腳本。"
    exit 1
fi

# 創建虛擬環境（如果不存在）
if [ ! -d "venv" ]; then
    echo "創建虛擬環境..."
    python3 -m venv venv
    echo "虛擬環境創建完成！"
else
    echo "虛擬環境已存在。"
fi

# 激活虛擬環境
source venv/bin/activate

# 升級 pip
pip install --upgrade pip

# 安裝依賴
echo "安裝依賴..."
pip install -r requirements.txt

echo "=================================="
echo "環境設置完成！"
echo ""
echo "要運行機器人，請執行："
echo "  source venv/bin/activate"
echo "  python main.py"
echo ""
echo "注意：首次運行前，請在項目根目錄創建 .env 文件"
echo "並添加 DISCORD_TOKEN=your_bot_token_here"
echo "=================================="