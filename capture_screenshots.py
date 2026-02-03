#!/usr/bin/env python3
"""
WAVELET 截图自动化脚本
用于Steam商店素材捕获

使用方法:
    python3 capture_screenshots.py --mode all
    python3 capture_screenshots.py --mode ui
    python3 capture_screenshots.py --mode ai

依赖:
    pip install pyautogui pillow opencv-python
"""

import argparse
import os
import time
import subprocess
from pathlib import Path
from datetime import datetime

# 配置
OUTPUT_DIR = Path(__file__).parent / "screenshots"
STEAM_REQUIREMENTS = {
    "main_capsule": (1232, 706, "main_capsule"),
    "small_capsule": (462, 174, "small_capsule"),
    "header": (920, 430, "header"),
    "screenshot": (1920, 1080, "screenshot"),
    "library_hero": (3840, 1240, "library_hero"),
}

# 截图清单 - 10个必拍场景
SCREENSHOT_PLAN = [
    {
        "name": "01_main_interface",
        "description": "主界面全貌 - 展示完整UI布局",
        "duration": 5,
        "highlight": "主界面"
    },
    {
        "name": "02_dark_theme",
        "description": "Dark主题 - 专业沉稳风格",
        "duration": 3,
        "highlight": "Dark主题"
    },
    {
        "name": "03_retro_theme",
        "description": "Retro主题 - 温暖复古风格",
        "duration": 3,
        "highlight": "Retro主题"
    },
    {
        "name": "04_cyber_theme",
        "description": "Cyber主题 - 酷炫科幻风格",
        "duration": 3,
        "highlight": "Cyber主题"
    },
    {
        "name": "05_oscillator_control",
        "description": "振荡器控制 - 波形选择特写",
        "duration": 4,
        "highlight": "振荡器"
    },
    {
        "name": "06_filter_control",
        "description": "滤波器调节 - 旋钮特写",
        "duration": 4,
        "highlight": "滤波器"
    },
    {
        "name": "07_ai_melody_generation",
        "description": "AI旋律生成 - 风格选择界面",
        "duration": 6,
        "highlight": "AI旋律"
    },
    {
        "name": "08_ai_chord_progression",
        "description": "AI和弦进行 - 生成结果展示",
        "duration": 5,
        "highlight": "AI和弦"
    },
    {
        "name": "09_ai_rhythm_generation",
        "description": "AI节奏生成 - 鼓组界面",
        "duration": 5,
        "highlight": "AI节奏"
    },
    {
        "name": "10_visualizer",
        "description": "实时可视化 - 波形跳动效果",
        "duration": 4,
        "highlight": "可视化"
    },
    {
        "name": "11_preset_browser",
        "description": "预设浏览器 - 50+预设展示",
        "duration": 4,
        "highlight": "预设"
    },
    {
        "name": "12_community_panel",
        "description": "社区面板 - 用户分享展示",
        "duration": 4,
        "highlight": "社区"
    },
]


def setup_output_dir():
    """创建输出目录"""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    print(f"📁 截图将保存到: {OUTPUT_DIR}")


def capture_screenshot(name, description, delay=3):
    """
    捕获单张截图
    
    Args:
        name: 截图名称
        description: 场景描述
        delay: 等待秒数 (给UI时间加载)
    """
    print(f"\n🎬 准备捕获: {name}")
    print(f"   描述: {description}")
    print(f"   等待 {delay} 秒...")
    
    time.sleep(delay)
    
    try:
        import pyautogui
        screenshot = pyautogui.screenshot()
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"{name}_{timestamp}.png"
        filepath = OUTPUT_DIR / filename
        screenshot.save(filepath)
        print(f"   ✅ 已保存: {filepath}")
        return filepath
    except ImportError:
        print("   ⚠️ pyautogui 未安装，使用备用方法...")
        return capture_with_screencapture(name, delay)


