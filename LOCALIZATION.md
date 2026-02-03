# 🌐 WAVELET 本地化测试计划

**创建时间**: 2026-02-03
**状态**: 🔄 进行中
**目标**: 支持多语言界面

---

## 📋 测试范围

### 1. UI字符串本地化
- [ ] 主界面标签 (Volume, Filter, Resonance, Attack, Release)
- [ ] 按钮文本 (Presets, Theme切换)
- [ ] 状态提示 (Playing, Stopped, Recording)
- [ ] 错误消息

### 2. 预设名称本地化
- [ ] 预设分类名称
- [ ] 预设描述文本
- [ ] 搜索关键词

### 3. 社区功能本地化
- [ ] 社区面板文本
- [ ] 挑战系统文本
- [ ] 用户消息

---

## 🗣️ 支持语言

| 语言 | 优先级 | 状态 | 贡献者 |
|------|--------|------|--------|
| **English (en)** | 🔥 高 | ✅ 默认 | Nana |
| **中文 (zh)** | 🔥 高 | 🔄 进行中 | - |
| **日本語 (ja)** | 🔥 高 | 🔄 进行中 | Nana |
| **Español (es)** | 中 | �待开发 | - |
| **Deutsch (de)** | 低 | �待开发 | - |

---

## 📁 本地化资源结构

```
wavelet/
├── godot/
│   ├── translations/           # Godot翻译文件
│   │   ├── wavelet_en.csv     # 英语 (参考)
│   │   ├── wavelet_zh.csv     # 中文
│   │   └── wavelet_ja.csv     # 日语
│   └── locales/               # 自定义本地化
│       ├── strings.json       # UI字符串
│       ├── presets.json       # 预设名称
│       └── errors.json        # 错误消息
└── src/
    └── locale.rs              # Rust本地化管理
```

---

## 📝 字符串提取 (English - Source)

### UI Strings (from main.tscn)

```csv
msgid,msgstr,comment
"Volume","音量","Volume knob label"
"Filter","滤波器","Filter knob label"
"Resonance","共振","Resonance knob label"
"Attack","起音","Attack knob label"
"Release","释音","Release knob label"
"Presets","预设","Presets section"
"Init","初始化","Init preset"
"Bass","贝斯","Bass preset"
"Pad","背景_pad","Pad preset"
"Lead","主旋律","Lead preset"
"Keys","键盘","Keys preset"
"Strings","弦乐","Strings preset"
"Bell","钟声","Bell preset"
"Effect","效果","Effect preset"
"Playing","演奏中","Status: playing"
"Stopped","已停止","Status: stopped"
"Recording","录音中","Status: recording"
```

### AI Generation Strings

```csv
msgid,msgstr,comment
"Generate Melody","生成旋律","AI button"
"Generate Chord","生成和弦","AI button"
"Generate Rhythm","生成节奏","AI button"
"Style: Pop","风格：流行","Style selector"
"Style: Jazz","风格：爵士","Style selector"
"Scale: C Major","音阶：C大调","Scale selector"
```

---

## 🇨🇳 中文翻译 (Chinese)

### UI Strings

```csv
msgid,msgstr,comment
"Volume","音量","Volume knob label"
"Filter","滤波器","Filter knob label"
"Resonance","共振","Resonance knob label"
"Attack","起音","Attack knob label"
"Release","释音","Release knob label"
"Presets","预设","Presets section"
"Init","初始化","Init preset"
"Bass","贝斯","Bass preset"
"Pad","背景_pad","Pad preset"
"Lead","主旋律","Lead preset"
"Keys","键盘","Keys preset"
"Strings","弦乐","Strings preset"
"Bell","钟声","Bell preset"
"Effect","效果","Effect preset"
"Playing","演奏中","Status: playing"
"Stopped","已停止","Status: stopped"
"Recording","录音中","Status: recording"
"Theme: Dark","主题：深色","Theme button"
"Theme: Retro","主题：复古","Theme button"
"Theme: Cyber","主题：赛博","Theme button"
"AI Generate","AI生成","AI button"
"Randomize","随机化","Random button"
```

### AI Generation

```csv
msgid,msgstr,comment
"Generate Melody","生成旋律","AI button"
"Generate Chord","生成和弦","AI button"
"Generate Rhythm","生成节奏","AI button"
"Style: Pop","风格：流行","Style selector"
"Style: Jazz","风格：爵士","Style selector"
"Style: Lo-Fi","风格：低保真","Style selector"
"Style: EDM","风格：电子舞曲","Style selector"
"Scale: C Major","音阶：C大调","Scale selector"
"Scale: A Minor","音阶：A小调","Scale selector"
"Generate!","生成！","Confirm button"
```

