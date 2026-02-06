#!/usr/bin/env python3
"""
WAVELET Screenshot Automation Script
For capturing Steam store assets

Usage:
    python3 capture_screenshots.py --mode all
    python3 capture_screenshots.py --mode ui
    python3 capture_screenshots.py --mode ai

Dependencies:
    pip install pyautogui pillow opencv-python
"""

import argparse
import os
import time
import subprocess
from pathlib import Path
from datetime import datetime

# Configuration
OUTPUT_DIR = Path(__file__).parent / "screenshots"
STEAM_REQUIREMENTS = {
    "main_capsule": (1232, 706, "main_capsule"),
    "small_capsule": (462, 174, "small_capsule"),
    "header": (920, 430, "header"),
    "screenshot": (1920, 1080, "screenshot"),
    "library_hero": (3840, 1240, "library_hero"),
}

# Screenshot checklist - 10 must-capture scenes
SCREENSHOT_PLAN = [
    {
        "name": "01_main_interface",
        "description": "Main interface overview - showcasing the full UI layout",
        "duration": 5,
        "highlight": "Main interface"
    },
    {
        "name": "02_dark_theme",
        "description": "Dark theme - professional and sleek style",
        "duration": 3,
        "highlight": "Dark theme"
    },
    {
        "name": "03_retro_theme",
        "description": "Retro theme - warm vintage style",
        "duration": 3,
        "highlight": "Retro theme"
    },
    {
        "name": "04_cyber_theme",
        "description": "Cyber theme - cool sci-fi style",
        "duration": 3,
        "highlight": "Cyber theme"
    },
    {
        "name": "05_oscillator_control",
        "description": "Oscillator controls - waveform selection close-up",
        "duration": 4,
        "highlight": "Oscillator"
    },
    {
        "name": "06_filter_control",
        "description": "Filter adjustment - knob close-up",
        "duration": 4,
        "highlight": "Filter"
    },
    {
        "name": "07_ai_melody_generation",
        "description": "AI melody generation - style selection interface",
        "duration": 6,
        "highlight": "AI melody"
    },
    {
        "name": "08_ai_chord_progression",
        "description": "AI chord progression - generated results display",
        "duration": 5,
        "highlight": "AI chords"
    },
    {
        "name": "09_ai_rhythm_generation",
        "description": "AI rhythm generation - drum kit interface",
        "duration": 5,
        "highlight": "AI rhythm"
    },
    {
        "name": "10_visualizer",
        "description": "Real-time visualization - waveform animation effect",
        "duration": 4,
        "highlight": "Visualizer"
    },
    {
        "name": "11_preset_browser",
        "description": "Preset browser - 50+ presets showcase",
        "duration": 4,
        "highlight": "Presets"
    },
    {
        "name": "12_community_panel",
        "description": "Community panel - user sharing showcase",
        "duration": 4,
        "highlight": "Community"
    },
]


def setup_output_dir():
    """Create output directory"""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    print(f"Screenshots will be saved to: {OUTPUT_DIR}")


def capture_screenshot(name, description, delay=3):
    """
    Capture a single screenshot

    Args:
        name: Screenshot name
        description: Scene description
        delay: Wait time in seconds (to allow UI to load)
    """
    print(f"\nPreparing to capture: {name}")
    print(f"   Description: {description}")
    print(f"   Waiting {delay} seconds...")
    
    time.sleep(delay)
    
    try:
        import pyautogui
        screenshot = pyautogui.screenshot()
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"{name}_{timestamp}.png"
        filepath = OUTPUT_DIR / filename
        screenshot.save(filepath)
        print(f"   Saved: {filepath}")
        return filepath
    except ImportError:
        print("   pyautogui is not installed, using fallback method...")
        return capture_with_screencapture(name, delay)


def capture_with_screencapture(name, delay=3):
    """Use macOS screencapture command (fallback method)"""
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
        print(f"   Saved: {filepath}")
        return filepath
    else:
        print(f"   Screenshot failed: {result.stderr}")
        return None


def generate_steam_assets(screenshots):
    """
    Generate Steam assets in various sizes from screenshots

    Args:
        screenshots: List of screenshot file paths
    """
    try:
        from PIL import Image
    except ImportError:
        print("Pillow is required: pip install pillow")
        return

    from PIL import Image  # Ensure Image is available

    print("\nGenerating Steam assets...")
    
    for shot in screenshots:
        if shot is None or not shot.exists():
            continue
            
        with Image.open(shot) as img:
            base_name = shot.stem
            
            # Generate various sizes required by Steam
            for size_name, (width, height, suffix) in STEAM_REQUIREMENTS.items():
                if width > img.width or height > img.height:
                    print(f"   Skipping {size_name}: source image too small")
                    continue
                
                # Scale to target size (maintain aspect ratio, center crop)
                resized = resize_and_crop(img, width, height)
                output_path = OUTPUT_DIR / f"{base_name}_{suffix}.png"
                resized.save(output_path, "PNG", quality=95)
                print(f"   ✅ {size_name}: {output_path}")


def resize_and_crop(img, target_width, target_height):
    """Resize and center-crop the image"""
    img_ratio = img.width / img.height
    target_ratio = target_width / target_height
    
    if img_ratio > target_ratio:
        # Image is wider, scale by height
        new_height = target_height
        new_width = int(new_height * img_ratio)
    else:
        # Image is taller, scale by width
        new_width = target_width
        new_height = int(new_width / img_ratio)
    
    resized = img.resize((new_width, new_height), Image.Resampling.LANCZOS)
    
    # Center crop
    left = (new_width - target_width) // 2
    top = (new_height - target_height) // 2
    right = left + target_width
    bottom = top + target_height
    
    return resized.crop((left, top, right, bottom))


def simulate_capture(name, description, delay=0):
    """
    Simulate capture (used when no actual UI is available)
    Creates a placeholder image
    """
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError:
        print("Pillow is required: pip install pillow")
        return None

    print(f"\nSimulated capture: {name}")
    print(f"   Description: {description}")

    # Create placeholder image
    width, height = 1920, 1080
    img = Image.new('RGB', (width, height), color=(30, 30, 50))
    draw = ImageDraw.Draw(img)
    
    # Draw title
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
    print(f"   Placeholder saved: {filepath}")
    
    return filepath


def run_automation(godot_executable=None, headless=False):
    """
    Run the automated screenshot workflow

    Args:
        godot_executable: Path to the Godot executable
        headless: Whether to use headless mode
    """
    setup_output_dir()
    
    print("\n" + "="*50)
    print("WAVELET Screenshot Automation")
    print("="*50)
    
    if godot_executable and os.path.exists(godot_executable):
        print(f"Launching Godot: {godot_executable}")
        # Launch Godot
        # subprocess.Popen([godot_executable, "--headless"])
        # time.sleep(5)
    
    screenshots = []
    
    # Capture each scene
    for scene in SCREENSHOT_PLAN:
        if godot_executable:
            filepath = capture_screenshot(scene["name"], scene["description"], scene["duration"])
        else:
            filepath = simulate_capture(scene["name"], scene["description"])
        
        if filepath:
            screenshots.append(filepath)
    
    # Generate Steam assets
    if screenshots:
        generate_steam_assets(screenshots)
    
    print("\n" + "="*50)
    print("Screenshots complete!")
    print(f"Output directory: {OUTPUT_DIR}")
    print(f"Screenshots captured: {len(screenshots)}")
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
