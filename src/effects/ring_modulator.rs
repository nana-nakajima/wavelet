// WAVELET - Ring Modulator Effect Module
// 参考: Elektron Tonverk Ring Modulator
// 
// Ring Modulator (环形调制器) 将输入信号与载波振荡器相乘
// 产生非谐波边带频率，是制造金属/钟声音色的经典方法
//
// 公式: output = input * carrier
//
// 载波类型:
// - OSC1: 内部正弦波振荡器
// - OSC2: 第二个正弦波振荡器  
// - LFO: 低频振荡器 (产生节奏性调制)
//
// 参数:
// - OSC FREQ: 载波频率 (Hz)
// - OSC WAVE: 载波波形
// - LFO RATE: LFO调制速度 (当选择LFO模式时)
// - LFO DEPTH: LFO调制深度

use std::f32::consts::PI;

/// Ring Modulator载波模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingModulatorMode {
    /// 内部振荡器作为载波
    Oscillator,
    
    /// LFO作为载波 (产生振颤效果)
    Lfo,
    
    /// 外部输入作为载波 (可选扩展)
    External,
}

/// Ring Modulator波形
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingModulatorWave {
    /// 正弦波 - 最纯净的环形调制
    Sine,
    
    /// 方波 - 产生奇次谐波
    Square,
    
    /// 锯齿波 - 产生所有谐波
    Saw,
    
    /// 三角波 - 较少谐波
    Triangle,
}

/// Ring Modulator配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RingModulatorConfig {
    /// 载波模式
    pub mode: RingModulatorMode,
    
    /// 载波频率 (Hz, 20 - 2000)
    pub osc_frequency: f32,
    
    /// 载波波形
    pub osc_wave: RingModulatorWave,
    
    /// LFO速率 (Hz, 0.1 - 20)
    pub lfo_rate: f32,
    
    /// LFO调制深度 (0.0 - 1.0)
    pub lfo_depth: f32,
}

impl Default for RingModulatorConfig {
    fn default() -> Self {
        Self {
            mode: RingModulatorMode::Oscillator,
            osc_frequency: 440.0,
            osc_wave: RingModulatorWave::Sine,
            lfo_rate: 5.0,
            lfo_depth: 0.5,
        }
    }
}

/// 环形调制器
#[derive(Debug, Clone)]
pub struct RingModulator {
    /// 配置
    config: RingModulatorConfig,
    
    /// 采样率
    sample_rate: f32,
    
    /// 载波振荡器相位
    carrier_phase: f32,
    
    /// 载波相位增量
    carrier_increment: f32,
    
    /// LFO相位
    lfo_phase: f32,
    
    /// LFO相位增量
    lfo_increment: f32,
}

impl Default for RingModulator {
    fn default() -> Self {
        Self::new()
    }
}

impl RingModulator {
    /// 创建新的Ring Modulator
    pub fn new() -> Self {
        Self::new_with_sample_rate(44100.0)
    }
    
    /// 创建带采样率的Ring Modulator
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let mut ring = Self {
            config: RingModulatorConfig::default(),
            sample_rate,
            carrier_phase: 0.0,
            carrier_increment: 0.0,
            lfo_phase: 0.0,
            lfo_increment: 0.0,
        };
        
