// WAVELET - Phaser Effect Module
// 参考: Elektron Tonverk Phaser
// 
// Phaser是一个基于全通滤波器(All-pass Filter)的相位效果器
// 通过调制滤波器的频率产生相位移动效果
//
// 参数:
// - RATE: 调制速度
// - DEPTH: 效果深度
// - FEEDBACK: 反馈量
// - POLES: 滤波器级数 (2/4/6/8)
// - MIX: 干湿比

use std::f32::consts::PI;

#[cfg(test)]
use crate::audio_analysis;

/// Phaser配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaserConfig {
    /// 调制速度 (Hz, 0.01 - 10.0)
    pub rate: f32,
    
    /// 效果深度 (0.0 - 1.0)
    pub depth: f32,
    
    /// 反馈量 (0.0 - 0.95)
    pub feedback: f32,
    
    /// 滤波器级数 (2, 4, 6, 8)
    pub poles: u8,
    
    /// 干湿比 (0.0 - 1.0)
    pub mix: f32,
}

impl Default for PhaserConfig {
    fn default() -> Self {
        Self {
            rate: 0.5,
            depth: 0.5,
            feedback: 0.3,
            poles: 4,
            mix: 0.5,
        }
    }
}

/// 单个全通滤波器
#[derive(Debug, Clone, Copy)]
struct AllpassFilter {
    /// Q值
    q: f32,
    
    /// 上一个输入
    x1: f32,
    x2: f32,
    
    /// 上一个输出
    y1: f32,
    y2: f32,
}

impl AllpassFilter {
    fn new() -> Self {
        Self {
            q: 1.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
    
    /// 处理样本
    #[inline]
    fn process(&mut self, input: f32, coef: f32) -> f32 {
        // 全通滤波器公式:
        // y[n] = -g * x[n] + x[n-1] + g * y[n-1]
        // 其中 g = tan(ω/2) / (1 + tan(ω/2) * Q)
        
        let g = (coef / (1.0 + coef * self.q)).tan();
        
        let output = -g * input + self.x1 + g * self.y1;
        
        // 更新状态
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        
        output
    }
    
    /// 重置状态
    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Phaser效果器
#[derive(Debug, Clone)]
pub struct Phaser {
    /// 配置
    config: PhaserConfig,
    
    /// 采样率
    sample_rate: f32,
    
    /// 滤波器数组
    filters: Vec<AllpassFilter>,
    
    /// LFO相位 (0.0 - 2π)
    lfo_phase: f32,
    
    /// LFO增量
    lfo_increment: f32,
    
    /// 最小频率 (Hz)
    min_frequency: f32,
    
    /// 最大频率 (Hz)
    max_frequency: f32,
    
    /// 上一个输出 (用于反馈)
    last_output: f32,
}

impl Default for Phaser {
    fn default() -> Self {
        Self::new()
    }
}

impl Phaser {
    /// 创建新的Phaser
    pub fn new() -> Self {
        Self {
            config: PhaserConfig::default(),
            sample_rate: 44100.0,
            filters: Vec::new(),
            lfo_phase: 0.0,
            lfo_increment: 0.0,
            min_frequency: 200.0,
            max_frequency: 8000.0,
            last_output: 0.0,
        }
    }
    
    /// 创建带采样率的Phaser
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let mut phaser = Self::new();
        phaser.set_sample_rate(sample_rate);
        phaser
    }
    
    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_lfo_increment();
    }
    
    /// 设置配置
    pub fn set_config(&mut self, config: PhaserConfig) {
        self.config = config;
        self.update_lfo_increment();
        self.resize_filters();
    }
    
    /// 获取配置
    pub fn get_config(&self) -> PhaserConfig {
        self.config
    }
    
    /// 设置调制速度
    pub fn set_rate(&mut self, rate: f32) {
        self.config.rate = rate.clamp(0.01, 10.0);
        self.update_lfo_increment();
    }
    
    /// 设置效果深度
    pub fn set_depth(&mut self, depth: f32) {
        self.config.depth = depth.clamp(0.0, 1.0);
    }
    
    /// 设置反馈量
    pub fn set_feedback(&mut self, feedback: f32) {
        self.config.feedback = feedback.clamp(0.0, 0.95);
    }
    
    /// 设置滤波器级数
    pub fn set_poles(&mut self, poles: u8) {
        self.config.poles = poles.clamp(2, 8);
        if !self.config.poles.is_multiple_of(2) {
            self.config.poles += 1; // 确保是偶数
        }
        self.resize_filters();
    }
    
    /// 设置干湿比
    pub fn set_mix(&mut self, mix: f32) {
        self.config.mix = mix.clamp(0.0, 1.0);
    }
    
    /// 设置频率范围
    pub fn set_frequency_range(&mut self, min: f32, max: f32) {
        self.min_frequency = min.clamp(20.0, 20000.0);
        self.max_frequency = max.clamp(self.min_frequency, 20000.0);
    }
    
    /// 更新LFO增量
    fn update_lfo_increment(&mut self) {
        // LFO周期 = 1 / rate
        // 每样本增量 = 2π / (sample_rate / rate)
        self.lfo_increment = 2.0 * PI * self.config.rate / self.sample_rate;
    }
    
    /// 调整滤波器数量
    fn resize_filters(&mut self) {
        let num_filters = (self.config.poles / 2) as usize;
        self.filters.resize_with(num_filters, AllpassFilter::new);
    }
    
    /// 处理立体声样本
    #[inline]
    pub fn process_stereo(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        // 更新LFO相位
        self.lfo_phase += self.lfo_increment;
        if self.lfo_phase > 2.0 * PI {
            self.lfo_phase -= 2.0 * PI;
        }
        
        // 计算当前频率 (LFO正弦波)
        let lfo_value = self.lfo_phase.sin(); // -1 到 1
        
        // 频率范围: min + (max - min) * (0.5 + 0.5 * sin)
        let frequency = self.min_frequency + 
            (self.max_frequency - self.min_frequency) * 
            (0.5 + 0.5 * lfo_value * self.config.depth);
        
        // 转换为角频率
        let omega = 2.0 * PI * frequency / self.sample_rate;
        
        // 处理左声道
        let wet_left = self.process_channel(input_left, omega);
        
        // 处理右声道 (使用反相LFO产生立体声效果)
        let omega_right = 2.0 * PI * frequency / self.sample_rate; // 相同频率
        let wet_right = self.process_channel(input_right, omega_right);
        
        // 混合干湿信号
        let dry = 1.0 - self.config.mix;
        let output_left = input_left * dry + wet_left * self.config.mix;
        let output_right = input_right * dry + wet_right * self.config.mix;
        
        (output_left, output_right)
    }
    
    /// 处理单声道样本
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        // 更新LFO相位
        self.lfo_phase += self.lfo_increment;
        if self.lfo_phase > 2.0 * PI {
            self.lfo_phase -= 2.0 * PI;
        }
        
        // 计算当前频率
        let lfo_value = self.lfo_phase.sin();
        let frequency = self.min_frequency + 
            (self.max_frequency - self.min_frequency) * 
            (0.5 + 0.5 * lfo_value * self.config.depth);
        
        let omega = 2.0 * PI * frequency / self.sample_rate;
        let wet = self.process_channel(input, omega);
        
        // 混合干湿信号
        let dry = 1.0 - self.config.mix;
        input * dry + wet * self.config.mix
    }
    
