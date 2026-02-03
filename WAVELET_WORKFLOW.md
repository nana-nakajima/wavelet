# WAVELET 开发工作流 (基于 BMAD-METHOD)

> **版本**: v1.0  
> **日期**: 2026-02-03  
> **灵感来源**: BMAD-METHOD (Breakthrough Method of Agile AI Driven Development)

---

## 📋 概述

本文档定义了WAVELET项目的结构化开发流程，借鉴BMAD-METHOD的核心理念，结合WAVELET的技术特点 (Rust + Godot 4) 进行定制。

### 核心理念

1. **AI作为协作者** - 不是替代思考，而是引导最佳实践
2. **规模自适应** - 小功能用Quick Flow，大功能用Full Path
3. **结构化流程** - 从PRD到实现，每个阶段有明确输出
4. **持续改进** - 每个Sprint后回顾优化

### 适用范围

- WAVELET 核心功能开发
- 效果器模块 (Per-track Effects)
- 调制矩阵 (Modulation Matrix)
- UI/UX 改进
- Bug Fix 和小改进

---

## 🛠️ 工具链

### 必需工具

```bash
# Rust 工具链
cargo build          # 构建
cargo test           # 测试
cargo clippy         # 代码检查
cargo fmt            # 代码格式化

# Godot 4
godot --export       # 导出项目

# 版本控制
git add/commit/push  # 标准Git流程

# OpenClaw (我们的AI助手)
imsg                 # iMessage通信
clawdbot cron        # 定时任务
memory_search        # 记忆检索
```

### 可选工具

```bash
# BMAD CLI (可选，借鉴理念为主)
npx bmad-method install

# Claude Code / Cursor (代码编辑)
# 配合WAVELET开发工作流使用
```

---

## 🔄 工作流程

### 流程A: Quick Flow (小型功能)

**适用场景**:
- Bug fix
- 小型改进 (< 1天工作量)
- 明确需求的功能

**工作流**:
```
1. 需求分析 → 写入 memory/YYYY-MM-DD.md
2. 技术规格 → 明确实现方案
3. 实现 → 编写代码
4. 测试 → cargo test
5. 审查 → 代码审查 + 验证
6. 提交 → git commit + push
```

**时间目标**: 1天以内

**输出物**:
- `memory/YYYY-MM-DD.md` 更新
- Git commit
- 通过测试

---

### 流程B: Full Path (大型功能)

**适用场景**:
- 新模块 (如Per-track Effects)
- 复杂重构 (> 1天工作量)
- 需要架构设计的功能

**工作流**:
```
Phase 1: 产品定义
├── 1.1 Product Brief → 什么是这个功能?
├── 1.2 PRD → 详细需求文档
└── 1.3 验收标准 → 定义"完成"

Phase 2: 架构设计  
├── 2.1 Architecture → 技术架构文档
├── 2.2 接口设计 → 模块间接口
└── 2.3 数据结构 → 核心数据结构

Phase 3: 任务分解
├── 3.1 Epics → 大功能块
├── 3.2 Stories → 具体任务
└── 3.3 Sprint Planning → 排期

Phase 4: 实现
├── 4.1 按Story实现 → 逐个开发
├── 4.2 单元测试 → 每个模块测试
└── 4.3 集成测试 → 模块间测试

Phase 5: 审查
├── 5.1 代码审查 → 质量检查
├── 5.2 性能测试 → 性能基准
└── 5.3 文档更新 → 更新README/注释
```

**时间目标**: 1周+ (根据复杂度)

**输出物**:
- `docs/PRODUCT_BRIEF_功能名.md`
- `docs/ARCHITECTURE_功能名.md`
- `docs/STORIES_功能名.md`
- Git commits + tags
- 通过所有测试

---

## 📝 文档模板

### 模板1: Product Brief

```markdown
# Product Brief: [功能名称]

## 概述
一句话描述这个功能要解决什么问题。

## 用户故事
作为 [用户类型]，我希望 [目标]，以便 [价值]。

## 核心需求
1. [需求1]
2. [需求2]
3. [需求3]

## 成功标准
- [标准1]
- [标准2]

## 约束条件
- 技术约束
- 时间约束

## 风险
- [风险1] → [缓解措施]
```

### 模板2: PRD (Product Requirements Document)

```markdown
# PRD: [功能名称]

## 1. 背景
为什么需要这个功能？

## 2. 用户画像
- 主要用户是谁？
- 他们的需求是什么？

## 3. 功能规格

### 3.1 核心功能
| 功能 | 优先级 | 描述 |
|------|--------|------|
| F1 | P0 | 核心功能 |
| F2 | P1 | 重要功能 |
| F3 | P2 | 辅助功能 |

### 3.2 用户界面
- UI描述或草图

### 3.3 API接口
```rust
// Rust接口定义
pub fn new_feature() -> Result<(), Error> {
    // ...
}
```

## 4. 验收标准
- [标准1]
- [标准2]

## 5. 风险与依赖
- 依赖: [其他模块]
- 风险: [技术风险]

## 6. 时间估算
- 估算: [X] 人天
```

### 模板3: Architecture

```markdown
# Architecture: [功能名称]

## 1. 概述
功能的高层架构。

## 2. 系统图
```
[组件A] → [组件B] → [组件C]
```

## 3. 组件设计

### 3.1 [组件A]
- 职责: 
- 输入:
- 输出:
- 依赖:

### 3.2 [组件B]
...

## 4. 数据流
```
Step 1: [描述]
Step 2: [描述]
...
```

## 5. 技术决策
| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| 语言 | Rust/C++ | Rust | [理由] |
| 库 | lib1/lib2 | lib1 | [理由] |

## 6. 性能考虑
- 音频延迟要求
- 内存使用限制

## 7. 测试策略
- 单元测试
- 集成测试
```

