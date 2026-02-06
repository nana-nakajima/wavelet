# Stories: Per-track Effects

**Version**: v1.0
**Date**: 2026-02-03
**Related**: 
- PRODUCT_BRIEF_PER_TRACK_EFFECTS.md
- ARCHITECTURE_PER_TRACK_EFFECTS.md

---

## Epic: Per-track Effects 核心功能

**目标**: 为 WAVELET 音序器的每个轨道添加独立效果器支持

---

### Story 1: 效果器槽位数据结构

**作为** 开发者  
**我希望** 创建效果器槽位的数据结构  
**以便** 为每个轨道存储效果器配置

**验收标准**:
- [ ] `TrackEffectSlot` 结构体定义完成
- [ ] 支持效果器类型存储 (`Option<EffectType>`)
- [ ] 支持效果器实例存储 (`Option<Box<dyn Effect>>`)
- [ ] 支持启用状态存储
- [ ] 支持 Mix 参数存储
- [ ] 支持参数锁存储 (`HashMap<ParameterId, f32>`)
- [ ] 单元测试覆盖

**技术笔记**:
- 相关文件: `src/effects/track_effects.rs`
- 依赖: 无
- 估算: 0.5天

**任务**:
- [ ] 创建 `TrackEffectSlot` 结构体
- [ ] 实现 `Default` trait
- [ ] 实现基本 getter/setter
- [ ] 编写单元测试

---

### Story 2: 轨道效果器链

**作为** 开发者  
**我希望** 创建轨道效果器链管理器  
**以便** 管理单个轨道的所有效果器

**验收标准**:
- [ ] `TrackEffects` 结构体定义完成
- [ ] 支持最多4个效果器槽位
- [ ] 支持启用/旁路状态
- [ ] 支持添加/移除效果器
- [ ] 支持效果器处理方法
- [ ] 单元测试覆盖

**技术笔记**:
- 相关文件: `src/effects/track_effects.rs`
- 依赖: Story 1
- 估算: 0.5天

**任务**:
- [ ] 创建 `TrackEffects` 结构体
- [ ] 实现 `new()` 构造函数
- [ ] 实现 `add_effect()` 方法
- [ ] 实现 `remove_effect()` 方法
- [ ] 实现 `process()` 方法 (效果链处理)
- [ ] 编写单元测试

---

### Story 3: 效果器工厂

**作为** 开发者  
**我希望** 创建效果器工厂类  
**以便** 统一管理效果器的创建和销毁

**验收标准**:
- [ ] `EffectFactory` 结构体定义完成
- [ ] 支持创建基础效果器 (Filter, Saturation, Compressor, EQ)
- [ ] 支持注册新效果器类型
- [ ] 支持效果器类型查询
- [ ] 单元测试覆盖

**技术笔记**:
- 相关文件: `src/effects/track_effects.rs`
- 依赖: Story 2
- 估算: 0.5天

**任务**:
- [ ] 创建 `EffectFactory` 结构体
- [ ] 实现 `create_effect()` 方法
- [ ] 实现 `register()` 方法
- [ ] 注册基础效果器类型
- [ ] 编写单元测试

---

### Story 4: 全局效果器管理器

**作为** 开发者  
**我希望** 创建全局效果器管理器  
**以便** 统一管理8个轨道的效果器

**验收标准**:
- [ ] `PerTrackEffectsManager` 结构体定义完成
- [ ] 支持8个独立轨道效果器
- [ ] 支持批量启用/禁用
- [ ] 支持全局旁路
- [ ] 支持快照保存/恢复
- [ ] 单元测试覆盖

**技术笔记**:
- 相关文件: `src/effects/track_effects.rs`
- 依赖: Story 3
- 估算: 0.5天

**任务**:
- [ ] 创建 `PerTrackEffectsManager` 结构体
- [ ] 实现 `new()` 构造函数
- [ ] 实现 `process_track()` 方法
- [ ] 实现 `set_bypass()` 方法
- [ ] 实现 `to_snapshot()` / `from_snapshot()` 方法
- [ ] 编写单元测试

---

### Story 5: 与音序器集成

**作为** 开发者  
**我希望** 将效果器系统集成到音序器中  
**以便** 音序器可以处理带效果的音频

**验收标准**:
- [ ] `StepSequencer` 包含 `PerTrackEffectsManager`
- [ ] 音序器处理时应用效果器
- [ ] 效果器处理不影响时序
- [ ] 集成测试通过

**技术笔记**:
- 相关文件: `src/step_sequencer.rs`
- 依赖: Story 4 + 完成的效果器模块
- 估算: 0.5天

**任务**:
- [ ] 修改 `StepSequencer` 添加 `track_effects` 字段
- [ ] 实现 `process_track_output()` 方法
- [ ] 修改 `process()` 方法集成效果处理
- [ ] 编写集成测试

---

### Story 6: 效果器参数控制

**作为** 用户  
**我希望** 调整效果器的参数  
**以便** 定制每个轨道的声音