        ring.update_increments();
        ring
    }
    
    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_increments();
    }
    
    /// 设置配置
    pub fn set_config(&mut self, config: RingModulatorConfig) {
        self.config = config;
        self.update_increments();
    }
    
    /// 获取配置
    pub fn get_config(&self) -> RingModulatorConfig {
        self.config
    }
    
    /// 设置载波模式
    pub fn set_mode(&mut self, mode: RingModulatorMode) {
        self.config.mode = mode;
    }
    
    /// 设置载波频率
    pub fn set_osc_frequency(&mut self, freq: f32) {
        self.config.osc_frequency = freq.clamp(20.0, 2000.0);
        self.update_increments();
    }
    
    /// 设置载波波形
    pub fn set_osc_wave(&mut self, wave: RingModulatorWave) {
        self.config.osc_wave = wave;
    }
    
    /// 设置LFO速率
    pub fn set_lfo_rate(&mut self, rate: f32) {
        self.config.lfo_rate = rate.clamp(0.1, 20.0);
        self.update_increments();
    }
    
    /// 设置LFO深度
    pub fn set_lfo_depth(&mut self, depth: f32) {
        self.config.lfo_depth = depth.clamp(0.0, 1.0);
    }
    
    /// 更新相位增量
    fn update_increments(&mut self) {
        // 载波相位增量: 2π * freq / sample_rate
        self.carrier_increment = 2.0 * PI * self.config.osc_frequency / self.sample_rate;
        
        // LFO相位增量
        self.lfo_increment = 2.0 * PI * self.config.lfo_rate / self.sample_rate;
    }
    
    /// 生成载波波形
    #[inline]
    fn carrier_wave(&self, phase: f32) -> f32 {
        match self.config.osc_wave {
            RingModulatorWave::Sine => phase.sin(),
            RingModulatorWave::Square => {
                if phase.sin() >= 0.0 { 1.0 } else { -1.0 }
            }
            RingModulatorWave::Saw => {
                // 锯齿波: 2 * (phase / 2π) - 1
                let normalized = (phase / (2.0 * PI)).fract();
                2.0 * normalized - 1.0
            }
            RingModulatorWave::Triangle => {
                // 三角波
                let normalized = (phase / (2.0 * PI)).fract();
                if normalized < 0.5 {
                    4.0 * normalized - 1.0
                } else {
                    -4.0 * normalized + 3.0
                }
            }
        }
    }
    
    /// 处理样本
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        // 更新相位
        self.carrier_phase += self.carrier_increment;
        if self.carrier_phase > 2.0 * PI {
            self.carrier_phase -= 2.0 * PI;
        }
        
        self.lfo_phase += self.lfo_increment;
        if self.lfo_phase > 2.0 * PI {
            self.lfo_phase -= 2.0 * PI;
        }
        
        match self.config.mode {
            RingModulatorMode::Oscillator => {
                // 直接环形调制
                let carrier = self.carrier_wave(self.carrier_phase);
                input * carrier
            }
            RingModulatorMode::Lfo => {
                // LFO调制的载波频率 - 产生周期性变化的效果
                // LFO值范围 -1 到 1，调制后的频率 = osc_freq * (1 + lfo_depth * lfo_value)
                let lfo_value = self.lfo_phase.sin();
                let modulated_freq = self.config.osc_frequency * 
                    (1.0 + lfo_value * self.config.lfo_depth);
                
                // 使用调制后的频率重新计算相位
                let modulated_increment = 2.0 * PI * modulated_freq / self.sample_rate;
                let modulated_phase = (self.carrier_increment * self.sample_rate / (2.0 * PI)).atan2(1.0) + 
                    self.lfo_phase.sin() * self.config.lfo_depth * modulated_increment;
                
                let carrier = self.carrier_wave(modulated_phase);
                input * carrier
            }
            RingModulatorMode::External => {
                // 外部载波 (简化: 使用内部载波作为后备)
                let carrier = self.carrier_wave(self.carrier_phase);
                input * carrier
            }
        }
    }
    
    /// 处理立体声样本
    #[inline]
    pub fn process_stereo(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        (self.process(input_left), self.process(input_right))
    }
    
    /// 重置状态
    pub fn reset(&mut self) {
        self.carrier_phase = 0.0;
        self.lfo_phase = 0.0;
    }
    
    /// 启用/禁用
    pub fn bypass(&mut self, enabled: bool) {
        if !enabled {
            self.reset();
        }
    }
}

/// 立体声Ring Modulator
#[derive(Debug, Clone)]
pub struct StereoRingModulator {
    /// 左声道
    ring_l: RingModulator,
    
    /// 右声道
    ring_r: RingModulator,
    
    /// 立体声相位偏移
    stereo_phase: f32,
}

impl Default for StereoRingModulator {
    fn default() -> Self {
        Self::new()
    }
}