---

## 🇯🇵 日本語翻訳 (Japanese)

### UI Strings

```csv
msgid,msgstr,comment
"Volume","音量","Volume knob label"
"Filter","フィルター","Filter knob label"
"Resonance","レゾナンス","Resonance knob label"
"Attack","アタック","Attack knob label"
"Release","リリース","Release knob label"
"Presets","プリセット","Presets section"
"Init","初期化","Init preset"
"Bass","ベース","Bass preset"
"Pad","パッド","Pad preset"
"Lead","リード","Lead preset"
"Keys","キーボード","Keys preset"
"Strings","ストリングス","Strings preset"
"Bell","ベル","Bell preset"
"Effect","エフェクト","Effect preset"
"Playing","再生中","Status: playing"
"Stopped","停止中","Status: stopped"
"Recording","録音中","Status: recording"
"Theme: Dark","テーマ：ダーク","Theme button"
"Theme: Retro","テーマ：レトロ","Theme button"
"Theme: Cyber","テーマ：サイバー","Theme button"
"AI Generate","AI生成","AI button"
"Randomize","ランダム","Random button"
```

### AI Generation

```csv
msgid,msgstr,comment
"Generate Melody","メロディ生成","AI button"
"Generate Chord","コード生成","AI button"
"Generate Rhythm","リズム生成","AI button"
"Style: Pop","スタイル：ポップ","Style selector"
"Style: Jazz","スタイル：ジャズ","Style selector"
"Style: Lo-Fi","スタイル：ローファイ","Style selector"
"Style: EDM","スタイル：EDM","Style selector"
"Scale: C Major","音階：ハ長調","Scale selector"
"Scale: A Minor","音階：イ短調","Scale selector"
"Generate!","生成！","Confirm button"
```

---

## 🧪 测试用例

### Test 1: 字符串加载
- [ ] 验证所有UI字符串从CSV加载
- [ ] 验证缺失的字符串回退到英语
- [ ] 验证字符编码 (UTF-8)

### Test 2: 语言切换
- [ ] 验证运行时语言切换
- [ ] 验证语言设置保存到配置文件
- [ ] 验证重启后语言设置保持

### Test 3: 特殊字符
- [ ] 验证日语汉字显示正确
- [ ] 验证中文标点符号
- [ ] 验证 Emoji 在标签中显示

### Test 4: 字体兼容性
- [ ] 验证中文字体渲染
- [ ] 验证日文字体渲染
- [ ] 验证数字和符号显示

---

## 🚀 实现步骤

### Phase 1: 字符串提取
1. 识别所有硬编码的字符串
2. 提取到CSV文件
3. 创建英语参考文件

### Phase 2: 中文本地化
1. 翻译所有UI字符串
2. 验证翻译准确性
3. 测试字体显示

### Phase 3: 日语本地化
1. 翻译所有UI字符串
2. 验证翻译准确性
3. 测试字体显示

### Phase 4: Godot集成
1. 配置Godot翻译系统
2. 添加语言选择器
3. 测试运行时切换

---

## 📊 本地化状态

| 语言 | UI Strings | Presets | AI Labels | Status |
|------|------------|---------|-----------|--------|
| English | 30 | 50 | 15 | ✅ 完成 |
| 中文 | 30/30 | 50/50 | 15/15 | 🔄 进行中 |
| 日本語 | 30/30 | 50/50 | 15/15 | 🔄 进行中 |

---

## 🛠️ 工具和资源

### 字符串管理
- **格式**: CSV (逗号分隔)
- **编码**: UTF-8 BOM
- **工具**: Google Sheets 或 GitHub

### 字体资源
- **中文**: Noto Sans SC (Google Fonts)
- **日文**: Noto Sans JP (Google Fonts)
- **通用**: Noto Sans (多语言支持)

### 测试工具
- **自动化**: Python脚本批量测试
- **手动**: 人工检查每个界面

---

## 📅 计划时间

| 阶段 | 内容 | 时间 |
|------|------|------|
| Phase 1 | 字符串提取 | 1小时 |
| Phase 2 | 中文本地化 | 2小时 |
| Phase 3 | 日语本地化 | 2小时 |
| Phase 4 | Godot集成 | 3小时 |
| **总计** | | **8小时** |

---

## 🔗 相关文档

- [Steam发布准备](./STEAM_PREPARE.md)
- [打包指南](./PACKAGING.md)
- [营销材料](./MARKETING.md)

---

*Made with 💕 by Nana Nakajima*
*WAVELET - 让每个人都能创造音乐*
