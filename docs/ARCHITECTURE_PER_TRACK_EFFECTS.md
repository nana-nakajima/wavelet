# Architecture: Per-track Effects

**版本**: v1.0  
**日期**: 2026-02-03  
**关联**: PRODUCT_BRIEF_PER_TRACK_EFFECTS.md

---

## 1. 概述

Per-track Effects 模块为 WAVELET 音序器的每个轨道提供独立的效果器链支持。

### 1.1 设计目标

1. **轻量级**: 每个轨道效果器占用 < 5% CPU
2. **可扩展**: 易于添加新效果器类型
3. **高性能**: 实时音频处理，无锁设计
4. **兼容**: 与现有 Effect trait 系统无缝集成

### 1.2 架构原则

```
┌─────────────────────────────────────────────────────────────┐
│                    Track Effects System                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐   │
│  │ Track 0 │    │ Track 1 │    │ Track 2 │    │ Track 3 │   │
│  ├─────────┤    ├─────────┤    ├─────────┤    ├─────────┤   │
│  │ Filter  │    │ Distort │    │   EQ    │    │  Comp   │   │
│  │ Distort │    │   EQ    │    │  Comp   │    │  Delay  │   │
│  │   ...   │    │   ...   │    │   ...   │    │   ...   │   │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. 系统架构

### 2.1 核心组件

```
src/
├── effects/
│   ├── mod.rs           # Effect trait 定义
│   ├── track_effects.rs # Per-track effects 核心 (NEW)
│   ├── filter.rs        # 滤波器效果器
│   ├── distortion.rs    # 失真效果器
│   ├── compressor.rs    # 压缩效果器
│   └── simple_eq.rs     # 均衡效果器
├── step_sequencer.rs    # 音序器 (集成 TrackEffects)
└── lib.rs               # 模块导出
```

### 2.2 数据结构

#### TrackEffectSlot (效果器槽位)

```rust
/// 单个效果器槽位
pub struct TrackEffectSlot {
    /// 效果器类型
    effect_type: Option<EffectType>,
    
    /// 效果器实例
    effect: Option<Box<dyn Effect>>,
    
    /// 是否启用
    enabled: bool,
    
    /// Mix 比例 (0.0 = dry, 1.0 = wet)
    mix: f32,
    
    /// 参数锁映射
    param_locks: HashMap<ParameterId, f32>,
}
```

#### TrackEffects (效果器链)

```rust
/// 单个轨道的效果器链
pub struct TrackEffects {
    /// 效果器槽位 (最多4个)
    slots: [Option<TrackEffectSlot>; 4],
    
    /// 轨道 ID
    track_id: u8,
    
    /// 启用状态
    enabled: bool,
    
    /// 旁路状态
    bypass: bool,
}
```

#### PerTrackEffectsManager (管理器)

```rust
/// 全局效果器管理器
pub struct PerTrackEffectsManager {
    /// 所有轨道效果器 (8轨道)
    track_effects: [TrackEffects; 8],
    
    /// 共享效果器工厂
    effect_factory: EffectFactory,
    
    /// 缓存的预设
    presets: Vec<EffectChainPreset>,
}
```

---

## 3. 效果器工厂模式

### 3.1 EffectFactory

```rust
/// 效果器工厂 - 创建和管理效果器实例
pub struct EffectFactory {
    /// 样本率
    sample_rate: f32,
    
    /// 效果器类型注册表
    registered_types: HashMap<EffectType, EffectBuilder>,
}

impl EffectFactory {
    /// 创建指定类型的效果器
    pub fn create_effect(&self, effect_type: EffectType) -> Option<Box<dyn Effect>> {
        self.registered_types
            .get(&effect_type)
            .and_then(|builder| builder(self.sample_rate))
    }
    
    /// 注册新的效果器类型
    pub fn register<E: Effect + Default + 'static>(
        &mut self,
        effect_type: EffectType,
    ) {
        self.registered_types.insert(
            effect_type,
            Box::new(|sr| Box::new(E::new(sr))),
        );
    }
}
```

### 3.2 支持的效果器类型

| EffectType | 实现类 | CPU 估算 | 用途 |
|------------|--------|----------|------|
| Filter | BiquadFilter | ~1% | 音色塑形 |
| Saturation | Saturation | ~1% | 失真/温暖感 |
| Compressor | Compressor | ~2% | 动态控制 |
| SimpleEQ | SimpleEq | ~1% | 频率调节 |
| Chorus | Chorus | ~2% | 立体声宽度 |
| Delay | Delay | ~2% | 空间感 |

---

## 4. 数据流

### 4.1 音频处理流程

```
Input Sample
    │
    ▼
┌───────────────────┐
│  TrackEffects     │
│  ┌─────────────┐  │
│  │ Slot 0      │  │ ← 如果 enabled
│  │ (Filter)    │──┤
│  └─────────────┘  │
│  ┌─────────────┐  │
│  │ Slot 1      │  │ ← 如果 enabled
│  │ (Saturation)│──┤
│  └─────────────┘  │
│  ┌─────────────┐  │
│  │ Slot 2      │  │ ← 如果 enabled
│  │ (Compressor)│──┤
│  └─────────────┘  │
└───────────────────┘
    │
    ▼
Output Sample
```

### 4.2 实时参数更新

```
Parameter Change Event
    │
    ├───> Effect Slot (直接更新)
    │
    └───> ParamLock Manager (记录映射)
             │
             └───> Apply on Next Step (音序器同步)
```

---

## 5. 与音序器集成

### 5.1 修改 StepSequencer

```rust
pub struct StepSequencer {
    // ... 现有字段 ...
    
    /// 轨道效果器
    track_effects: PerTrackEffectsManager,
}

