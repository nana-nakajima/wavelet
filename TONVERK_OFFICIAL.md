# Tonverk Official Feature Analysis

**创建日期**: 2026-02-04
**来源**: Tonverk User Manual (OS1.0.2, 2025-10-27)
**状态**: 🔥 进行中

---

## 关于 Tonverk

> "Tonverk is an instrument synonymous with the journey, with adventure."
> — Elektron

Elektron于2025年推出的新一代音乐工作站，结合了采样播放、合成器建模和强大的音序功能。

---

## 核心架构

### 音轨系统 (Tracks)

| 音轨类型 | 数量 | 说明 |
|---------|------|------|
| 音频音轨 | 8 | 多音色立体声采样播放 |
| MIDI/总线音轨 | 8 | 支持MIDI控制和总线处理 |
| **总计** | **16** | 灵活组合 |

### 模式系统 (Patterns)

| 属性 | 值 |
|------|-----|
| 模式数量 | 128 (16 banks × 8 patterns) |
| 每模式步数 | 最多64步 |
| 每步参数 | 音符、力度、门长度、概率、条件 |

### 歌曲系统 (Songs)

| 属性 | 值 |
|------|-----|
| 歌曲数量 | 16 |
| 每歌曲行数 | 最多99行 |
| 每行设置 | 模式选择、重复次数、长度修正、速度修正 |

---

## 音序器功能

### 步进功能

- **音符**: 音符编号、力度
- **门长度**: 音符持续时间
- **参数锁定**: 每步可独立设置音高、音高偏移、音量、声像、调制
- **触发条件**:
  - Normal: 正常触发
  - Probability: 概率触发 (0-100%)
  - Mute: 静音该步
  - Solo: 只触发该步

### Swing/Shuffle

- **Swing**: 偶数步延迟，产生摇摆感
- **Shuffle**: 基于音符时值的随机化
- 两者可组合使用

### 微调 (Micro Timing)

- 每步可独立调整时间偏移 (-80 to +80)

---

## 效果器系统

### 插入效果 (Insert Effects)

每个音轨最多可添加10+种效果：

| 效果类型 | 说明 |
|---------|------|
| Amp | 放大器模拟 |
| Bass | 低频增强 |
| Bit Crush | 比特破碎 |
| Chorus | 合唱效果 |
| Compressor | 压缩器 |
| Decay | 衰减控制 |
| Delay | 延迟效果 |
| Distortion | 失真效果 |
| Drive | 驱动效果 |
| EQ | 均衡器 |
| Filter | 滤波器 |
| Flanger | 镶边效果 |
| LFO Filter | LFO调制滤波器 |
| Overdrive | 过载效果 |
| Phaser | 相位效果 |
| Reverb | 混响效果 |
| Ring Mod | 环形调制 |
| Tremolo | 颤音效果 |
| Utility |  Utility (Utility) |
| Wave | 波形塑形 |

### 发送效果 (Send Effects)

| 发送轨 | 效果槽 | 说明 |
|--------|--------|------|
| Send A | 2个 | 可串行或并行连接 |
| Send B | 2个 | 可串行或并行连接 |
| Send C | 2个 | 可串行或并行连接 |

### 效果返回 (Returns)

| 返回轨 | 通道数 | 用途 |
|--------|--------|------|
| Return A | 立体声 | Send A的返回 |
| Return B | 立体声 | Send B的返回 |
| Return C | 立体声 | Send C的返回 |

---

## 采样系统

### 采样格式

| 属性 | 支持格式 |
|------|---------|
| 格式 | .wav, .aiff |
| 位深度 | 16-bit, 24-bit |
| 采样率 | 44.1kHz, 48kHz, 88.2kHz, 96kHz |

### 采样管理

- 每个音轨可加载多个采样
- 支持跨音高分层 (Key zones)
- 支持循环 (Loop)
- 支持时间拉伸 (Time Stretch)
- 支持音高偏移 (Pitch Shift)

### 采样参数

| 参数 | 范围 | 说明 |
|------|------|------|
| Start | 0-100% | 起始位置 |
| Length | 1-100% | 长度 |
| Loop | On/Off | 循环开关 |
| Crossfade | 0-100ms | 交叉淡化 |
| Reverse | On/Off | 反向播放 |
| Gain | -inf to +24dB | 增益 |
| Pan | -100 to +100 | 声像 |
| Pitch | -12 to +12 semitones | 音高偏移 |
| Time Stretch | 25-400% | 时间拉伸比例 |

---

## 合成器参数

### 振荡器 (Oscillators)

| 参数 | 范围 | 说明 |
|------|------|------|
| Waveform | Sine, Triangle, Saw, Square, Noise | 波形选择 |
| Pitch | -12 to +12 semitones | 音高偏移 |
| Sync | On/Off | 振荡器同步 |
| Ring Mod | On/Off | 环形调制 |
| Mix | 0-100% | 混合比例 |

### 滤波器 (Filter)