def capture_with_screencapture(name, delay=3):
    """使用macOS screencapture命令 (备用方案)"""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"{name}_{timestamp}.png"
    filepath = OUTPUT_DIR / filename
    
    time.sleep(delay)
    
    result = subprocess.run(
        ["screencapture", "-x", str(filepath)],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        print(f"   ✅ 已保存: {filepath}")
        return filepath
    else:
        print(f"   ❌ 截图失败: {result.stderr}")
        return None


def generate_steam_assets(screenshots):
    """
    根据截图生成Steam所需的各种尺寸素材
    
    Args:
        screenshots: 截图文件路径列表
    """
    try:
        from PIL import Image
    except ImportError:
        print("⚠️ 需要安装Pillow: pip install pillow")
        return
    
    from PIL import Image  # 确保Image可用
    
    print("\n🖼️ 生成Steam素材...")
    
    for shot in screenshots:
        if shot is None or not shot.exists():
            continue
            
        with Image.open(shot) as img:
            base_name = shot.stem
            
            # 生成Steam需要的各种尺寸
            for size_name, (width, height, suffix) in STEAM_REQUIREMENTS.items():
                if width > img.width or height > img.height:
                    print(f"   ⏭️ 跳过 {size_name}: 原图太小")
                    continue
                
                # 缩放到目标尺寸 (保持比例，居中裁剪)
                resized = resize_and_crop(img, width, height)
                output_path = OUTPUT_DIR / f"{base_name}_{suffix}.png"
                resized.save(output_path, "PNG", quality=95)
                print(f"   ✅ {size_name}: {output_path}")


def resize_and_crop(img, target_width, target_height):
    """调整大小并居中裁剪图片"""
    img_ratio = img.width / img.height
    target_ratio = target_width / target_height
    
    if img_ratio > target_ratio:
        # 图片更宽，按高度缩放
        new_height = target_height
        new_width = int(new_height * img_ratio)
    else:
        # 图片更高，按宽度缩放
        new_width = target_width
        new_height = int(new_width / img_ratio)
    
    resized = img.resize((new_width, new_height), Image.Resampling.LANCZOS)
    
    # 居中裁剪
    left = (new_width - target_width) // 2
    top = (new_height - target_height) // 2
    right = left + target_width
    bottom = top + target_height
    
    return resized.crop((left, top, right, bottom))


def simulate_capture(name, description, delay=0):
    """
    模拟捕获 (用于没有实际UI时)
    创建一个占位图
    """
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError:
        print("⚠️ 需要安装Pillow: pip install pillow")
        return None
    
    print(f"\n🎬 模拟捕获: {name}")
    print(f"   描述: {description}")
    
    # 创建占位图
    width, height = 1920, 1080
    img = Image.new('RGB', (width, height), color=(30, 30, 50))
    draw = ImageDraw.Draw(img)
    
    # 绘制标题
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 60)
    except:
        font = ImageFont.load_default()
    
    draw.text((width//2 - 300, height//2 - 50), f"WAVELET - {name}", 
              fill=(255, 255, 255), font=font)
    draw.text((width//2 - 200, height//2 + 20), description, 
              fill=(150, 150, 150), font=font)
    
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"{name}_{timestamp}.png"
    filepath = OUTPUT_DIR / filename
    img.save(filepath, "PNG")
    print(f"   ✅ 占位图已保存: {filepath}")
    
    return filepath


def run_automation(godot_executable=None, headless=False):
    """
    运行自动化截图流程
    
    Args:
        godot_executable: Godot可执行文件路径
        headless: 是否使用headless模式
    """
    setup_output_dir()
    
    print("\n" + "="*50)
    print("🎮 WAVELET 截图自动化")
    print("="*50)
    
    if godot_executable and os.path.exists(godot_executable):
        print(f"🚀 启动Godot: {godot_executable}")
        # 启动Godot
        # subprocess.Popen([godot_executable, "--headless"])
        # time.sleep(5)
    
    screenshots = []
    
    # 捕获每个场景
    for scene in SCREENSHOT_PLAN:
        if godot_executable:
            filepath = capture_screenshot(scene["name"], scene["description"], scene["duration"])
        else:
            filepath = simulate_capture(scene["name"], scene["description"])
        
        if filepath:
            screenshots.append(filepath)
    
    # 生成Steam素材
    if screenshots:
        generate_steam_assets(screenshots)
    
    print("\n" + "="*50)
    print("✅ 截图完成!")
    print(f"📁 输出目录: {OUTPUT_DIR}")
    print(f"📊 捕获截图: {len(screenshots)}张")
    print("="*50)
    
    return screenshots


def main():
    parser = argparse.ArgumentParser(description="WAVELET 截图自动化工具")
    parser.add_argument("--mode", choices=["all", "ui", "ai"], default="all",
                       help="截图模式: all=全部, ui=UI界面, ai=AI功能")
    parser.add_argument("--godot", type=str, help="Godot可执行文件路径")
    parser.add_argument("--simulate", action="store_true",
                       help="使用模拟模式(不依赖实际UI)")
    
    args = parser.parse_args()
    
    if args.simulate:
        run_automation(godot_executable=None)
    else:
        run_automation(godot_executable=args.godot)


if __name__ == "__main__":
    main()
