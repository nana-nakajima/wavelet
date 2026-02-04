# WAVELET CI 最佳实践

> 确保GitHub Actions持续正确运转

## 📋 推送前检查清单

每次推送前运行：
```bash
./ci-monitor.sh
```

或者手动检查：
```bash
# 1. 运行测试
cargo test --lib --no-default-features

# 2. 检查Clippy
cargo clippy --lib --no-default-features -- -D warnings

# 3. 检查格式
cargo fmt --check
```

## 🔧 添加新测试模块

当添加新模块时，需要更新：
1. `.github/workflows/ci.yml` - 添加新的测试job
2. `ci-monitor.sh` - 添加新模块测试
3. 确保模块有`#[cfg(test)]`测试

## 📊 当前测试状态

| 模块 | 测试数 | 状态 |
|------|--------|------|
| oscillator | 8 | ✅ |
| envelope | 6 | ✅ |
| filter | 8 | ✅ |
| lfo | 5 | ✅ |
| synth | 7 | ✅ |
| mod_matrix | 21 | ✅ |
| arpeggiator | 6 | ✅ |
| step_sequencer | 11 | ✅ |
| piano_roll | 10 | ✅ |
| melody_generator | 83 | ✅ |
| chord_generator | 47 | ✅ |
| rhythm_generator | 14 | ✅ |
| effects/chorus | 17 | ✅ |
| effects/phaser | 13 | ✅ |
| effects/flanger | 16 | ✅ |
| effects/tremolo | 17 | ✅ |
| effects/warp | 17 | ✅ |
| effects/ring_modulator | 12 | ✅ |
| effects/bit_crusher | 13 | ✅ |
| effects/filter_bank | 15 | ✅ |
| effects/freeze | 9 | ✅ |
| effects/simple_eq | 8 | ✅ |
| audio_analysis | 3 | ✅ |
| midi_cc | 8 | ✅ |
| project_save | 28 | ✅ |
| time_stretch | 12 | ✅ |
| presets | 5 | ✅ |
| **总计** | **402** | ✅ |

## 🚨 故障排除

### 测试失败
```bash
cargo test --lib --no-default-features -- module_name
```

### Clippy警告
```bash
cargo clippy --lib --no-default-features
```

### 格式问题
```bash
cargo fmt
```

## 🔄 Git Hooks

项目已配置`pre-push` hook，推送前自动运行检查。

## 📈 CI监控

每30分钟自动运行CI检查（通过cron job）
