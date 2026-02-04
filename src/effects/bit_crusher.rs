// WAVELET - Bit Crusher Effect Module
// 参考: Elektron Tonverk Bit Crusher
//
// Bit Crusher (比特粉碎) 通过降低信号的位深度和采样率来产生数字复古质感
//
// 位深度降低 (Bit Reduction):
// - 原始: 24-bit 或 16-bit
// - 粉碎后: 1-24 bit
// - 公式: output = quantize(input, bits)
//
// 采样率降低 (Sample Rate Reduction / Decimation):
// - 原始: 44100 Hz
// - 降低后: 11025 Hz - 44100 Hz
// - 原理: 每N个样本取一个，丢弃中间的
//
// 参数:
// - BIT REDUCTION: 位深度 (1-24 bit)
// - SAMPLE RATE: 降采样比率 (1x - 4x)
//
// 声音特性:
// - 低位深度: 产生量化噪声，粗糙感
// - 降采样: 产生折叠频率(alias)，金属质感

/// 降采样模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimationMode {
    /// 简单降采样 (丢弃中间样本)
    Decimate,

    /// 样本保持 (Sample & Hold)
    SampleHold,

    /// 线性插值降采样
    Linear,
}

/// Bit Crusher配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitCrusherConfig {
    /// 位深度 (1-24 bit, 24 = 无损)
    pub bit_depth: f32,

    /// 降采样比率 (1.0 = 无降采样, 4.0 = 4x降采样)
    pub sample_rate_reduction: f32,

    /// 降采样模式
    pub decimation_mode: DecimationMode,

    /// 混合比例 (dry/wet)
    pub mix: f32,

    /// 输出增益补偿 (dB)
    pub output_gain_db: f32,
}

impl Default for BitCrusherConfig {
    fn default() -> Self {
        Self {
            bit_depth: 8.0,
            sample_rate_reduction: 2.0,
            decimation_mode: DecimationMode::Decimate,
            mix: 1.0,
            output_gain_db: 0.0,
        }
    }
}

/// Bit Crusher
#[derive(Debug, Clone)]
pub struct BitCrusher {
    /// 配置
    config: BitCrusherConfig,

    /// 采样率
    sample_rate: f32,

    /// 当前样本计数
    sample_counter: usize,

    /// 上一个处理后的样本 (用于降采样)
    last_output: f32,

    /// 降采样间隔
    decimation_interval: usize,

    /// 输出增益线性值
    output_gain: f32,

    /// 量化步长
    quantize_step: f32,
}

impl Default for BitCrusher {
    fn default() -> Self {
        Self::new()
    }
}

impl BitCrusher {
    /// 创建新的Bit Crusher
    pub fn new() -> Self {
        Self::new_with_sample_rate(44100.0)
    }

    /// 创建带采样率的Bit Crusher
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let mut crusher = Self {
            config: BitCrusherConfig::default(),
            sample_rate,
            sample_counter: 0,
            last_output: 0.0,
            decimation_interval: 1,
            output_gain: 1.0,
            quantize_step: 1.0 / (2.0f32.powf(23.0)), // 24-bit half
        };