**验收标准**:
- [ ] 支持设置 Mix 比例
- [ ] 支持启用/禁用单个效果器
- [ ] 支持设置效果器类型特有参数
- [ ] 效果器状态实时更新
- [ ] 单元测试覆盖

**技术笔记**:
- 相关文件: `src/effects/track_effects.rs`
- 依赖: Story 4
- 估算: 0.5天

**任务**:
- [ ] 实现 `set_mix()` 方法
- [ ] 实现 `set_enabled()` 方法
- [ ] 实现 `set_effect_param()` 方法
- [ ] 编写单元测试

---

### Story 7: 参数锁集成

**作为** 用户  
**我希望** 为效果器参数设置参数锁  
**以便** 在特定步骤改变效果器参数

**验收标准**:
- [ ] 效果器参数可以添加到参数锁系统
- [ ] 参数锁在正确步骤生效
- [ ] 参数锁值正确应用到效果器
- [ ] 单元测试覆盖

**技术笔记**:
- 相关文件: `src/effects/track_effects.rs`, `src/step_sequencer.rs`
- 依赖: Story 5 + 现有参数锁系统
- 估算: 0.5天

**任务**:
- [ ] 定义效果器参数 ID 枚举
- [ ] 实现 `apply_param_locks()` 方法
- [ ] 在音序器步骤处理中调用参数锁应用
- [ ] 编写单元测试

---

### Story 8: 性能优化

**作为** 开发者  
**我希望** 优化效果器的性能  
**以便** 满足实时音频处理需求

**验收标准**:
- [ ] 单轨效果器 CPU < 5%
- [ ] 8轨效果器总 CPU < 30%
- [ ] 效果处理延迟 < 1ms
- [ ] 性能测试通过

**技术笔记**:
- 相关文件: `src/effects/track_effects.rs`
- 依赖: 所有其他 Stories
- 估算: 0.5天

**任务**:
- [ ] 实现效果器懒加载
- [ ] 优化旁路时的处理逻辑
- [ ] 添加性能基准测试
- [ ] 运行并优化 CPU 占用

---

## 任务清单

### 开发顺序

| Story | 依赖 | 估算 |
|-------|------|------|
| Story 1: 效果器槽位数据结构 | 无 | 0.5天 |
| Story 2: 轨道效果器链 | Story 1 | 0.5天 |
| Story 3: 效果器工厂 | Story 2 | 0.5天 |
| Story 4: 全局效果器管理器 | Story 3 | 0.5天 |
| Story 5: 与音序器集成 | Story 4 | 0.5天 |
| Story 6: 效果器参数控制 | Story 4 | 0.5天 |
| Story 7: 参数锁集成 | Story 5 | 0.5天 |
| Story 8: 性能优化 | 所有 | 0.5天 |

**总估算**: 4天

### 验收标准汇总

- [ ] 所有 Stories 完成
- [ ] 单元测试 > 20个
- [ ] 集成测试 > 5个
- [ ] 性能测试 > 3个
- [ ] 所有测试通过 (100% pass rate)

---

## 测试详情

### 单元测试清单

```rust
// track_effects.rs tests

#[test]
fn test_track_effect_slot_creation() { ... }

#[test]
fn test_track_effect_slot_defaults() { ... }

#[test]
fn test_track_effects_creation() { ... }

#[test]
fn test_add_effect() { ... }

#[test]
fn test_remove_effect() { ... }

#[test]
fn test_effect_chain_processing() { ... }

#[test]
fn test_bypass_behavior() { ... }

#[test]
fn test_mix_parameter() { ... }

#[test]
fn test_effect_factory_create() { ... }

#[test]
fn test_effect_factory_register() { ... }

#[test]
fn test_per_track_effects_manager() { ... }

#[test]
fn test_multi_track_effects() { ... }

#[test]
fn test_snapshot_save_restore() { ... }

#[test]
fn test_effect_param_lock() { ... }
```

### 集成测试清单

```rust
// integration_tests.rs

#[test]
fn test_sequencer_with_track_effects() { ... }

#[test]
fn test_effects_dont_affect_timing() { ... }

#[test]
fn test_effects_parameter_changes() { ... }

#[test]
fn test_all_8_tracks_with_effects() { ... }

#[test]
fn test_effect_bypass_no_artifacts() { ... }
```

### 性能测试清单

```rust
// performance_tests.rs

#[test]
fn test_single_track_effect_performance() { ... }

#[test]
fn test_all_8_tracks_performance() { ... }

#[test]
fn test_effect_processing_latency() { ... }
```

---

## 里程碑

| 日期 | 里程碑 | 交付物 |
|------|--------|--------|
| Day 1 | 完成 Stories 1-2 | 核心数据结构 |
| Day 2 | 完成 Stories 3-4 | 效果器系统 |
| Day 3 | 完成 Stories 5-6 | 音序器集成 |
| Day 4 | 完成 Stories 7-8 | 参数锁 + 优化 |

---

*本文档基于 WAVELET 开发工作流 (BMAD-METHOD) 创建*