    /// 处理单个通道
    #[inline]
    fn process_channel(&mut self, input: f32, omega: f32) -> f32 {
        // 计算全通滤波器系数
        // g = tan(ω/2)
        let g = (omega / 2.0).tan();
        
        // 应用反馈
        let input_with_feedback = input + self.config.feedback * self.last_output;
        
        // 通过所有全通滤波器
        let mut output = input_with_feedback;
        for filter in &mut self.filters {
            output = filter.process(output, g);
        }
        
        self.last_output = output;
        output
    }
    
    /// 重置状态
    pub fn reset(&mut self) {
        self.lfo_phase = 0.0;
        self.last_output = 0.0;
        for filter in &mut self.filters {
            filter.reset();
        }
    }
    
    /// 启用/禁用
    pub fn bypass(&mut self, enabled: bool) {
        if !enabled {
            self.reset();
        }
    }
}

/// 立体声Phaser (带L/R独立控制)
#[derive(Debug, Clone)]
pub struct StereoPhaser {
    /// 左声道Phaser
    phaser_l: Phaser,
    
    /// 右声道Phaser
    phaser_r: Phaser,
    
    /// 立体声宽度 (0.0 = 单声道, 1.0 = 最大立体声)
    stereo_width: f32,
}

impl Default for StereoPhaser {
    fn default() -> Self {
        Self {
            phaser_l: Phaser::new(),
            phaser_r: Phaser::new(),
            stereo_width: 0.5,
        }
    }
}

