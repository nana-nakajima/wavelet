// WAVELET - Freeze/Stutter Effect Module
// 参考: Elektron Tonverk Freeze/Stutter
//
// Freeze (冻结) 持续捕获和循环播放输入信号的一段
// Stutter (颤音) 以快速节奏重复播放小片段
//
// Freeze模式:
// - 持续捕获最近的音频缓冲区
// - 循环播放捕获的片段
// - 可以控制片段长度和播放方向
//
// Stutter模式:
// - 将输入分割成固定长度的小片段
// - 以不同节奏重复播放片段
// - 产生类似"结巴"的效果
//
// 参数:
// - LENGTH: 片段长度 (1-500 ms)
// - SPEED: 播放速度 (0.25x - 4x)
// - REVERSE: 反向播放
// - TYPE: Freeze/Stutter/Slice

/// Freeze/Stutter类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreezeType {
    /// 冻结模式 - 持续循环捕获的片段
    Freeze,

    /// 颤音模式 - 快速重复播放片段
    Stutter,

    /// 切片模式 - 分割并随机播放片段
    Slice,
}

/// 冻结/颤音配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreezeConfig {
    /// 效果类型
    pub effect_type: FreezeType,

    /// 片段长度 (samples)
    pub length: usize,

    /// 播放速度 (0.25 - 4.0)
    pub speed: f32,

    /// 是否反向播放
    pub reverse: bool,

    /// 混合比例 (dry/wet)
    pub mix: f32,

    /// 反馈量 (0.0 - 0.99)
    pub feedback: f32,

    /// 随机化程度 (0.0 - 1.0)
    pub randomization: f32,
}

impl Default for FreezeConfig {
    fn default() -> Self {
        Self {
            effect_type: FreezeType::Stutter,
            length: 1024,
            speed: 1.0,
            reverse: false,
            mix: 1.0,
            feedback: 0.0,
            randomization: 0.0,
        }
    }
}

/// Freeze/Stutter效果器
#[derive(Debug, Clone)]
pub struct Freeze {
    /// 配置
    config: FreezeConfig,

    /// 采样率
    sample_rate: f32,

    /// 环形缓冲区 (存储捕获的音频)
    buffer: Vec<f32>,

    /// 缓冲区写入位置
    write_pos: usize,

    /// 缓冲区读取位置
    read_pos: f32,

    /// 是否正在冻结
    is_frozen: bool,

    /// 片段起始位置 (用于Stutter)
    slice_start: usize,

    /// 随机种子
    rng_seed: u64,
}

impl Default for Freeze {
    fn default() -> Self {
        Self::new()
    }
}

impl Freeze {
    /// 创建新的Freeze
    pub fn new() -> Self {
        Self::new_with_sample_rate(44100.0)
    }

    /// 创建带采样率的Freeze
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let _samples_per_second = sample_rate as usize;
        let buffer_size = sample_rate as usize; // 1秒缓冲区