### 模板4: Stories

```markdown
# Stories: [功能名称]

## Epic: [Epic名称]

### Story 1: [故事名称]
**作为**: 
**我希望**: 
**以便**: 

**验收标准**:
- [ ] 标准1
- [ ] 标准2

**技术笔记**:
- 相关文件: `src/xxx.rs`
- 依赖: Story 2
- 估算: 0.5天

---

### Story 2: [故事名称]
...
```

---

## 📊 WAVELET 项目结构

```
wavelet/
├── src/
│   ├── lib.rs              # 主入口
│   ├── synth.rs            # 合成器核心
│   ├── oscillator.rs       # 振荡器
│   ├── filter.rs           # 滤波器
│   ├── envelope.rs         # 包络
│   ├── lfo.rs              # 低频振荡器
│   ├── effects/            # 效果器
│   │   ├── mod.rs
│   │   ├── reverb.rs
│   │   ├── delay.rs
│   │   ├── distortion.rs
│   │   ├── chorus.rs       # 待开发
│   │   └── phaser.rs       # 待开发
│   ├── arpeggiator.rs      # ✅ 已完成
│   ├── step_sequencer.rs   # ✅ 已完成
│   ├── piano_roll.rs       # ✅ 已完成
│   ├── modulation_matrix.rs # 待开发
│   └── ...
├── godot/                  # Godot UI
├── tests/                  # 集成测试
├── docs/                   # 架构文档
│   ├── ARCHITECTURE_xxx.md
│   ├── PRD_xxx.md
│   └── STORIES_xxx.md
├── memory/                 # 日志
└── README.md
```

---

## 🚀 使用示例

### 示例1: 开发Chorus效果器 (Full Path)

**Step 1: Product Brief**
```bash
# 创建文档
touch docs/PRODUCT_BRIEF_CHORUS.md
```

```markdown
# Product Brief: Chorus效果器

## 概述
为WAVELET添加合唱效果器，产生宽广的立体声效果。

## 用户故事
作为音乐制作者，我希望有合唱效果，以便让单音音色更加丰富宽广。

## 核心需求
1. 立体声合唱效果
2. 可调速率 (Rate)
3. 可调深度 (Depth)
4. 可调混合比例 (Mix)

## 成功标准
- ✅ 产生明显的立体声合唱效果
- ✅ CPU使用率 < 5%
- ✅ 与现有效果器链兼容

## 风险
- 立体声处理可能增加CPU负载 → 优化算法
```

**Step 2: PRD** → 创建 `docs/PRD_CHORUS.md`

**Step 3: Architecture** → 创建 `docs/ARCHITECTURE_CHORUS.md`

**Step 4: Stories** → 创建 `docs/STORIES_CHORUS.md`

**Step 5: 实现** → 按Story开发

**Step 6: 测试** → cargo test

---

### 示例2: 修复Bug (Quick Flow)

**Step 1: 分析**
```
memory/2026-02-03.md:
## Bug: 滤波器截止频率不正确

问题: set_filter_cutoff() 设置的值与实际频率不匹配

分析: 
- 期望: 1000Hz
- 实际: 约707Hz
- 原因: 频率映射公式有误
```

**Step 2: 修复**
```rust
// src/filter.rs
// 修复前
pub fn set_cutoff(&mut self, value: f32) {
    self.cutoff = value * 1000.0; // ❌ 错误
}

// 修复后  
pub fn set_cutoff(&mut self, value: f32) {
    self.cutoff = value; // ✅ 正确 (0-1映射到实际频率)
}
```

**Step 3: 测试**
```bash
cargo test test_filter_cutoff
```

**Step 4: 提交**
```bash
git add src/filter.rs
git commit -m "fix: correct filter cutoff frequency mapping"
git push
```

---

## 📈 进度跟踪

### 看板风格的任务管理

| 状态 | 描述 |
|------|------|
| 📋 Backlog | 待开发的功能 |
| 🎯 Sprint | 当前Sprint中的任务 |
| 🔧 In Progress | 正在开发 |
| ✅ Review | 待审查 |
| ✅ Done | 已完成 |

### Sprint回顾模板

```markdown
## Sprint回顾: [日期]

### 完成的Stories
- [x] Story 1
- [x] Story 2

### 遇到的问题
- 问题1 → 解决方案
- 问题2 → 解决方案

### 改进行动
- [ ] 行动项1
- [ ] 行动项2

### 下个Sprint计划
- Story 3
- Story 4
```

---

## 🧪 测试要求

### 单元测试

每个模块必须包含单元测试:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chorus_basic() {
        let mut chorus = Chorus::new();
        chorus.set_rate(1.0);
        chorus.set_depth(0.5);
        // 验证输出...
        assert!(true);
    }
}
```

### 测试覆盖率要求

- **核心模块**: > 90%
- **效果器**: > 80%
- **整体**: > 70%

---

## 📚 参考资源

- **BMAD-METHOD**: https://github.com/bmad-code-org/BMAD-METHOD
- **BMGD (Game Dev Studio)**: https://github.com/bmad-code-org/bmad-module-game-dev-studio
- **Rust 测试最佳实践**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **Godot 4 文档**: https://docs.godotengine.org

---

## 🔄 持续改进

本文档是活的，会根据实践反馈持续改进。

### 改进来源
1. 每个Sprint回顾
2. 团队反馈
3. 工具链更新

### 更新日志

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-03 | v1.0 | 初始版本 |

---

*本文档基于BMAD-METHOD理念定制，适用于WAVELET项目。*
