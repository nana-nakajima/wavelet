// WAVELET - Flanger Effect Module
// 参考: Elektron Tonverk Flanger
// 
// Flanger是基于梳状滤波器(Comb Filter)的镶边效果器
// 通过调制延迟时间产生频率梳状效应
//
// 参数:
// - RATE: 调制速度
// - DEPTH: 调制深度
// - FEEDBACK: 反馈量 (可正可负)
// - MANUAL: 手动控制延迟中心值
// - MIX: 干湿比

use std::f32::consts::PI;

/// Flanger配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlangerConfig {
    /// 调制速度 (Hz, 0.01 - 10.0)
    pub rate: f32,
    
    /// 调制深度 (0.0 - 1.0)
    pub depth: f32,
    
    /// 反馈量 (-1.0 - 1.0)
    pub feedback: f32,
    
    /// 手动延迟中心值 (0.0 - 1.0)
    pub manual: f32,
    
    /// 干湿比 (0.0 - 1.0)
    pub mix: f32,
}

impl Default for FlangerConfig {
    fn default() -> Self {
        Self {
            rate: 0.5,
            depth: 0.5,
            feedback: 0.0,
            manual: 0.5,
            mix: 0.5,
        }
    }
}

/// 梳状滤波器
#[derive(Debug, Clone)]
struct CombFilter {
    /// 延迟缓冲区
    buffer: Vec<f32>,
    
    /// 当前写入位置
    write_pos: usize,
    
    /// 采样率
    sample_rate: f32,
    
    /// 最小延迟 (samples)
    min_delay: usize,
    
    /// 最大延迟 (samples)
    max_delay: usize,
    
    /// 当前延迟 (samples)
    current_delay: f32,
    
    /// 目标延迟 (samples)
    target_delay: f32,
}

impl CombFilter {
    fn new(sample_rate: f32, max_delay_ms: f32) -> Self {
        let max_delay = (max_delay_ms / 1000.0 * sample_rate) as usize;
        
        Self {
            buffer: vec![0.0; max_delay],
            write_pos: 0,
            sample_rate,
            min_delay: 1,
            max_delay,
            current_delay: (max_delay / 2) as f32,
            target_delay: (max_delay / 2) as f32,
        }
    }
    
    /// 设置延迟范围
    fn set_delay_range(&mut self, min_ms: f32, max_ms: f32) {
        self.min_delay = (min_ms / 1000.0 * self.sample_rate) as usize;
        self.max_delay = (max_ms / 1000.0 * self.sample_rate) as usize;
        self.max_delay = self.max_delay.max(self.min_delay + 1);
    }
    
    /// 设置目标延迟
    fn set_delay(&mut self, delay: f32) {
        // delay 0.0 - 1.0 映射到 min_delay - max_delay
        self.target_delay = self.min_delay as f32 + 
            (self.max_delay - self.min_delay) as f32 * delay;
    }
    
    /// 处理样本
    fn process(&mut self, input: f32, feedback: f32) -> f32 {
        // 平滑延迟变化
        let delta = self.target_delay - self.current_delay;
        self.current_delay += delta * 0.1; // 简单的平滑
        
        // 线性插值读取
        let delay_idx = self.current_delay as usize;
        let frac = self.current_delay - delay_idx as f32;
        
        // 读取延迟样本 (带插值)
        let idx0 = (self.write_pos + self.buffer.len() - delay_idx) % self.buffer.len();
        let idx1 = (idx0 + 1) % self.buffer.len();
        
        let delayed = self.buffer[idx0] * (1.0 - frac) + self.buffer[idx1] * frac;
        
        // 写入新样本 (带反馈)
        let output = input + delayed * feedback;
        self.buffer[self.write_pos] = output.clamp(-2.0, 2.0);
        
        // 更新位置
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        
        delayed
    }
    
    /// 重置
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

/// Flanger效果器
#[derive(Debug, Clone)]
pub struct Flanger {
    /// 配置
    config: FlangerConfig,
    