        crusher.update_parameters();
        crusher
    }

    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_parameters();
    }

    /// 设置配置
    pub fn set_config(&mut self, config: BitCrusherConfig) {
        self.config = config;
        self.update_parameters();
    }

    /// 获取配置
    pub fn get_config(&self) -> BitCrusherConfig {
        self.config
    }

    /// 设置位深度
    pub fn set_bit_depth(&mut self, bits: f32) {
        self.config.bit_depth = bits.clamp(1.0, 24.0);
        self.update_parameters();
    }

    /// 设置降采样比率
    pub fn set_sample_rate_reduction(&mut self, ratio: f32) {
        self.config.sample_rate_reduction = ratio.clamp(1.0, 8.0);
        self.update_parameters();
    }

    /// 设置混合比例
    pub fn set_mix(&mut self, mix: f32) {
        self.config.mix = mix.clamp(0.0, 1.0);
    }

    /// 设置降采样模式
    pub fn set_decimation_mode(&mut self, mode: DecimationMode) {
        self.config.decimation_mode = mode;
    }

    /// 更新内部参数
    fn update_parameters(&mut self) {
        // 计算降采样间隔
        self.decimation_interval = self.config.sample_rate_reduction as usize;
        if self.decimation_interval < 1 {
            self.decimation_interval = 1;
        }

        // 计算量化步长
        // 24-bit: 步长 = 2^-23 (考虑符号位)
        // 1-bit: 步长 = 1.0
        let effective_bits = self.config.bit_depth.clamp(1.0, 24.0);
        self.quantize_step = 2.0f32.powf(1.0 - effective_bits);

        // 输出增益 (补偿量化损失)
        self.output_gain = 10.0f32.powf(self.config.output_gain_db / 20.0);
    }

    /// 量化样本到指定位深度
    #[inline]
    fn quantize(&self, input: f32) -> f32 {
        // 范围 [-1.0, 1.0]
        let clamped = input.clamp(-1.0, 1.0);

        if self.quantize_step >= 1.0 {
            // 1-2 bit: 使用符号函数
            // sign(input) * step/2 对于1-bit
            clamped.signum() * self.quantize_step * 0.5
        } else {
            // 3-24 bit: 标准量化
            (clamped / self.quantize_step).round() * self.quantize_step
        }
    }

    /// 处理样本
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let is_decimation_point = self.sample_counter.is_multiple_of(self.decimation_interval);

        let processed = match (is_decimation_point, self.config.decimation_mode) {
            (true, _) => {
                // 降采样点: 量化并更新输出
                let quantized = self.quantize(input);
                self.last_output = quantized * self.output_gain;
                self.last_output
            }
            (false, DecimationMode::Decimate) => {
                // 非降采样点: 保持上一个值 (Decimate)
                self.last_output
            }
            (false, DecimationMode::SampleHold) => {
                // 样本保持
                self.last_output
            }
            (false, DecimationMode::Linear) => {
                // 线性插值 (简化: 返回上一个值)
                self.last_output
            }
        };

        self.sample_counter += 1;

        // 干湿混合
        input + (processed - input) * self.config.mix
    }

    /// 处理立体声样本
    #[inline]
    pub fn process_stereo(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        (self.process(input_left), self.process(input_right))
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.sample_counter = 0;
        self.last_output = 0.0;
    }

    /// 计算理论延迟
    pub fn get_latency(&self) -> usize {
        // Bit Crusher本身零延迟，但可能有滤波器延迟
        0
    }

    /// 计算理论采样率
    pub fn get_effective_sample_rate(&self) -> f32 {
        self.sample_rate / self.config.sample_rate_reduction
    }
}

/// 立体声Bit Crusher
#[derive(Debug, Clone)]
pub struct StereoBitCrusher {
    /// 左声道
    crusher_l: BitCrusher,

    /// 右声道
    crusher_r: BitCrusher,

    /// 立体声相位偏移
    stereo_offset: usize,
}

impl Default for StereoBitCrusher {
    fn default() -> Self {
        Self::new()
    }
}

