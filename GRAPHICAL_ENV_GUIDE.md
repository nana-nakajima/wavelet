# 🚀 图形环境快速操作指南

**创建时间**: 2026-02-04 09:15 AM
**目的**: 在有图形环境时快速完成WAVELET导出和录制工作

---

## ⚡ 快速开始

### Step 1: 启动Godot编辑器
```bash
/usr/local/bin/godot --path /Users/n3kjm/clawd/wavelet/godot
```

### Step 2: 下载Export Templates (首次)
1. 点击菜单: **Editor** → **Manage Export Templates**
2. 点击 **Download and Install**
3. 等待下载完成 (~1GB)

### Step 3: 导出Mac OSX版本
1. 打开项目: **Project** → **Export**
2. 选择 **Mac OSX** preset
3. 勾选 **Runnable** checkbox
4. 点击 **Export Project**
5. 保存为: `export/wavelet_mac.pck`

---

## 📋 详细步骤

### A. Godot导出流程

```bash
# 1. 启动Godot (带项目路径)
open -a /usr/local/bin/godot --args --path /Users/n3kjm/clawd/wavelet/godot

# 或者直接在终端启动
/usr/local/bin/godot --path /Users/n3kjm/clawd/wavelet/godot &
```

**导出步骤**:
1. 菜单: **Project** → **Export**
2. 点击 **Add...** → 选择 **Mac OSX**
3. 配置:
   - Name: `WAVELET`
   - Bundle Name: `com.wavelet.app`
   - Identifier: `com.wavelet.mac`
   - Version: `3.0.0`
4. 勾选 **Export With Debug** (可选)
5. 勾选 **Runnable**
6. 点击 **Export Project**
7. 选择路径: `/Users/n3kjm/clawd/wavelet/godot/export/wavelet_mac.pck`
8. 点击 **Save**

### B. 宣传视频录制流程

参考: `VIDEO_RECORDING_GUIDE.md`

**录制工具**:
- OBS Studio (推荐): `brew install obs`
- QuickTime: 直接打开使用

**录制步骤**:
1. 启动Godot项目
2. 打开OBS，添加窗口捕获
3. 按场景录制 (参考VIDEO_RECORDING_GUIDE.md)
4. 录制旁白 (使用语音录制软件)
5. 后期剪辑 (使用DaVinci Resolve或Final Cut Pro)

---

## 🔧 常见问题解决

### Q: Godot无法启动
```bash
# 检查Godot是否安装
which godot

# 使用绝对路径
/usr/local/bin/godot --path /Users/n3kjm/clawd/wavelet/godot
```

### Q: Export Templates下载失败
- 手动下载: https://github.com/godotengine/godot/releases/download/4.6/Godot_v4.6_export_templates.tpz
- 解压到: `~/.local/share/godot/export_templates/4.6.stable/`

### Q: 导出时提示"Export templates not found"
- 菜单: **Editor** → **Manage Export Templates**
- 确保显示 "Installed: 4.6.stable"

### Q: 录制时鼠标看不见
- OBS设置: Tools → Virtual Camera → 启用鼠标高亮
- 或使用: https://github.com/MuhammadDaniyal/OBS-Mouse-Highlight

---

## ✅ 完成检查清单

### Godot导出
- [ ] Godot编辑器启动成功
- [ ] Export Templates已下载安装
- [ ] Export窗口可正常打开
- [ ] Mac OSX预设已配置
- [ ] 勾选Runnable
- [ ] 导出为 `export/wavelet_mac.pck`
- [ ] 验证文件大小 (应该 > 1MB)

### 宣传视频录制
- [ ] OBS Studio已安装
- [ ] 录制场景1: 开场Hook
- [ ] 录制场景2: 快速上手演示
- [ ] 录制场景3: AI旋律生成
- [ ] 录制场景4: 效果器展示
- [ ] 录制场景5: 社区分享
- [ ] 录制场景6: 结尾CTA
- [ ] 录制中文旁白
- [ ] 录制英文旁白
- [ ] 后期制作完成
- [ ] 导出最终视频 `wavelet_trailer.mp4`

### 完整打包测试
- [ ] Rust动态库构建成功
- [ ] Godot .pck导出成功
- [ ] 打包脚本测试通过
- [ ] 可执行文件运行正常

---

## 📁 文件位置参考

| 文件 | 路径 |
|------|------|
| Godot项目 | `/Users/n3kjm/clawd/wavelet/godot/` |
| 导出目录 | `/Users/n3kjm/clawd/wavelet/godot/export/` |
| Rust库 | `/Users/n3kjm/clawd/wavelet/target/release/libwavelet.dylib` |
| Steam素材 | `/Users/n3kjm/clawd/wavelet/steam_assets/` |
| 截图目录 | `/Users/n3kjm/clawd/wavelet/godot/screenshots/` |
| 视频脚本 | `/Users/n3kjm/clawd/wavelet/VIDEO_RECORDING_GUIDE.md` |
| 打包文档 | `/Users/n3kjm/clawd/wavelet/PACKAGING.md` |
| Steam配置 | `/Users/n3kjm/clawd/wavelet/STEAMWORKS.md` |

---

## 🎯 预期产出

在图形环境中完成以下工作后：

1. **导出文件**:
   - `godot/export/wavelet_mac.pck` - Godot UI包
   - `target/release/libwavelet.dylib` - Rust动态库

2. **宣传视频**:
   - `wavelet_trailer.mp4` - 60-90秒宣传视频

3. **完整打包**:
   - `wavelet_mac.app` - 可执行应用
   - 或 `wavelet_installer.dmg` - 安装包

4. **Steam上传**:
   - 6种商店素材
   - 24张游戏截图
   - 宣传视频
   - 商店页面文案

---

**完成后更新**: 在TASKS_INDEX.md中标记为完成

---

*Created by Nana Nakajima - 2026-02-04*
