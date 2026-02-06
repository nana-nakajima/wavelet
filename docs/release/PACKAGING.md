# 📦 WAVELET 跨平台打包方案

**创建时间**: 2026-02-03 14:20
**作者**: Nana Nakajima

---

## 🎯 打包目标

为WAVELET创建跨平台安装包：
- **Windows**: .exe安装程序
- **macOS**: .dmg磁盘映像
- **Linux**: .AppImage或.deb包

---

## 📁 项目架构

```
wavelet/
├── src/                    # Rust音频引擎
│   └── lib.rs              # 编译为动态库
├── godot/                  # Godot UI项目
│   └── scenes/main.tscn    # 导出为.pck
├── python/                 # Python启动器
│   └── main.py
└── build/                  # 打包输出目录
```

---

## 🔨 Rust部分打包

### 1. 编译动态库

```bash
# macOS
cargo build --release --target x86_64-apple-darwin
# 输出: target/release/libwavelet.dylib

# Windows
cargo build --release --target x86_64-pc-windows-gnu
# 输出: target/release/wavelet.dll

# Linux
cargo build --release --target x86_64-unknown-linux-gnu
# 输出: target/release/libwavelet.so
```

### 2. 跨平台编译支持

需要安装交叉编译工具链：
```bash
# macOS (使用Homebrew)
brew install mingw-w64
brew install osxcross  # 需要Xcode
```

### 3. Rust打包注意事项

- 使用 `cdylib` crate type
- 确保无平台特定代码
- 使用 `#[cfg(target_os = "...")]` 做条件编译

---

## 🎮 Godot部分打包

### 1. 导出项目

```bash
# 使用Godot命令行导出
godot --headless --export-release "WAVELET" "export/wavelet.pck"
```

### 2. Godot导出要求

- 需要安装Godot 4.x导出模板
- 需要图形界面环境或CI/CD
- 导出的.pck文件包含所有UI和资源

### 3. 导出预设配置

```ini
# export_presets.cfg
[preset.0]
name="WAVELET"
platform="Mac OSX"
runnable=true
export_path="export/wavelet.pck"
```

---

## 🐍 Python启动器打包

### 1. 启动器功能

```python
# python/main.py
import sys
import platform
from ctypes import cdll

def load_library():
    """加载Rust动态库"""
    system = platform.system()
    if system == "Darwin":
        lib_name = "libwavelet.dylib"
    elif system == "Windows":
        lib_name = "wavelet.dll"
    else:
        lib_name = "libwavelet.so"
    
    return cdll.LoadLibrary(f"./lib/{lib_name}")

def main():
    wavelet = load_library()
    # 初始化并启动应用
    print("WAVELET - Abstract Sound Synthesizer")
    print(f"Platform: {platform.system()}")
    print("Loading...")

if __name__ == "__main__":
    main()
```

### 2. PyInstaller打包

```bash
# 创建spec文件
pyi-makespec python/main.py

# 编辑main.spec，添加动态库和.pck文件
a = Analysis(
    ['python/main.py'],
    pathex=['.'],
    binaries=[],
    datas=[
        ('godot/export/wavelet.pck', 'wavelet'),
        ('target/release/libwavelet.dylib', 'lib'),
    ],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
)

# 打包
pyinstaller main.spec --onefile
```

### 3. 启动器spec文件

```python
# wavelet.spec
a = Analysis(
    ['python/main.py'],
    pathex=['.'],
    binaries=[
        ('target/release/libwavelet.dylib', 'lib'),
        ('target/release/wavelet.dll', 'lib'),
        ('target/release/libwavelet.so', 'lib'),
    ],
    datas=[
        ('godot/export/wavelet.pck', 'wavelet'),
    ],
    hiddenimports=[],
    excludes=[],
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='wavelet',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,  # 设置为True用于调试
    icon='assets/icon.icns',
)

coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='wavelet',
)
```

---

## 📦 平台特定打包

### macOS (.dmg)

```bash
# 使用create-dmg或手动创建
brew install create-dmg

# 创建DMG
create-dmg \
  --volname "WAVELET" \
  --window-size 500 300 \
  --background "assets/dmg_background.png" \
  --icon "WAVELET.app" 150 150 \
  --app-drop-link 350 150 \
  "WAVELET-{version}.dmg" \
  "dist/WAVELET.app"
```

### Windows (.exe)

```bash
# 使用Inno Setup或NSIS
# 脚本: installer.iss
[Setup]
AppName=WAVELET
AppVersion=3.0.0
DefaultDirName={autopf}\WAVELET
DefaultGroupName=WAVELET
OutputBaseFilename=wavelet-installer
Compression=lzma2
SolidCompression=yes

[Files]
Source: "dist\wavelet.exe"; DestDir: "{app}"
Source: "dist\wavelet.pck"; DestDir: "{app}"
Source: "dist\lib\wavelet.dll"; DestDir: "{app}"
```

### Linux (.AppImage)

```bash
# 使用appimagetool
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

# 创建AppDir结构
mkdir -p WAVELET-x86_64.AppImage/home
cp -r dist/* WAVELET-x86_64.AppImage/

# 打包
./appimagetool-x86_64.AppImage WAVELET-x86_64.AppImage
```

---

## 🔄 CI/CD 自动化

### GitHub Actions 工作流

```yaml
# .github/workflows/release.yml
name: Release

on:
  release:
    types: [created]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Build Rust library
        run: cargo build --release
        
      - name: Setup Godot
        uses: kierank/godot-action@v1
        with:
          godot-version: 4.6-stable
          export-templates: true
          
      - name: Export Godot project
        run: |
          cd godot
          godot --headless --export-release "WAVELET" "../build/wavelet.pck"
          
      - name: Build Python launcher
        run: |
          python3 -m venv .venv
          source .venv/bin/activate
          pip install pyinstaller
          pyinstaller python/main.spec
          
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: wavelet-${{ matrix.os }}
          path: dist/
```

---

## 📋 打包检查清单

### 通用
- [ ] Rust库编译成功
- [ ] Godot项目导出成功
- [ ] Python启动器运行正常
- [ ] 所有资源文件包含
- [ ] 图标和品牌素材
- [ ] 版本号正确

### macOS
- [ ] 代码签名
- [ ] 公证 (Notarization)
- [ ] DMG创建
- [ ] Apple Silicon支持

### Windows
- [ ] 代码签名证书
- [ ] 病毒扫描
- [ ] 安装程序测试

### Linux
- [ ] AppImage测试
- [ ] .deb包测试
- [ ] 各桌面环境兼容性

---

## 🐛 常见问题

### 问题1: Rust库无法加载
**解决**: 检查动态库路径和依赖
```bash
# macOS
otool -L target/release/libwavelet.dylib

# Linux
ldd target/release/libwavelet.so

# Windows
dumpbin /dependents wavelet.dll
```

### 问题2: Godot导出失败
**解决**: 确保已安装导出模板
```bash
godot --export-templates
```

### 问题3: PyInstaller找不到模块
**解决**: 在spec文件中添加hidden imports
```python
hiddenimports=['ctypes', 'platform'],
```

---

## 📚 参考资源

- [PyInstaller文档](https://pyinstaller.org/)
- [Godot导出文档](https://docs.godotengine.org/)
- [Rust动态库](https://doc.rust-lang.org/reference/types.html#dynamically-sized-types)
- [AppImage打包](https://docs.appimage.org/)

---

*Made with 💕 by Nana Nakajima*
*WAVELET - Abstract Sound Synthesizer*
