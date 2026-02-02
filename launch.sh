#!/bin/bash

# WAVELET - 一键启动脚本 🎮🎸

echo "🎵 启动 WAVELET 合成器..."

# 检查 Godot 是否安装
if ! command -v godot &> /dev/null; then
    echo "⚠️  Godot 4 未找到，请先安装 Godot 4"
    echo "   下载地址: https://godotengine.org/download"
    exit 1
fi

# 启动 Godot 项目
cd "$(dirname "$0")"
godot --path godot/ || godot4 --path godot/

echo "👋 WAVELET 已关闭，再见！"