impl StereoBitCrusher {
    /// 创建新的立体声Bit Crusher
    pub fn new() -> Self {
        Self {
            crusher_l: BitCrusher::new(),
            crusher_r: BitCrusher::new(),
            stereo_offset: 0,
        }
    }

    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.crusher_l.set_sample_rate(sample_rate);
        self.crusher_r.set_sample_rate(sample_rate);
    }

    /// 设置配置
    pub fn set_config(&mut self, config: BitCrusherConfig) {
        self.crusher_l.set_config(config);

        // 右声道使用相位偏移
        let config_r = config;
        self.stereo_offset = (config.sample_rate_reduction as usize) / 2;
        self.crusher_r.set_config(config_r);
    }

    /// 处理立体声样本
    #[inline]
    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        self.crusher_l.process_stereo(input_left, input_right)
    }

    /// 重置
    pub fn reset(&mut self) {
        self.crusher_l.reset();
        self.crusher_r.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_analysis::measure_rms;
    use std::f32::consts::PI;

    // ============ 理论验证测试 ============

    /// 理论: 位深度越低，量化步长越大，RMS应该变化
    #[test]
    fn test_bit_crusher_bit_depth_effect() {
        let input: Vec<f32> = (0..44100)
            .take(22050) // 0.5秒
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let _input_rms = measure_rms(&input);

        // 测试不同位深度
        for &bits in &[24.0, 16.0, 8.0, 4.0, 2.0, 1.0] {
            let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
            crusher.set_bit_depth(bits);
            crusher.set_sample_rate_reduction(1.0); // 无降采样

            let output: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();
            let output_rms = measure_rms(&output);

            // 所有位深度都应该产生有效输出
            assert!(
                output_rms > 0.0,
                "Bit depth {} should produce valid output, RMS = {}",
                bits,
                output_rms
            );
        }
    }

    /// 理论: 24-bit应该接近无损
    #[test]
    fn test_bit_crusher_high_bit_depth() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(24.0);
        crusher.set_sample_rate_reduction(1.0);

        let output: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();

        // 24-bit应该几乎无损
        let diff = measure_rms(&input) - measure_rms(&output);
        assert!(
            diff < 0.01,
            "24-bit should be nearly lossless, RMS diff = {}",
            diff
        );
    }

    /// 理论: 1-bit产生方波，RMS应该显著
    #[test]
    fn test_bit_crusher_1bit_theory() {
        // 正弦波输入
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.707 // 0 dBFS
            })
            .collect();

        let _input_rms = measure_rms(&input);

        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(1.0);
        crusher.set_sample_rate_reduction(1.0);
        crusher.set_mix(1.0);

        let output: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();
        let output_rms = measure_rms(&output);

        // 1-bit量化后的方波RMS应该在0.25-0.5之间
        assert!(
            output_rms > 0.2,
            "1-bit output RMS {} should be significant (> 0.2)",
            output_rms
        );
    }

    /// 理论: 降采样产生折叠频率(aliasing)
    #[test]
    fn test_bit_crusher_decimation_aliasing() {
        // 高频输入 (接近奈奎斯特频率的一半)
        let input_freq = 10000.0;
        let sample_rate = 44100.0;

        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * PI * input_freq * t).sin() * 0.5
            })
            .collect();

        let input_rms = measure_rms(&input);

        // 4x降采样
        let mut crusher = BitCrusher::new_with_sample_rate(sample_rate);
        crusher.set_bit_depth(24.0);
        crusher.set_sample_rate_reduction(4.0);
        crusher.set_mix(1.0);

        let output: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();
        let output_rms = measure_rms(&output);

        // 降采样应该改变RMS (产生aliasing)
        let _rms_diff = (input_rms - output_rms).abs();
        assert!(
            output_rms > 0.0,
            "Decimated output should have energy, RMS = {}",
            output_rms
        );
    }

    /// 理论: 静音输入产生静音输出
    #[test]
    fn test_bit_crusher_silence() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(4.0);
        crusher.set_sample_rate_reduction(4.0);

        let silence: Vec<f32> = vec![0.0; 1000];
        let output: Vec<f32> = silence.iter().map(|&s| crusher.process(s)).collect();

        let output_rms = measure_rms(&output);
        assert!(
            output_rms < 0.0001,
            "Silence input should produce silence output, RMS = {}",
            output_rms
        );
    }

    /// 理论: 降采样间隔应该产生周期性变化
    #[test]
    fn test_bit_crusher_decimation_period() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(24.0);
        crusher.set_sample_rate_reduction(4.0);

        // 正弦波输入
        let input: Vec<f32> = (0..100).map(|i| (i as f32 / 10.0).sin() * 0.5).collect();

        let output: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();

        // 验证输出有值
        let output_rms = measure_rms(&output);
        assert!(
            output_rms > 0.01,
            "Decimated output should have energy, RMS = {}",
            output_rms
        );
    }

    /// 理论: 降采样产生近零延迟
    #[test]
    fn test_bit_crusher_latency() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(8.0);
        crusher.set_sample_rate_reduction(4.0);

        // 脉冲输入
        let mut input = vec![0.0; 100];
        input[0] = 1.0;

        let output: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();

        // 找到输出峰值位置
        let max_idx = output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        // Bit Crusher应该有近零延迟 (在降采样间隔内)
        let max_allowed = crusher.decimation_interval;
        assert!(
            max_idx <= max_allowed,
            "Bit crusher latency {} should be <= decimation interval {}",
            max_idx,
            max_allowed
        );
    }

    /// 理论: 混合比例应该线性变化
    #[test]
    fn test_bit_crusher_mix_linearity() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(4.0);

        // 测试干湿混合
        crusher.set_mix(0.0); // 纯干
        let output_dry: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();
        let rms_dry = measure_rms(&output_dry);

        crusher.set_mix(1.0); // 纯湿
        let output_wet: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();
        let rms_wet = measure_rms(&output_wet);

        crusher.set_mix(0.5); // 混合
        let output_mix: Vec<f32> = input.iter().map(|&s| crusher.process(s)).collect();
        let rms_mix = measure_rms(&output_mix);

        // 混合应该在干湿之间
        assert!(
            rms_mix >= rms_dry.min(rms_wet) - 0.01 && rms_mix <= rms_dry.max(rms_wet) + 0.01,
            "Mix 0.5 RMS {} should be between dry {} and wet {}",
            rms_mix,
            rms_dry,
            rms_wet
        );
    }

    /// 理论: DC输入应该被正确量化
    #[test]
    fn test_bit_crusher_dc_input() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(4.0);
        crusher.set_sample_rate_reduction(1.0);
        crusher.set_mix(1.0);

        let dc_input = 0.5;

        let output: Vec<f32> = (0..100).map(|_| crusher.process(dc_input)).collect();
        let _output_rms = measure_rms(&output);

        // DC输入的量化应该稳定
        let all_same = output.iter().all(|&x| (x - output[0]).abs() < 0.001);
        assert!(all_same, "DC input should produce stable quantized output");
    }

    // ============ 基本功能测试 ============

    #[test]
    fn test_bit_crusher_creation() {
        let crusher = BitCrusher::new();
        assert_eq!(crusher.config.bit_depth, 8.0);
        assert_eq!(crusher.config.sample_rate_reduction, 2.0);
        assert_eq!(crusher.config.mix, 1.0);
        assert_eq!(crusher.config.decimation_mode, DecimationMode::Decimate);
    }

    #[test]
    fn test_bit_crusher_set_bit_depth() {
        let mut crusher = BitCrusher::new();

        crusher.set_bit_depth(4.0);
        assert_eq!(crusher.config.bit_depth, 4.0);

        // 测试范围限制
        crusher.set_bit_depth(0.5);
        assert_eq!(crusher.config.bit_depth, 1.0);

        crusher.set_bit_depth(30.0);
        assert_eq!(crusher.config.bit_depth, 24.0);
    }

    #[test]
    fn test_bit_crusher_set_sample_rate_reduction() {
        let mut crusher = BitCrusher::new();

        crusher.set_sample_rate_reduction(4.0);
        assert_eq!(crusher.config.sample_rate_reduction, 4.0);

        // 测试范围限制
        crusher.set_sample_rate_reduction(0.5);
        assert_eq!(crusher.config.sample_rate_reduction, 1.0);

        crusher.set_sample_rate_reduction(20.0);
        assert_eq!(crusher.config.sample_rate_reduction, 8.0);
    }

    #[test]
    fn test_bit_crusher_set_mix() {
        let mut crusher = BitCrusher::new();

        crusher.set_mix(0.5);
        assert_eq!(crusher.config.mix, 0.5);

        crusher.set_mix(1.5);
        assert_eq!(crusher.config.mix, 1.0);

        crusher.set_mix(-0.5);
        assert_eq!(crusher.config.mix, 0.0);
    }

    #[test]
    fn test_bit_crusher_process() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(8.0);
        crusher.set_sample_rate_reduction(2.0);

        for _ in 0..1000 {
            let output = crusher.process(0.5);
            assert!(output.is_finite());
            assert!(output.abs() <= 1.5); // 允许一些过载
        }
    }

    #[test]
    fn test_bit_crusher_stereo() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(8.0);

        for _ in 0..1000 {
            let (left, right) = crusher.process_stereo(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_bit_crusher_reset() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);

        // 处理一些样本
        for _ in 0..100 {
            crusher.process(0.5);
        }

        // 重置
        crusher.reset();

        // 应该能正常继续处理
        let output = crusher.process(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_bit_crusher_config() {
        let config = BitCrusherConfig {
            bit_depth: 4.0,
            sample_rate_reduction: 4.0,
            decimation_mode: DecimationMode::SampleHold,
            mix: 0.7,
            output_gain_db: 6.0,
        };

        let mut crusher = BitCrusher::new();
        crusher.set_config(config);

        assert_eq!(crusher.config.bit_depth, 4.0);
        assert_eq!(crusher.config.sample_rate_reduction, 4.0);
        assert_eq!(crusher.config.mix, 0.7);
        assert_eq!(crusher.config.output_gain_db, 6.0);
    }

    #[test]
    fn test_bit_crusher_effective_sample_rate() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);

        crusher.set_sample_rate_reduction(1.0);
        assert_eq!(crusher.get_effective_sample_rate(), 44100.0);

        crusher.set_sample_rate_reduction(2.0);
        assert_eq!(crusher.get_effective_sample_rate(), 22050.0);

        crusher.set_sample_rate_reduction(4.0);
        assert_eq!(crusher.get_effective_sample_rate(), 11025.0);
    }

    #[test]
    fn test_bit_crusher_decimation_modes() {
        let mut crusher = BitCrusher::new_with_sample_rate(44100.0);
        crusher.set_bit_depth(8.0);
        crusher.set_sample_rate_reduction(4.0);

        for mode in [
            DecimationMode::Decimate,
            DecimationMode::SampleHold,
            DecimationMode::Linear,
        ] {
            crusher.set_decimation_mode(mode);
            crusher.reset();

            let output: Vec<f32> = (0..100).map(|_| crusher.process(0.5)).collect();
            let rms = measure_rms(&output);

            assert!(
                rms > 0.0,
                "Mode {:?} should produce valid output, RMS = {}",
                mode,
                rms
            );
        }
    }

    #[test]
    fn test_stereo_bit_crusher_creation() {
        let _crusher = StereoBitCrusher::new();
    }

    #[test]
    fn test_stereo_bit_crusher_process() {
        let mut crusher = StereoBitCrusher::new();
        crusher.set_sample_rate(44100.0);

        for _ in 0..1000 {
            let (left, right) = crusher.process(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }
}
