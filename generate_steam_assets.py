#!/usr/bin/env python3
"""
Steam Store Assets Generator
从现有截图生成Steam要求的各种尺寸素材
"""

from PIL import Image, ImageDraw, ImageFont
import os

# 配置
SOURCE_DIR = "screenshots"
OUTPUT_DIR = "steam_assets"
os.makedirs(OUTPUT_DIR, exist_ok=True)

# Steam素材尺寸规格
STEAM_ASSETS = {
    # 商店展示素材
    "header_capsule": (920, 430),      # 商店头图
    "small_capsule": (462, 174),       # 小图
    "main_capsule": (1232, 706),       # 主展示图
    "vertical_capsule": (748, 896),    # 竖图

    # 库展示素材
    "library_capsule": (600, 900),     # 库主图
    "library_hero": (3840, 1240),      # 库英雄图

    # 截图 (已有)
    "screenshot": (1920, 1080),        # 商店截图
}

def create_store_asset(source_path, output_name, size, fit_strategy="cover"):
    """
    创建Steam素材
    fit_strategy:
    - "cover": 裁剪填充 (保留内容)
    - "contain": 保持比例放入中间
    """
    source = Image.open(source_path)
    source = source.convert("RGB")

    target_w, target_h = size
    source_w, source_h = source.size

    target_ratio = target_w / target_h
    source_ratio = source_w / source_h

    if fit_strategy == "cover":
        # 裁剪填充
        if source_ratio > target_ratio:
            # 源图更宽，按高度裁剪宽度
            new_h = target_h
            new_w = int(new_h * source_ratio)
            resized = source.resize((new_w, new_h), Image.LANCZOS)
            left = (new_w - target_w) // 2
            cropped = resized.crop((0, 0, target_w, target_h))
        else:
            # 源图更高，按宽度裁剪高度
            new_w = target_w
            new_h = int(new_w / source_ratio)
            resized = source.resize((new_w, new_h), Image.LANCZOS)
            top = (new_h - target_h) // 2
            cropped = resized.crop((0, 0, target_w, target_h))
    else:
        # 保持比例放入中间，添加黑边
        resized = Image.new("RGB", (target_w, target_h), (0, 0, 0))
        if source_ratio > target_ratio:
            new_w = target_w
            new_h = int(new_w / source_ratio)
            resized_source = source.resize((new_w, new_h), Image.LANCZOS)
            top = (target_h - new_h) // 2
            resized.paste(resized_source, (0, top))
        else:
            new_h = target_h
            new_w = int(new_h * source_ratio)
            resized_source = source.resize((new_w, new_h), Image.LANCZOS)
            left = (target_w - new_w) // 2
            resized.paste(resized_source, (left, 0))
        cropped = resized

    # 添加水印或文字
    draw = ImageDraw.Draw(cropped)

    # 保存
    output_path = os.path.join(OUTPUT_DIR, f"{output_name}.png")
    cropped.save(output_path, quality=95)
    print(f"✅ {output_name}: {target_w}x{target_h} -> {output_path}")
    return output_path

def create_hero_with_logo(source_paths, output_name, size):
    """创建带Logo的英雄图"""
    target_w, target_h = size

    # 创建背景
    bg = Image.new("RGB", (target_w, target_h), (15, 15, 25))  # 深色背景
    draw = ImageDraw.Draw(bg)

    # 计算网格
    n = len(source_paths)
    cols = min(4, n)
    rows = (n + cols - 1) // cols
    padding = int(target_w * 0.02)
    gap = padding

    cell_w = (target_w - padding * (cols + 1)) // cols
    cell_h = int(cell_w * 9/16)
    total_h = padding + cell_h * rows + gap * (rows - 1)
    start_y = (target_h - total_h) // 2

    # 添加截图缩略图
    for i, src in enumerate(source_paths[:12]):  # 最多12张
        img = Image.open(src)
        img = img.convert("RGB")
        img = img.resize((cell_w, cell_h), Image.LANCZOS)

        col = i % cols
        row = i // cols
        x = padding + col * (cell_w + gap)
        y = start_y + row * (cell_h + gap)

        # 添加圆角效果（简单模拟）
        bg.paste(img, (x, y))

    # 添加标题文字 (用色块代替)
    title_y = start_y - 80
    draw.rectangle([padding, title_y, padding + 400, title_y + 60], fill=(100, 100, 200))

    # 保存
    output_path = os.path.join(OUTPUT_DIR, f"{output_name}.png")
    bg.save(output_path, quality=95)
    print(f"✅ {output_name}: {target_w}x{target_h} -> {output_path}")
    return output_path

def main():
    print("🎨 WAVELET Steam素材生成器")
    print("=" * 50)

    # 获取可用截图
    screenshots = sorted([f for f in os.listdir(SOURCE_DIR) if f.endswith('.png')])
    print(f"📸 找到 {len(screenshots)} 张截图")

    if not screenshots:
        print("❌ 未找到截图!")
        return

    # 使用最新的截图
    latest_screenshot = os.path.join(SOURCE_DIR, screenshots[-1])
    print(f"🎯 使用: {screenshots[-1]}")

    # 生成商店素材
    print("\n🏪 生成商店素材...")

    create_store_asset(latest_screenshot, "header_capsule", STEAM_ASSETS["header_capsule"])
    create_store_asset(latest_screenshot, "small_capsule", STEAM_ASSETS["small_capsule"])
    create_store_asset(latest_screenshot, "main_capsule", STEAM_ASSETS["main_capsule"])
    create_store_asset(latest_screenshot, "vertical_capsule", STEAM_ASSETS["vertical_capsule"])

    print("\n📚 生成库素材...")

    # 库英雄图使用多张截图
    recent_screenshots = [os.path.join(SOURCE_DIR, s) for s in screenshots[-6:]]
    create_hero_with_logo(recent_screenshots, "library_hero", STEAM_ASSETS["library_hero"])

    # 库主图
    create_store_asset(latest_screenshot, "library_capsule", STEAM_ASSETS["library_capsule"])

    print("\n✅ 素材生成完成!")
    print(f"📁 输出目录: {OUTPUT_DIR}/")

if __name__ == "__main__":
    main()