| 参数 | 范围 | 说明 |
|------|------|------|
| Type | Low Pass, Band Pass, High Pass | 滤波器类型 |
| Cutoff | 20Hz-20kHz | 截止频率 |
| Resonance | 0-100% | 共振 |
| Drive | 0-100% | 驱动量 |
| Envelope | 0-100% | 包络影响 |

### 包络 (Envelope)

| 包络类型 | 参数 |
|---------|------|
| Amp Envelope | Attack, Decay, Sustain, Release |
| Filter Envelope | Attack, Decay, Sustain, Release, Amount |
| Mod Envelope | Attack, Decay, Sustain, Release, Amount |

### LFO (Low Frequency Oscillator)

| 参数 | 范围 | 说明 |
|------|------|------|
| Waveform | Sine, Triangle, Saw, Square, Random | 波形 |
| Rate | 0.01Hz-100Hz | 速率 |
| Sync | Off, Tempo | 同步模式 |
| Retrigger | On/Off | 重触发 |
| One Shot | On/Off | 单次触发 |
| Fade In | 0-10s | 淡入时间 |

---

## MIDI 功能

### MIDI 输入

| 功能 | 说明 |
|------|------|
| Note Input | 音符触发 |
| Velocity | 力度感应 |
| Note Off | 音符释放 |
| Sustain Pedal | 延音踏板 |
| Mod Wheel | 调制轮 |
| Pitch Bend | 弯音轮 |
| Aftertouch | Aftertouch (Poly, Channel) |
| MIDI CC | 支持16个可分配CC |
| Program Change | 程序切换 |
| Bank Change | 银行切换 |

### MIDI 输出

| 功能 | 说明 |
|------|------|
| MIDI Clock | 同步时钟 |
| MIDI Start/Stop | 开始/停止 |
| MIDI Continue | 继续 |
| MIDI CC | 发送CC值 |

---

## 项目管理

### 保存/加载

| 功能 | 说明 |
|------|------|
| 项目保存 | 完整保存所有设置 |
| 项目加载 | 加载已保存项目 |
| 工厂预设 | 预装音色 |
| 用户预设 | 用户创建音色 |
| 导入/导出 | 跨项目共享 |

### 备份

| 功能 | 说明 |
|------|------|
| 自动备份 | 定期自动保存 |
| 手动备份 | 用户手动创建 |
| 云同步 | 可选云端备份 |

---

## WAVELET 功能对比

### 核心功能对比

| 类别 | Tonverk | WAVELET | 差距 | 优先级 |
|------|---------|---------|------|--------|
| 音轨数 | 16 (8音频+8MIDI) | 8 | -8 | 中 |
| 模式数 | 128 (16 banks × 8) | 8 | -120 | 中 |
| 歌曲数 | 16 | ❌ | -16 | 🔥 **最高** |
| 插入效果 | 10+种 | 15种 | +5 | ✅ |
| 发送效果 | 3轨 | ❌ | -3 | 🔥 **高** |
| 参数锁定 | ✅ | ✅ | 0 | ✅ |
| 概率触发 | ✅ | ✅ | 0 | ✅ |
| Swing/Shuffle | ✅ | ✅ | 0 | ✅ |
| 采样播放 | ✅ | ✅ | 0 | ✅ |
| 时间拉伸 | ✅ | ❌ | -1 | 🔥 **高** |
| 项目保存 | ✅ | ❌ | -1 | 🔥 **中** |
| MIDI CC映射 | ✅ | ⚠️ 部分 | - | ⚠️ |
| 多采样乐器 | ✅ | ⚠️ 部分 | - | ⚠️ |
| Subtracks | ✅ | ❌ | - | ❌ 低 |

### 效果器对比

| Tonverk效果 | WAVELET实现 | 状态 |
|------------|-------------|------|
| Amp | Distortion + EQ | ✅ |
| Bass | Filter + Saturation | ✅ |
| Bit Crush | BitCrusher | ✅ |
| Chorus | Chorus | ✅ |
| Compressor | Compressor | ✅ |
| Decay | Envelope | ✅ |
| Delay | Delay | ✅ |
| Distortion | Distortion | ✅ |
| Drive | Saturation | ✅ |
| EQ | SimpleEQ | ✅ |
| Filter | FilterBank | ✅ |
| Flanger | Flanger | ✅ |
| LFO Filter | LFO + Filter | ✅ |
| Overdrive | Distortion | ✅ |
| Phaser | Phaser | ✅ |
| Reverb | Reverb | ✅ |
| Ring Mod | RingMod | ✅ |
| Tremolo | Tremolo | ✅ |
| Utility | Utility effects | ✅ |
| Wave | Oscillator + Mod | ✅ |

---

## 缺失功能实现计划

### 🔥 最高优先级

#### 1. 歌曲模式 (Song Mode)

**需求**:
- 16首歌曲容量
- 每首最多99行
- 每行设置：模式选择、重复次数、长度修正、速度修正
- 模式链播放