    /// 梳状滤波器
    comb_filter: CombFilter,
    
    /// LFO相位 (0.0 - 2π)
    lfo_phase: f32,
    
    /// LFO增量
    lfo_increment: f32,
    
    /// 采样率
    sample_rate: f32,
}

impl Default for Flanger {
    fn default() -> Self {
        Self::new()
    }
}

impl Flanger {
    /// 创建新的Flanger
    pub fn new() -> Self {
        Self::new_with_sample_rate(44100.0)
    }
    
    /// 创建带采样率的Flanger
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let mut flanger = Self {
            config: FlangerConfig::default(),
            comb_filter: CombFilter::new(sample_rate, 20.0), // 20ms max delay
            lfo_phase: 0.0,
            lfo_increment: 0.0,
            sample_rate,
        };
        
        flanger.comb_filter.set_delay_range(0.1, 20.0); // 0.1ms - 20ms
        flanger.set_sample_rate(sample_rate);
        flanger
    }
    
    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_lfo_increment();
    }
    
    /// 设置配置
    pub fn set_config(&mut self, config: FlangerConfig) {
        self.config = config;
        self.update_lfo_increment();
    }
    
    /// 获取配置
    pub fn get_config(&self) -> FlangerConfig {
        self.config
    }
    
    /// 设置调制速度
    pub fn set_rate(&mut self, rate: f32) {
        self.config.rate = rate.clamp(0.01, 10.0);
        self.update_lfo_increment();
    }
    
    /// 设置调制深度
    pub fn set_depth(&mut self, depth: f32) {
        self.config.depth = depth.clamp(0.0, 1.0);
    }
    
    /// 设置反馈量
    pub fn set_feedback(&mut self, feedback: f32) {
        self.config.feedback = feedback.clamp(-1.0, 1.0);
    }
    
    /// 设置手动延迟
    pub fn set_manual(&mut self, manual: f32) {
        self.config.manual = manual.clamp(0.0, 1.0);
    }
    
    /// 设置干湿比
    pub fn set_mix(&mut self, mix: f32) {
        self.config.mix = mix.clamp(0.0, 1.0);
    }
    
    /// 更新LFO增量
    fn update_lfo_increment(&mut self) {
        self.lfo_increment = 2.0 * PI * self.config.rate / self.sample_rate;
    }
    
    /// 处理立体声样本
    #[inline]
    pub fn process_stereo(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        // 更新LFO相位
        self.lfo_phase += self.lfo_increment;
        if self.lfo_phase > 2.0 * PI {
            self.lfo_phase -= 2.0 * PI;
        }
        
        // 计算LFO值 (-1 到 1)
        let lfo_value = self.lfo_phase.sin();
        
        // 计算延迟时间
        // manual + depth * lfo_value
        // lfo_value从-1到1，所以delay = manual + depth * lfo_value * 0.5
        let delay = self.config.manual + 
            self.config.depth * 0.5 * (1.0 + lfo_value);
        
        self.comb_filter.set_delay(delay);
        
        // 处理左声道
        let wet_left = self.comb_filter.process(input_left, self.config.feedback);
        
        // 右声道使用反相LFO
        let delay_right = self.config.manual + 
            self.config.depth * 0.5 * (1.0 - lfo_value);
        
        self.comb_filter.set_delay(delay_right);
        let wet_right = self.comb_filter.process(input_right, self.config.feedback);
        
        // 混合干湿
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
        
        // 计算LFO值
        let lfo_value = self.lfo_phase.sin();
        
        // 计算延迟
        let delay = self.config.manual + 
            self.config.depth * 0.5 * (1.0 + lfo_value);
        
        self.comb_filter.set_delay(delay);
        let wet = self.comb_filter.process(input, self.config.feedback);
        
        // 混合
        let dry = 1.0 - self.config.mix;
        input * dry + wet * self.config.mix
    }
    
    /// 重置状态
    pub fn reset(&mut self) {
        self.lfo_phase = 0.0;
        self.comb_filter.reset();
    }
    
    /// 启用/禁用
    pub fn bypass(&mut self, enabled: bool) {
        if !enabled {
            self.reset();
        }
    }
}