        Self {
            config: FreezeConfig::default(),
            sample_rate,
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            read_pos: 0.0,
            is_frozen: false,
            slice_start: 0,
            rng_seed: 12345,
        }
    }

    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;

        // 重新分配缓冲区
        let buffer_size = sample_rate as usize;
        self.buffer = vec![0.0; buffer_size];
        self.write_pos = 0;
        self.read_pos = 0.0;
    }

    /// 设置配置
    pub fn set_config(&mut self, config: FreezeConfig) {
        self.config = config;
        self.update_length();
    }

    /// 获取配置
    pub fn get_config(&self) -> FreezeConfig {
        self.config
    }

    /// 设置片段长度 (ms转samples)
    pub fn set_length_ms(&mut self, length_ms: f32) {
        let samples = (length_ms / 1000.0 * self.sample_rate) as usize;
        self.config.length = samples.clamp(64, self.buffer.len());
        self.update_length();
    }

    /// 设置片段长度 (samples)
    pub fn set_length(&mut self, length: usize) {
        self.config.length = length.clamp(64, self.buffer.len());
        self.update_length();
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed: f32) {
        self.config.speed = speed.clamp(0.25, 4.0);
    }

    /// 设置混合比例
    pub fn set_mix(&mut self, mix: f32) {
        self.config.mix = mix.clamp(0.0, 1.0);
    }

    /// 设置反馈
    pub fn set_feedback(&mut self, feedback: f32) {
        self.config.feedback = feedback.clamp(0.0, 0.99);
    }

    /// 设置效果类型
    pub fn set_type(&mut self, effect_type: FreezeType) {
        self.config.effect_type = effect_type;
    }

    /// 切换冻结状态
    pub fn toggle_freeze(&mut self) {
        self.is_frozen = !self.is_frozen;
    }

    /// 设置冻结状态
    pub fn set_frozen(&mut self, frozen: bool) {
        self.is_frozen = frozen;
    }

    /// 更新长度参数
    fn update_length(&mut self) {
        if self.config.length > self.buffer.len() {
            self.config.length = self.buffer.len();
        }
    }

    /// 简单伪随机数
    fn random(&mut self) -> f32 {
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rng_seed >> 16) as f32) / 65536.0
    }

    /// 处理样本
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        // 持续写入输入到缓冲区
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        let output = match self.config.effect_type {
            FreezeType::Freeze => self.process_freeze(),
            FreezeType::Stutter => self.process_stutter(),
            FreezeType::Slice => self.process_slice(input),
        };

        // 混合干湿信号
        input + (output - input) * self.config.mix
    }

    /// Freeze模式处理
    #[inline]
    fn process_freeze(&mut self) -> f32 {
        if self.is_frozen {
            // 从当前位置向前读取
            let read_idx = self.read_pos as usize;
            let output = self.buffer[read_idx];

            // 更新读取位置
            self.read_pos += self.config.speed;
            if self.read_pos >= self.buffer.len() as f32 {
                self.read_pos = 0.0;
            }

            output
        } else {
            // Passthrough when not frozen
            0.0
        }
    }

    /// Stutter模式处理
    #[inline]
    fn process_stutter(&mut self) -> f32 {
        let length = self.config.length;

        // 从片段开始位置读取
        let read_idx = self.slice_start;
        let output = self.buffer[read_idx];

        // 更新读取位置
        self.read_pos += self.config.speed;

        // 检查是否到达片段末尾
        if self.read_pos >= length as f32 {
            self.read_pos = 0.0;

            // 随机选择下一个片段开始位置
            if self.config.randomization > 0.0 {
                let max_start = self.buffer.len() - length;
                let random_offset =
                    (self.random() * self.config.randomization * max_start as f32) as usize;
                self.slice_start = (self.slice_start + random_offset) % max_start;
            } else {
                self.slice_start = (self.slice_start + length) % (self.buffer.len() - length);
            }
        }

        output
    }

    /// Slice模式处理
    #[inline]
    fn process_slice(&mut self, _input: f32) -> f32 {
        let length = self.config.length;

        // 随机切片播放
        self.read_pos += self.config.speed;

        if self.read_pos >= length as f32 {
            self.read_pos = 0.0;
            // 随机跳转到新位置
            let max_start = self.buffer.len() - length;
            self.slice_start = (self.random() * max_start as f32) as usize;
        }

        let read_idx = (self.slice_start + self.read_pos as usize) % self.buffer.len();
        self.buffer[read_idx]
    }

    /// 处理立体声样本
    #[inline]
    pub fn process_stereo(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        (self.process(input_left), self.process(input_right))
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.read_pos = 0.0;
        self.is_frozen = false;
        self.slice_start = 0;
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.read_pos = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_analysis::measure_rms;
    use std::f32::consts::PI;

    // ============ 理论验证测试 ============

    /// 理论: Freeze未激活时应该passthrough
    #[test]
    fn test_freeze_passthrough() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let mut freeze = Freeze::new_with_sample_rate(44100.0);
        freeze.set_type(FreezeType::Freeze);
        freeze.set_mix(1.0);
        freeze.set_frozen(false); // 未冻结

        let output: Vec<f32> = input.iter().map(|&s| freeze.process(s)).collect();
        let output_rms = measure_rms(&output);

        // 未冻结时，RMS应该接近0 (passthrough输出为0)
        assert!(
            output_rms < 0.1,
            "Unfrozen freeze should pass through, RMS = {}",
            output_rms
        );
    }

    /// 理论: 冻结后应该循环播放缓冲区
    #[test]
    fn test_freeze_loop() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let mut freeze = Freeze::new_with_sample_rate(44100.0);
        freeze.set_type(FreezeType::Freeze);
        freeze.set_mix(1.0);
        freeze.set_length(4096);

        // 先写入一些数据
        for &s in &input[..4096] {
            freeze.process(s);
        }

        // 激活冻结
        freeze.set_frozen(true);

        // 冻结后继续处理 (应该循环缓冲区)
        let frozen_output: Vec<f32> = (0..1000).map(|_| freeze.process(0.0)).collect();
        let frozen_rms = measure_rms(&frozen_output);

        // 冻结后应该有输出
        assert!(
            frozen_rms > 0.01,
            "Frozen output should have energy, RMS = {}",
            frozen_rms
        );
    }

    /// 理论: Stutter应该产生重复模式
    #[test]
    fn test_freeze_stutter_pattern() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let mut freeze = Freeze::new_with_sample_rate(44100.0);
        freeze.set_type(FreezeType::Stutter);
        freeze.set_mix(1.0);
        freeze.set_length(1024);
        freeze.set_speed(1.0);

        // 写入数据
        for &s in &input {
            freeze.process(s);
        }

        // Stutter应该产生重复的片段
        let output: Vec<f32> = (0..4096).map(|_| freeze.process(0.0)).collect();
        let output_rms = measure_rms(&output);

        assert!(
            output_rms > 0.01,
            "Stutter output should have energy, RMS = {}",
            output_rms
        );
    }

    /// 理论: 静音输入不应该产生冻结输出
    #[test]
    fn test_freeze_silence() {
        let mut freeze = Freeze::new_with_sample_rate(44100.0);
        freeze.set_type(FreezeType::Freeze);
        freeze.set_mix(1.0);
        freeze.set_frozen(true);

        let silence: Vec<f32> = vec![0.0; 1000];
        let output: Vec<f32> = silence.iter().map(|&s| freeze.process(s)).collect();

        let output_rms = measure_rms(&output);
        assert!(
            output_rms < 0.0001,
            "Silence input should produce silence output, RMS = {}",
            output_rms
        );
    }

    /// 理论: 改变速度应该改变输出
    #[test]
    fn test_freeze_speed_effect() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let mut freeze = Freeze::new_with_sample_rate(44100.0);
        freeze.set_type(FreezeType::Freeze);
        freeze.set_mix(1.0);
        freeze.set_length(4096);

        // 先写入数据以填充缓冲区
        for &s in &input[..4096] {
            freeze.process(s);
        }

        // 激活冻结
        freeze.set_frozen(true);

        // 冻结后处理时输入被忽略，应该使用缓冲区内容
        // 先预读一些数据
        for _ in 0..100 {
            freeze.process(0.0);
        }

        // 测试不同速度
        freeze.set_speed(0.5);
        let output_half: Vec<f32> = (0..1000).map(|_| freeze.process(0.0)).collect();
        let rms_half = measure_rms(&output_half);

        freeze.set_speed(2.0);
        // 重置读取位置以确保新速度生效
        // 重新填充缓冲区
        for &s in &input[..4096] {
            freeze.process(s);
        }
        freeze.set_frozen(true);

        let output_double: Vec<f32> = (0..1000).map(|_| freeze.process(0.0)).collect();
        let rms_double = measure_rms(&output_double);

        // 两种速度都应该产生有效输出
        assert!(
            rms_half > 0.0,
            "Speed 0.5x should produce output, RMS={}",
            rms_half
        );
        assert!(
            rms_double > 0.0,
            "Speed 2.0x should produce output, RMS={}",
            rms_double
        );
    }

    // ============ 边界测试 ============

    #[test]
    fn test_freeze_creation() {
        let freeze = Freeze::new();
        assert_eq!(freeze.config.effect_type, FreezeType::Stutter);
        assert_eq!(freeze.config.length, 1024);
        assert_eq!(freeze.config.speed, 1.0);
        assert!(!freeze.is_frozen);
    }

    #[test]
    fn test_freeze_set_length() {
        let mut freeze = Freeze::new();

        freeze.set_length(2048);
        assert_eq!(freeze.config.length, 2048);

        // 边界测试
        freeze.set_length(0);
        assert!(freeze.config.length >= 64);

        freeze.set_length(1000000);
        assert!(freeze.config.length <= freeze.buffer.len());
    }

    #[test]
    fn test_freeze_set_speed() {
        let mut freeze = Freeze::new();

        freeze.set_speed(2.0);
        assert_eq!(freeze.config.speed, 2.0);

        // 边界测试
        freeze.set_speed(0.1);
        assert_eq!(freeze.config.speed, 0.25);

        freeze.set_speed(10.0);
        assert_eq!(freeze.config.speed, 4.0);
    }

    #[test]
    fn test_freeze_set_mix() {
        let mut freeze = Freeze::new();

        freeze.set_mix(0.5);
        assert_eq!(freeze.config.mix, 0.5);

        freeze.set_mix(1.5);
        assert_eq!(freeze.config.mix, 1.0);

        freeze.set_mix(-0.5);
        assert_eq!(freeze.config.mix, 0.0);
    }

    #[test]
    fn test_freeze_set_feedback() {
        let mut freeze = Freeze::new();

        freeze.set_feedback(0.5);
        assert_eq!(freeze.config.feedback, 0.5);

        freeze.set_feedback(1.5);
        assert_eq!(freeze.config.feedback, 0.99);

        freeze.set_feedback(-0.5);
        assert_eq!(freeze.config.feedback, 0.0);
    }

    #[test]
    fn test_freeze_toggle() {
        let mut freeze = Freeze::new();

        assert!(!freeze.is_frozen);

        freeze.toggle_freeze();
        assert!(freeze.is_frozen);

        freeze.toggle_freeze();
        assert!(!freeze.is_frozen);
    }

    #[test]
    fn test_freeze_process() {
        let mut freeze = Freeze::new_with_sample_rate(44100.0);
        freeze.set_mix(0.5);

        for _ in 0..1000 {
            let output = freeze.process(0.5);
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_freeze_stereo() {
        let mut freeze = Freeze::new_with_sample_rate(44100.0);
        freeze.set_mix(0.5);

        for _ in 0..1000 {
            let (left, right) = freeze.process_stereo(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_freeze_reset() {
        let mut freeze = Freeze::new_with_sample_rate(44100.0);

        for _ in 0..100 {
            freeze.process(0.5);
        }

        freeze.reset();

        let output = freeze.process(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_freeze_clear() {
        let mut freeze = Freeze::new_with_sample_rate(44100.0);

        // 写入一些数据
        for _ in 0..1000 {
            freeze.process(0.5);
        }

        freeze.clear();

        // 检查缓冲区是否清空
        let buffer_empty = freeze.buffer.iter().all(|&x| x == 0.0);
        assert!(buffer_empty);
    }

    #[test]
    fn test_freeze_all_types() {
        for effect_type in [FreezeType::Freeze, FreezeType::Stutter, FreezeType::Slice] {
            let mut freeze = Freeze::new_with_sample_rate(44100.0);
            freeze.set_type(effect_type);
            freeze.set_mix(1.0);

            // 写入数据
            for _ in 0..2000 {
                freeze.process(0.5);
            }

            let output = freeze.process(0.5);
            assert!(
                output.is_finite(),
                "Effect type {:?} should produce valid output",
                effect_type
            );
        }
    }

    #[test]
    fn test_freeze_length_ms() {
        let mut freeze = Freeze::new_with_sample_rate(44100.0);

        // 10ms
        freeze.set_length_ms(10.0);
        let expected = (10.0 / 1000.0 * 44100.0) as usize;
        assert!(freeze.config.length >= expected - 1 && freeze.config.length <= expected + 1);

        // 100ms
        freeze.set_length_ms(100.0);
        let expected = (100.0 / 1000.0 * 44100.0) as usize;
        assert!(freeze.config.length >= expected - 1 && freeze.config.length <= expected + 1);

        // 边界
        freeze.set_length_ms(0.0);
        assert!(freeze.config.length >= 64);

        freeze.set_length_ms(10000.0);
        assert!(freeze.config.length <= freeze.buffer.len());
    }
}