impl StepSequencer {
    /// 处理单轨道的音频输出
    fn process_track_output(&mut self, track_id: u8, sample: f32) -> f32 {
        // 1. 正常音序器处理
        let mut sample = self.tracks[track_id as usize].process(sample);
        
        // 2. 应用效果器链
        sample = self.track_effects.process_track(track_id, sample);
        
        sample
    }
}
```

### 5.2 与 ParameterLock 集成

```rust
impl TrackEffects {
    /// 应用参数锁到效果器
    pub fn apply_param_locks(&mut self, step: u8) {
        for slot in &mut self.slots {
            if let Some(ref mut effect) = slot.effect {
                // 查找该步骤的效果器参数锁
                for (param_id, value) in &slot.param_locks {
                    self.apply_param_to_effect(effect.as_mut(), param_id, *value);
                }
            }
        }
    }
}
```

---

## 6. 性能优化策略

### 6.1 CPU 优化

1. **效果器懒加载**: 只在效果器启用时分配内存
2. **SIMD 优化**: 对滤波器等计算密集型效果器使用 SIMD
3. **缓存友好**: 效果器状态使用栈分配，避免频繁内存访问
4. **旁路优化**: 效果器关闭时跳过所有处理

### 6.2 内存优化

```rust
// 使用 Option 避免不必要的分配
struct TrackEffects {
    slots: [Option<TrackEffectSlot>; 4], // 只有启用的效果器才会 Some
}

// 效果器实例使用 Box，但共享同一类型
effect: Option<Box<dyn Effect>>,
```

### 6.3 延迟优化

- 所有效果器处理在同一音频缓冲区完成
- 无需额外的缓冲区复制
- 预估延迟: < 0.5ms (44.1kHz @ 22 samples)

---

## 7. 错误处理

### 7.1 效果器创建失败

```rust
impl TrackEffects {
    pub fn add_effect(&mut self, slot_index: usize, effect_type: EffectType) -> Result<(), Error> {
        if slot_index >= 4 {
            return Err(Error::InvalidSlotIndex);
        }
        
        let effect = self.factory.create_effect(effect_type)
            .ok_or(Error::UnsupportedEffect)?;
            
        self.slots[slot_index] = Some(TrackEffectSlot {
            effect_type: Some(effect_type),
            effect: Some(effect),
            enabled: true,
            mix: 0.5,
            param_locks: HashMap::new(),
        });
        
        Ok(())
    }
}
```

### 7.2 状态恢复

```rust
impl TrackEffects {
    /// 从快照恢复效果器状态
    pub fn from_snapshot(snapshot: &TrackEffectsSnapshot) -> Self {
        let mut effects = Self::new(snapshot.track_id);
        
        for (i, slot_snapshot) in snapshot.slots.iter().enumerate() {
            if let Some(ref effect_type) = slot_snapshot.effect_type {
                effects.add_effect(i, *effect_type).unwrap();
                
                if let Some(ref mut slot) = effects.slots[i] {
                    slot.enabled = slot_snapshot.enabled;
                    slot.mix = slot_snapshot.mix;
                }
            }
        }
        
        effects
    }
}
```

---

## 8. 测试策略

### 8.1 单元测试

| 测试项 | 覆盖内容 |
|--------|----------|
| `test_track_effects_creation` | 创建和初始化 |
| `test_add_remove_effects` | 添加/移除效果器 |
| `test_effect_processing` | 效果处理正确性 |
| `test_bypass_behavior` | 旁路行为 |
| `test_mix_parameter` | Mix 参数 |
| `test_param_locks` | 参数锁集成 |

### 8.2 集成测试

- 音序器 + 效果器集成
- 8 轨道效果器性能测试
- 效果器切换无杂音测试

### 8.3 性能测试

```rust
#[test]
fn test_track_effects_performance() {
    let mut effects = PerTrackEffectsManager::new(44100.0);
    
    // 添加效果器到所有轨道
    for track_id in 0..8 {
        effects.add_effect(track_id, 0, EffectType::Filter).unwrap();
        effects.add_effect(track_id, 1, EffectType::Saturation).unwrap();
    }
    
    // 性能测试: 1000次处理
    let start = Instant::now();
    for _ in 0..1000 {
        for track_id in 0..8 {
            let _ = effects.process_track(track_id, 0.5);
        }
    }
    let duration = start.elapsed();
    
    // 应该在合理时间内完成
    assert!(duration.as_secs_f32() < 0.1);
}
```

---

## 9. 未来扩展

### 9.1 可扩展效果器

```rust
// 注册新效果器
effect_factory.register::<CustomEffect>(EffectType::Custom);

// 效果器自动出现在 UI 选择列表中
```

### 9.2 效果器预设系统

```rust
pub struct EffectChainPreset {
    name: String,
    author: String,
    slots: [Option<EffectSlotConfig>; 4],
    tags: Vec<String>,
}

// 预设可以保存/分享
```

### 9.3 高级效果器

- Reverb (卷积或算法混响)
- Delay (立体声延迟)
- Phaser (相位效果)
- Granular (粒子效果)

---

## 10. 决策记录

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| 效果器槽位数 | 4 / 8 | 4 | 平衡功能和 CPU |
| 效果器执行顺序 | 固定/可调 | 固定 (Filter→Dist→EQ→Comp) | 简化 UI，降低复杂度 |
| 参数锁粒度 | 步骤/小节 | 步骤 | 与音序器现有系统一致 |
| 效果器类型 | 基础/高级 | 基础 (P0优先) | 快速交付核心功能 |

---

*本文档基于 WAVELET 开发工作流 (BMAD-METHOD) 创建*