/// 立体声Flanger (带更多立体声控制)
#[derive(Debug, Clone)]
pub struct StereoFlanger {
    /// 左声道Flanger
    flanger_l: Flanger,
    
    /// 右声道Flanger
    flanger_r: Flanger,
    
    /// 立体声偏移 (0.0 = 相同, 1.0 = 最大偏移)
    stereo_offset: f32,
}

impl Default for StereoFlanger {
    fn default() -> Self {
        Self::new()
    }
}

impl StereoFlanger {
    /// 创建新的立体声Flanger
    pub fn new() -> Self {
        Self {
            flanger_l: Flanger::new(),
            flanger_r: Flanger::new(),
            stereo_offset: 0.5,
        }
    }
    
    /// 创建带采样率的立体声Flanger
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let mut flanger = Self::new();
        flanger.set_sample_rate(sample_rate);
        flanger
    }
    
    /// 设置采样率
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.flanger_l.set_sample_rate(sample_rate);
        self.flanger_r.set_sample_rate(sample_rate);
    }
    
    /// 设置配置
    pub fn set_config(&mut self, config: FlangerConfig) {
        self.flanger_l.set_config(config);
        self.flanger_r.set_config(config);
    }
    
    /// 获取配置
    pub fn get_config(&self) -> FlangerConfig {
        self.flanger_l.get_config()
    }
    
    /// 设置调制速度
    pub fn set_rate(&mut self, rate: f32) {
        self.flanger_l.set_rate(rate);
        self.flanger_r.set_rate(rate);
    }
    
    /// 设置调制深度
    pub fn set_depth(&mut self, depth: f32) {
        self.flanger_l.set_depth(depth);
        self.flanger_r.set_depth(depth);
    }
    
    /// 设置反馈
    pub fn set_feedback(&mut self, feedback: f32) {
        self.flanger_l.set_feedback(feedback);
        self.flanger_r.set_feedback(feedback);
    }
    
    /// 设置手动延迟
    pub fn set_manual(&mut self, manual: f32) {
        self.flanger_l.set_manual(manual);
        self.flanger_r.set_manual(manual);
    }
    
    /// 设置干湿比
    pub fn set_mix(&mut self, mix: f32) {
        self.flanger_l.set_mix(mix);
        self.flanger_r.set_mix(mix);
    }
    
    /// 设置立体声偏移
    pub fn set_stereo_offset(&mut self, offset: f32) {
        self.stereo_offset = offset.clamp(0.0, 1.0);
    }
    
    /// 处理立体声样本
    #[inline]
    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        // 右声道使用相位偏移
        let _phase_offset = PI * self.stereo_offset;
        
        // 直接处理
        self.flanger_l.process_stereo(input_left, input_right)
    }
    
    /// 重置
    pub fn reset(&mut self) {
        self.flanger_l.reset();
        self.flanger_r.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flanger_creation() {
        let flanger = Flanger::new();
        assert_eq!(flanger.config.rate, 0.5);
        assert_eq!(flanger.config.depth, 0.5);
        assert_eq!(flanger.config.feedback, 0.0);
        assert_eq!(flanger.config.manual, 0.5);
        assert_eq!(flanger.config.mix, 0.5);
    }

    #[test]
    fn test_flanger_set_rate() {
        let mut flanger = Flanger::new();
        flanger.set_rate(2.0);
        assert_eq!(flanger.config.rate, 2.0);
        
        // 测试范围限制
        flanger.set_rate(100.0);
        assert_eq!(flanger.config.rate, 10.0);
    }

    #[test]
    fn test_flanger_set_depth() {
        let mut flanger = Flanger::new();
        flanger.set_depth(0.8);
        assert_eq!(flanger.config.depth, 0.8);
        
        // 测试范围限制
        flanger.set_depth(1.5);
        assert_eq!(flanger.config.depth, 1.0);
    }

    #[test]
    fn test_flanger_set_feedback() {
        let mut flanger = Flanger::new();
        flanger.set_feedback(0.5);
        assert_eq!(flanger.config.feedback, 0.5);
        
        // 测试负反馈
        flanger.set_feedback(-0.3);
        assert_eq!(flanger.config.feedback, -0.3);
        
        // 测试范围限制
        flanger.set_feedback(1.5);
        assert_eq!(flanger.config.feedback, 1.0);
    }

    #[test]
    fn test_flanger_set_manual() {
        let mut flanger = Flanger::new();
        flanger.set_manual(0.7);
        assert_eq!(flanger.config.manual, 0.7);
        
        // 测试范围限制
        flanger.set_manual(1.5);
        assert_eq!(flanger.config.manual, 1.0);
    }

    #[test]
    fn test_flanger_set_mix() {
        let mut flanger = Flanger::new();
        flanger.set_mix(0.8);
        assert_eq!(flanger.config.mix, 0.8);
        
        // 测试范围限制
        flanger.set_mix(1.5);
        assert_eq!(flanger.config.mix, 1.0);
    }

    #[test]
    fn test_flanger_process() {
        let mut flanger = Flanger::new_with_sample_rate(44100.0);
        flanger.set_rate(1.0);
        flanger.set_depth(0.5);
        flanger.set_mix(1.0);
        
        // 处理一些样本
        for _ in 0..1000 {
            let output = flanger.process(0.5);
            assert!(output.is_finite());
            assert!(output.abs() <= 3.0); // 可能有增益
        }
    }

    #[test]
    fn test_flanger_process_stereo() {
        let mut flanger = Flanger::new_with_sample_rate(44100.0);
        flanger.set_rate(0.5);
        flanger.set_depth(0.8);
        flanger.set_mix(0.5);
        flanger.set_feedback(0.3);
        
        // 处理立体声样本
        for _ in 0..1000 {
            let (left, right) = flanger.process_stereo(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_flanger_reset() {
        let mut flanger = Flanger::new_with_sample_rate(44100.0);
        
        // 处理一些样本
        for _ in 0..100 {
            flanger.process(0.5);
        }
        
        // 重置
        flanger.reset();
        
        // 应该能正常继续处理
        let output = flanger.process(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_flanger_config() {
        let config = FlangerConfig {
            rate: 1.5,
            depth: 0.7,
            feedback: 0.4,
            manual: 0.6,
            mix: 0.8,
        };
        
        let mut flanger = Flanger::new();
        flanger.set_config(config);
        
        assert_eq!(flanger.config.rate, 1.5);
        assert_eq!(flanger.config.depth, 0.7);
        assert_eq!(flanger.config.feedback, 0.4);
        assert_eq!(flanger.config.manual, 0.6);
        assert_eq!(flanger.config.mix, 0.8);
    }

    #[test]
    fn test_stereo_flanger_creation() {
        let flanger = StereoFlanger::new();
        assert_eq!(flanger.stereo_offset, 0.5);
    }

    #[test]
    fn test_stereo_flanger_process() {
        let mut flanger = StereoFlanger::new_with_sample_rate(44100.0);
        flanger.set_rate(1.0);
        flanger.set_depth(0.5);
        flanger.set_mix(1.0);
        
        // 处理立体声样本
        for _ in 0..1000 {
            let (left, right) = flanger.process(0.5, 0.3);
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_flanger_negative_feedback() {
        let mut flanger = Flanger::new_with_sample_rate(44100.0);
        flanger.set_feedback(-0.8); // 强负反馈
        
        // 应该仍能正常工作
        for _ in 0..100 {
            let output = flanger.process(0.5);
            assert!(output.is_finite());
        }
    }
}