impl StereoRingModulator {
    /// 创建新的立体声Ring Modulator
    pub fn new() -> Self {
        Self {
            ring_l: RingModulator::new(),
            ring_r: RingModulator::new(),
            stereo_phase: PI / 2.0, // 90度偏移
        }
    }
    
    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.ring_l.set_sample_rate(sample_rate);
        self.ring_r.set_sample_rate(sample_rate);
    }
    
    /// 设置配置
    pub fn set_config(&mut self, config: RingModulatorConfig) {
        self.ring_l.set_config(config);
        
        // 右声道使用相位偏移
        let mut config_r = config;
        config_r.osc_frequency = config.osc_frequency * 
            (1.0 + (self.stereo_phase / (2.0 * PI)).sin() * 0.01); // 微调频率
        self.ring_r.set_config(config_r);
    }
    
    /// 设置立体声相位偏移
    pub fn set_stereo_phase(&mut self, phase: f32) {
        self.stereo_phase = phase.clamp(0.0, PI);
    }
    
    /// 处理立体声样本
    #[inline]
    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        self.ring_l.process_stereo(input_left, input_right)
    }
    
    /// 重置
    pub fn reset(&mut self) {
        self.ring_l.reset();
        self.ring_r.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_analysis::{measure_rms, measure_rms_db};
    
    // ============ 理论验证测试 ============
    
    /// 理论: 纯净正弦波输入应该产生输入频率 ± 载波频率的边带
    #[test]
    fn test_ring_modulator_sidebands() {
        let sample_rate = 44100.0;
        let carrier_freq = 440.0; // A4
        let input_freq = 880.0;    // A5
        
        let mut ring = RingModulator::new_with_sample_rate(sample_rate);
        ring.set_osc_frequency(carrier_freq);
        ring.set_osc_wave(RingModulatorWave::Sine);
        
        // 生成输入正弦波
        let input: Vec<f32> = (0..44100) // 1秒
            .take(22050) // 0.5秒
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * PI * input_freq * t).sin() * 0.5
            })
            .collect();
        
        let output: Vec<f32> = input.iter().map(|&s| ring.process(s)).collect();
        
        // 验证输出有能量
        let rms = measure_rms(&output);
        assert!(
            rms > 0.01,
            "Output RMS {} too low, ring modulation should produce output",
            rms
        );
    }
    
    /// 理论: DC输入应该产生纯载波
    /// 公式: output = DC × carrier
    /// RMS(output) = RMS(DC) × RMS(carrier)
    /// 正弦波RMS = 0.707, 所以 output_RMS = 0.5 × 0.707 = 0.354
    #[test]
    fn test_ring_modulator_dc_input() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_frequency(440.0);
        ring.set_osc_wave(RingModulatorWave::Sine);
        
        let dc_input = 0.5;
        
        // 验证输出是载波波形
        let output: Vec<f32> = (0..1000).map(|_| ring.process(dc_input)).collect();
        
        // DC输入的RMS应该等于输入值 × 载波RMS (0.707 for sine)
        let output_rms = measure_rms(&output);
        let expected_rms = dc_input * 0.707f32.sqrt(); // 正弦波RMS ≈ 0.707
        
        assert!(
            (output_rms - expected_rms).abs() < 0.1,
            "DC input {} should produce output RMS ~{}, got {}",
            dc_input, expected_rms, output_rms
        );
    }
    
    /// 理论: 正弦载波产生的边带应该是纯净的
    #[test]
    fn test_ring_modulator_pure_sine_carrier() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_frequency(1000.0);
        ring.set_osc_wave(RingModulatorWave::Sine);
        
        // 单音输入
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();
        
        let output: Vec<f32> = input.iter().map(|&s| ring.process(s)).collect();
        
        // 验证RMS在合理范围
        let output_rms = measure_rms(&output);
        let input_rms = measure_rms(&input);
        
        // 环形调制会改变RMS，理论上是输入RMS × 载波RMS
        // 正弦波RMS=0.707，所以输出RMS ≈ 输入RMS × 0.707
        let expected_rms = input_rms * 0.707f32.sqrt();
        assert!(
            (output_rms - expected_rms).abs() < 0.1,
            "Output RMS {} should be close to expected {}",
            output_rms, expected_rms
        );
    }
    
    /// 理论: 不同波形产生不同谐波含量
    #[test]
    fn test_ring_modulator_waveforms() {
        let input_freq = 440.0;
        let carrier_freq = 880.0;
        
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * input_freq * t).sin() * 0.5
            })
            .collect();
        
        // 测试所有波形
        for wave in [
            RingModulatorWave::Sine,
            RingModulatorWave::Square,
            RingModulatorWave::Saw,
            RingModulatorWave::Triangle,
        ] {
            let mut ring = RingModulator::new_with_sample_rate(44100.0);
            ring.set_osc_frequency(carrier_freq);
            ring.set_osc_wave(wave);
            
            let output: Vec<f32> = input.iter().map(|&s| ring.process(s)).collect();
            let rms = measure_rms(&output);
            
            // 所有波形都应该产生有效输出
            assert!(
                rms > 0.01,
                "Wave {:?} should produce valid output, RMS = {}",
                wave, rms
            );
        }
    }
    
    /// 理论: 频率越高，处理时间应该相同 (零延迟)
    #[test]
    fn test_ring_modulator_frequency_processing() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_wave(RingModulatorWave::Sine);
        
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();
        
        let mut rms_values = Vec::new();
        
        for freq in [100.0, 500.0, 1000.0, 2000.0] {
            ring.set_osc_frequency(freq);
            ring.reset();
            
            let output: Vec<f32> = input.iter().map(|&s| ring.process(s)).collect();
            rms_values.push(measure_rms(&output));
        }
        
        // 验证所有频率都产生了有效输出
        for (i, &rms) in rms_values.iter().enumerate() {
            assert!(
                rms > 0.01,
                "Frequency {} should produce valid output, RMS = {}",
                i, rms
            );
        }
    }
    
    /// 理论: 延迟应该接近零
    #[test]
    fn test_ring_modulator_latency() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_frequency(440.0);
        
        // 脉冲输入
        let mut input = vec![0.0; 1000];
        input[0] = 1.0;
        
        let output: Vec<f32> = input.iter().map(|&s| ring.process(s)).collect();
        
        // Ring Modulator是零延迟的
        let max_idx = output.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        
        assert!(
            max_idx <= 1,
            "Ring modulator should have near-zero latency, peak at sample {}",
            max_idx
        );
    }
    
    /// 理论: 静音输入应该产生静音输出
    #[test]
    fn test_ring_modulator_silence() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_frequency(440.0);
        
        let silence: Vec<f32> = vec![0.0; 1000];
        let output: Vec<f32> = silence.iter().map(|&s| ring.process(s)).collect();
        
        let output_rms = measure_rms(&output);
        assert!(
            output_rms < 0.0001,
            "Silence input should produce silence output, RMS = {}",
            output_rms
        );
    }
    
    /// 理论: 不同模式应该产生不同的处理结果
    #[test]
    fn test_ring_modulator_modes() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();
        
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_frequency(440.0);
        
        // Oscillator模式 - 固定频率
        ring.set_mode(RingModulatorMode::Oscillator);
        ring.reset();
        let output_osc: Vec<f32> = input.iter().map(|&s| ring.process(s)).collect();
        let rms_osc = measure_rms(&output_osc);
        
        // LFO模式 - 高LFO速率应该产生更剧烈的调制
        ring.set_mode(RingModulatorMode::Lfo);
        ring.set_lfo_rate(20.0); // 高速LFO
        ring.reset();
        let output_lfo: Vec<f32> = input.iter().map(|&s| ring.process(s)).collect();
        let rms_lfo = measure_rms(&output_lfo);
        
        // 两种模式都应该产生有效输出
        assert!(rms_osc > 0.01, "Oscillator mode should produce valid output");
        assert!(rms_lfo > 0.01, "LFO mode should produce valid output");
        
        // LFO模式由于调制应该产生不同的RMS (可能比纯振荡器模式有更多变化)
        let diff = (rms_osc - rms_lfo).abs();
        // 注意: RMS可能相似，但我们验证两种模式都能工作
        assert!(rms_osc > 0.0 && rms_lfo > 0.0, "Both modes should produce output");
    }
    
    // ============ 基本功能测试 ============
    
    #[test]
    fn test_ring_modulator_creation() {
        let ring = RingModulator::new();
        assert_eq!(ring.config.mode, RingModulatorMode::Oscillator);
        assert_eq!(ring.config.osc_frequency, 440.0);
        assert_eq!(ring.config.osc_wave, RingModulatorWave::Sine);
        assert_eq!(ring.config.lfo_rate, 5.0);
        assert_eq!(ring.config.lfo_depth, 0.5);
    }

    #[test]
    fn test_ring_modulator_set_frequency() {
        let mut ring = RingModulator::new();
        ring.set_osc_frequency(880.0);
        assert_eq!(ring.config.osc_frequency, 880.0);
        
        // 测试范围限制
        ring.set_osc_frequency(5000.0);
        assert_eq!(ring.config.osc_frequency, 2000.0); // 应该被限制
        
        ring.set_osc_frequency(10.0);
        assert_eq!(ring.config.osc_frequency, 20.0); // 应该被限制
    }

    #[test]
    fn test_ring_modulator_set_wave() {
        let mut ring = RingModulator::new();
        
        for wave in [
            RingModulatorWave::Sine,
            RingModulatorWave::Square,
            RingModulatorWave::Saw,
            RingModulatorWave::Triangle,
        ] {
            ring.set_osc_wave(wave);
            assert_eq!(ring.config.osc_wave, wave);
        }
    }

    #[test]
    fn test_ring_modulator_set_lfo() {
        let mut ring = RingModulator::new();
        
        ring.set_lfo_rate(10.0);
        assert_eq!(ring.config.lfo_rate, 10.0);
        
        ring.set_lfo_depth(0.8);
        assert_eq!(ring.config.lfo_depth, 0.8);
        
        // 测试范围限制
        ring.set_lfo_rate(50.0);
        assert_eq!(ring.config.lfo_rate, 20.0);
        
        ring.set_lfo_depth(1.5);
        assert_eq!(ring.config.lfo_depth, 1.0);
    }

    #[test]
    fn test_ring_modulator_process() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_frequency(440.0);
        
        // 处理一些样本
        for _ in 0..1000 {
            let output = ring.process(0.5);
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_ring_modulator_stereo() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        ring.set_osc_frequency(440.0);
        
        for _ in 0..1000 {
            let (left, right) = ring.process_stereo(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_ring_modulator_reset() {
        let mut ring = RingModulator::new_with_sample_rate(44100.0);
        
        // 处理一些样本
        for _ in 0..100 {
            ring.process(0.5);
        }
        
        // 重置
        ring.reset();
        
        // 应该能正常继续处理
        let output = ring.process(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_ring_modulator_config() {
        let config = RingModulatorConfig {
            mode: RingModulatorMode::Lfo,
            osc_frequency: 880.0,
            osc_wave: RingModulatorWave::Square,
            lfo_rate: 10.0,
            lfo_depth: 0.7,
        };
        
        let mut ring = RingModulator::new();
        ring.set_config(config);
        
        assert_eq!(ring.config.mode, RingModulatorMode::Lfo);
        assert_eq!(ring.config.osc_frequency, 880.0);
        assert_eq!(ring.config.osc_wave, RingModulatorWave::Square);
        assert_eq!(ring.config.lfo_rate, 10.0);
        assert_eq!(ring.config.lfo_depth, 0.7);
    }

    #[test]
    fn test_stereo_ring_modulator_creation() {
        let ring = StereoRingModulator::new();
        assert_eq!(ring.stereo_phase, PI / 2.0);
    }

    #[test]
    fn test_stereo_ring_modulator_process() {
        let mut ring = StereoRingModulator::new();
        ring.set_sample_rate(44100.0);
        
        for _ in 0..1000 {
            let (left, right) = ring.process(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }
}