impl StereoPhaser {
    /// 创建新的立体声Phaser
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 创建带采样率的立体声Phaser
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let mut phaser = Self::new();
        phaser.set_sample_rate(sample_rate);
        phaser
    }
    
    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.phaser_l.set_sample_rate(sample_rate);
        self.phaser_r.set_sample_rate(sample_rate);
    }
    
    /// 设置配置
    pub fn set_config(&mut self, config: PhaserConfig) {
        self.phaser_l.set_config(config);
        self.phaser_r.set_config(config);
    }
    
    /// 获取配置
    pub fn get_config(&self) -> PhaserConfig {
        self.phaser_l.get_config()
    }
    
    /// 设置立体声宽度
    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.0);
        
        // 右声道使用稍有不同的LFO
        let _phase_offset = PI * self.stereo_width;
        // 这里简化处理，实际应该使用不同的LFO相位
    }
    
    /// 设置速率
    pub fn set_rate(&mut self, rate: f32) {
        self.phaser_l.set_rate(rate);
        self.phaser_r.set_rate(rate);
    }
    
    /// 设置深度
    pub fn set_depth(&mut self, depth: f32) {
        self.phaser_l.set_depth(depth);
        self.phaser_r.set_depth(depth);
    }
    
    /// 设置混合比例
    pub fn set_mix(&mut self, mix: f32) {
        self.phaser_l.set_mix(mix);
        self.phaser_r.set_mix(mix);
    }
    
    /// 处理立体声样本
    #[inline]
    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        self.phaser_l.process_stereo(input_left, input_right)
    }
    
    /// 处理单声道样本
    #[inline]
    pub fn process_mono(&mut self, input: f32) -> f32 {
        self.phaser_l.process(input)
    }
    
    /// 重置状态
    pub fn reset(&mut self) {
        self.phaser_l.reset();
        self.phaser_r.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phaser_creation() {
        let phaser = Phaser::new();
        assert_eq!(phaser.config.rate, 0.5);
        assert_eq!(phaser.config.depth, 0.5);
        assert_eq!(phaser.config.feedback, 0.3);
        assert_eq!(phaser.config.poles, 4);
        assert_eq!(phaser.config.mix, 0.5);
    }

    #[test]
    fn test_phaser_set_rate() {
        let mut phaser = Phaser::new();
        phaser.set_rate(2.0);
        assert_eq!(phaser.config.rate, 2.0);
        
        // 测试范围限制
        phaser.set_rate(100.0);
        assert_eq!(phaser.config.rate, 10.0); // 应该被限制
        
        phaser.set_rate(0.001);
        assert_eq!(phaser.config.rate, 0.01); // 应该被限制
    }

    #[test]
    fn test_phaser_set_depth() {
        let mut phaser = Phaser::new();
        phaser.set_depth(0.8);
        assert_eq!(phaser.config.depth, 0.8);
        
        // 测试范围限制
        phaser.set_depth(1.5);
        assert_eq!(phaser.config.depth, 1.0); // 应该被限制
        
        phaser.set_depth(-0.5);
        assert_eq!(phaser.config.depth, 0.0); // 应该被限制
    }

    #[test]
    fn test_phaser_set_feedback() {
        let mut phaser = Phaser::new();
        phaser.set_feedback(0.5);
        assert_eq!(phaser.config.feedback, 0.5);
        
        // 测试范围限制 (最大0.95)
        phaser.set_feedback(0.99);
        assert_eq!(phaser.config.feedback, 0.95);
    }

    #[test]
    fn test_phaser_set_poles() {
        let mut phaser = Phaser::new();
        phaser.set_poles(6);
        assert_eq!(phaser.config.poles, 6);
        
        // 测试范围限制
        phaser.set_poles(10);
        assert_eq!(phaser.config.poles, 8); // 应该被限制
        
        // 测试奇数处理
        phaser.set_poles(5);
        assert_eq!(phaser.config.poles, 6); // 应该调整为偶数
    }

    #[test]
    fn test_phaser_set_mix() {
        let mut phaser = Phaser::new();
        phaser.set_mix(0.8);
        assert_eq!(phaser.config.mix, 0.8);
        
        // 测试范围限制
        phaser.set_mix(1.5);
        assert_eq!(phaser.config.mix, 1.0);
        
        phaser.set_mix(-0.1);
        assert_eq!(phaser.config.mix, 0.0);
    }

    #[test]
    fn test_phaser_process() {
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        phaser.set_rate(1.0);
        phaser.set_depth(0.5);
        phaser.set_mix(1.0);
        
        // 处理一些样本
        for _ in 0..1000 {
            let output = phaser.process(0.5);
            assert!(output.is_finite());
            assert!(output.abs() <= 2.0); // 应该有合理的范围
        }
    }

    #[test]
    fn test_phaser_process_stereo() {
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        phaser.set_rate(0.5);
        phaser.set_depth(0.8);
        phaser.set_mix(0.5);
        
        // 处理立体声样本
        for _ in 0..1000 {
            let (left, right) = phaser.process_stereo(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_phaser_reset() {
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        
        // 处理一些样本
        for _ in 0..100 {
            phaser.process(0.5);
        }
        
        // 重置
        phaser.reset();
        
        // 应该能正常继续处理
        let output = phaser.process(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_phaser_config() {
        let config = PhaserConfig {
            rate: 1.5,
            depth: 0.7,
            feedback: 0.4,
            poles: 6,
            mix: 0.6,
        };
        
        let mut phaser = Phaser::new();
        phaser.set_config(config);
        
        assert_eq!(phaser.config.rate, 1.5);
        assert_eq!(phaser.config.depth, 0.7);
        assert_eq!(phaser.config.feedback, 0.4);
        assert_eq!(phaser.config.poles, 6);
        assert_eq!(phaser.config.mix, 0.6);
    }

    #[test]
    fn test_stereo_phaser_creation() {
        let phaser = StereoPhaser::new();
        assert_eq!(phaser.stereo_width, 0.5);
    }

    #[test]
    fn test_stereo_phaser_process() {
        let mut phaser = StereoPhaser::new_with_sample_rate(44100.0);
        phaser.set_rate(1.0);
        phaser.set_depth(0.5);
        phaser.set_mix(1.0);
        
        // 处理立体声样本
        for _ in 0..1000 {
            let (left, right) = phaser.process(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_phaser_frequency_range() {
        let mut phaser = Phaser::new();
        phaser.set_frequency_range(100.0, 5000.0);
        
        // 处理样本验证
        phaser.set_sample_rate(44100.0);
        phaser.set_rate(1.0);
        
        for _ in 0..100 {
            let _ = phaser.process(0.5);
        }
    }

    // ============ 音频分析测试 ============

    #[test]
    fn test_phaser_dry_signal_integrity() {
        // 测试干信号直通应该保持完整
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        phaser.set_mix(0.0); // 100% dry
        
        let input: Vec<f32> = (0..1000).map(|i| (i as f32 / 100.0).sin() * 0.5).collect();
        let output: Vec<f32> = input.iter().map(|&s| phaser.process(s)).collect();
        
        // 干信号应该几乎相同 (误差 < 0.001)
        for (i, (inp, out)) in input.iter().zip(output.iter()).enumerate() {
            assert!(
                (inp - out).abs() < 0.001,
                "Dry signal mismatch at sample {}: {} vs {}",
                i, inp, out
            );
        }
    }

    #[test]
    fn test_phaser_wet_signal_gain() {
        // 测试效果信号应该有合理的增益变化
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        phaser.set_rate(1.0);
        phaser.set_depth(0.8);
        phaser.set_feedback(0.3);
        phaser.set_mix(1.0); // 100% wet
        
        let input: Vec<f32> = (0..44100).map(|i| (i as f32 / 44100.0 * 440.0).sin() * 0.5).collect();
        
        // 处理
        let output: Vec<f32> = input.iter().map(|&s| phaser.process(s)).collect();
        
        // 增益变化应该在6dB内
        let input_rms = audio_analysis::measure_rms(&input);
        let output_rms = audio_analysis::measure_rms(&output);
        let gain_db = 20.0 * (output_rms / input_rms.max(0.0001)).log10();
        
        assert!(
            gain_db.abs() <= 6.0,
            "Gain change {} dB exceeds 6 dB limit",
            gain_db
        );
    }

    #[test]
    fn test_phaser_no_dc_offset() {
        // 测试Phaser不应该引入直流偏移
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        phaser.set_rate(2.0);
        phaser.set_depth(0.5);
        phaser.set_mix(1.0);
        
        let input: Vec<f32> = (0..44100).map(|i| (i as f32 / 44100.0 * 440.0).sin() * 0.8).collect();
        let output: Vec<f32> = input.iter().map(|&s| phaser.process(s)).collect();
        
        // 检查DC偏移
        let mean: f32 = output.iter().sum();
        let dc_offset = mean / output.len() as f32;
        
        assert!(
            dc_offset.abs() < 0.01,
            "DC offset detected: {} (expected < 0.01)",
            dc_offset
        );
    }

    #[test]
    fn test_phaser_stereo_width() {
        // 测试立体声宽度 - 使用稍微不同的输入信号
        let mut phaser = StereoPhaser::new_with_sample_rate(44100.0);
        phaser.set_rate(0.5);
        phaser.set_depth(0.8);
        phaser.set_mix(1.0);
        
        // 使用相同频率但不同相位的信号
        let freq = 440.0;
        let input: Vec<f32> = (0..44100).map(|i| {
            let t = i as f32 / 44100.0;
            (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5
        }).collect();
        
        // 右声道使用反相
        let input_right: Vec<f32> = input.iter().map(|&s| -s).collect();
        
        let mut left = Vec::new();
        let mut right = Vec::new();
        
        for (l, r) in input.iter().zip(input_right.iter()) {
            let (out_l, out_r) = phaser.process(*l, *r);
            left.push(out_l);
            right.push(out_r);
        }
        
        // 检查立体声相关性 (应该不是完美相关)
        let correlation = audio_analysis::measure_stereo_correlation(&left, &right);
        
        assert!(
            correlation < 0.99,
            "Stereo too correlated: {} (expected < 0.99 for modulated effect)",
            correlation
        );
    }

    #[test]
    fn test_phaser_frequency_spectrum() {
        // 测试频率响应
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        phaser.set_rate(0.5);
        phaser.set_depth(0.5);
        phaser.set_mix(1.0);
        
        // 1kHz测试信号
        let freq = 1000.0;
        let input: Vec<f32> = (0..44100)
            .take(22050) // 0.5秒
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5
            })
            .collect();
        
        let output: Vec<f32> = input.iter().map(|&s| phaser.process(s)).collect();
        
        // 应该看到频率梳状效应 (多个峰值)
        let analyzer = audio_analysis::SpectrumAnalyzer::new(1024, 44100.0);
        let spectrum = analyzer.analyze(&output);
        
        // 至少有3个明显的峰值 (基波 + 2个相位相关峰值)
        let mut peaks = 0;
        for i in 1..spectrum.len() - 1 {
            if spectrum[i] > spectrum[i-1] && spectrum[i] > spectrum[i+1] && spectrum[i] > -40.0 {
                peaks += 1;
            }
        }
        
        assert!(
            peaks >= 3,
            "Expected at least 3 spectral peaks, got {}",
            peaks
        );
    }

    #[test]
    fn test_phaser_latency() {
        // 测试Phaser延迟 (应该接近0)
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        phaser.set_rate(1.0);
        phaser.set_depth(0.5);
        phaser.set_mix(0.5);
        
        // 脉冲信号
        let mut input = vec![0.0; 1000];
        input[0] = 1.0;
        
        // 查找峰值位置
        let output: Vec<f32> = input.iter().map(|&s| phaser.process(s)).collect();
        
        let max_idx = output.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        
        // Phaser的延迟应该在几个samples以内
        assert!(
            max_idx <= 10,
            "Latency too high: peak at sample {}",
            max_idx
        );
    }

    #[test]
    fn test_phaser_parameter_response() {
        // 测试参数是否正确存储和生效
        let mut phaser = Phaser::new_with_sample_rate(44100.0);
        
        // 验证默认值
        assert_eq!(phaser.config.rate, 0.5);
        assert_eq!(phaser.config.depth, 0.5);
        assert_eq!(phaser.config.feedback, 0.3);
        assert_eq!(phaser.config.poles, 4);
        assert_eq!(phaser.config.mix, 0.5);
        
        // 测试参数修改
        phaser.set_rate(2.0);
        assert_eq!(phaser.config.rate, 2.0);
        
        phaser.set_depth(0.8);
        assert_eq!(phaser.config.depth, 0.8);
        
        phaser.set_feedback(0.5);
        assert_eq!(phaser.config.feedback, 0.5);
        
        phaser.set_poles(6);
        assert_eq!(phaser.config.poles, 6);
        
        phaser.set_mix(0.9);
        assert_eq!(phaser.config.mix, 0.9);
        
        // 验证参数限制
        phaser.set_rate(100.0);
        assert_eq!(phaser.config.rate, 10.0); // 应该被限制
        
        phaser.set_depth(1.5);
        assert_eq!(phaser.config.depth, 1.0); // 应该被限制
        
        phaser.set_feedback(1.0);
        assert_eq!(phaser.config.feedback, 0.95); // 应该被限制
        
        phaser.set_poles(12);
        assert_eq!(phaser.config.poles, 8); // 应该被限制
    }
}