**实现方案**:
```rust
struct Song {
    id: u8,
    name: String,
    lines: Vec<SongLine>,
}

struct SongLine {
    pattern_id: u8,
    repeats: u8,          // 重复次数
    length_mod: f32,       // 长度修正 (0.5x - 2.0x)
    tempo_mod: f32,        // 速度修正 (0.5x - 2.0x)
}

impl Synth {
    fn play_song(&mut self, song_id: u8) {
        // 实现歌曲播放逻辑
    }
    
    fn song_to_pattern_chain(&self, song: &Song) -> PatternChain {
        // 将歌曲转换为模式链
    }
}
```

**测试用例**:
- 单行歌曲播放
- 多行歌曲播放
- 重复次数设置
- 速度和长度修改

#### 2. 发送效果架构 (Send FX)

**需求**:
- 3个发送效果轨 (Send A, Send B, Send C)
- 每个发送轨2个效果槽
- 可串行或并行连接
- 独立发送量控制
- 立体声返回

**实现方案**:
```rust
struct SendEffect {
    name: String,
    slots: [Option<Effect>; 2],
    connection: Connection,  // Serial or Parallel
    pre_post: PrePost,       // Pre-FX or Post-FX
}

struct SendReturn {
    effect: SendEffect,
    return_level: f32,
    return_pan: f32,
}

impl Track {
    fn add_send(&mut self, send_id: usize, send_level: f32) {
        // 添加发送
    }
    
    fn set_return(&mut self, return_id: usize, level: f32, pan: f32) {
        // 设置返回电平
    }
}
```

**测试用例**:
- 单发送效果
- 串联发送效果
- 并联发送效果
- 发送量控制
- 返回电平设置

### 🔧 高优先级

#### 3. 时间拉伸 (Time Stretch)

**需求**:
- 实时时间拉伸
- 保持音高不变
- 25%-400%拉伸范围
- 高质量拉伸算法

**实现方案**:
```rust
struct TimeStretch {
    algorithm: StretchAlgorithm,
    grain_size: f32,
    overlap: f32,
    pitch_preservation: bool,
}

enum StretchAlgorithm {
    Elastique,     // 高质量
    Simple,         // 简单算法
    Complex,        // 复杂算法
}

impl Sample {
    fn time_stretch(&mut self, ratio: f32, stretch: &TimeStretch) {
        match stretch.algorithm {
            StretchAlgorithm::Elastique => {
                // Elastique Pro算法
            }
            StretchAlgorithm::Simple => {
                // 简单的重采样
            }
            StretchAlgorithm::Complex => {
                // 复合算法
            }
        }
    }
}
```

**测试用例**:
- 简单拉伸 (1.5x)
- 大幅度拉伸 (0.25x, 4.0x)
- 拉伸同时保持音高
- 拉伸质量对比

#### 4. 项目保存/加载

**需求**:
- JSON格式保存
- 包含所有设置和模式
- 跨平台兼容
- 增量保存

**实现方案**:
```rust
#[derive(Serialize, Deserialize)]
struct Project {
    version: String,
    name: String,
    settings: ProjectSettings,
    tracks: Vec<Track>,
    patterns: Vec<Pattern>,
    songs: Vec<Song>,
    midi_config: MidiConfig,
}

impl Project {
    fn save(&self, path: &Path) -> Result<(), ProjectError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    fn load(&mut self, path: &Path) -> Result<(), ProjectError> {
        let json = std::fs::read_to_string(path)?;
        *self = serde_json::from_str(&json)?;
        Ok(())
    }
}
```

**测试用例**:
- 保存项目
- 加载项目
- 保存/加载完整性
- 版本兼容

---

## 实施路线图

### Phase 1: 核心缺失功能

| 功能 | 预计时间 | 优先级 |
|------|---------|--------|
| 歌曲模式 | 3天 | 🔥 最高 |
| 发送效果 | 4天 | 🔥 高 |
| 时间拉伸 | 5天 | 🔥 高 |
| 项目保存/加载 | 2天 | 中 |

### Phase 2: 功能增强

| 功能 | 预计时间 | 优先级 |
|------|---------|--------|
| 完整MIDI CC映射 | 2天 | 中 |
| 多采样乐器增强 | 3天 | 中 |
| 音轨数扩展 | 2天 | 低 |
| 模式数扩展 | 2天 | 低 |

### Phase 3: 优化和完善

| 功能 | 预计时间 | 优先级 |
|------|---------|--------|
| 性能优化 | 持续 | 低 |
| Bug修复 | 持续 | 中 |
| 文档完善 | 持续 | 低 |
| 测试覆盖 | 持续 | 中 |

---

## 参考资源

- Tonverk User Manual: OS1.0.2 (2025-10-27)
- WAVELET项目: /Users/n3kjm/clawd/wavelet/
- Tonverk官网: https://www.elektron.se/en/tonverk

---

*文档创建: 2026-02-04*
*最后更新: 2026-02-04*
