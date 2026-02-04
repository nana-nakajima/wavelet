// WAVELET - Audio Analysis Module
//
// 音频分析工具，用于测试和验证音频处理模块
// 包括：RMS测量、峰值检测、频谱分析、延迟测量等

#![allow(dead_code)] // Analysis functions reserved for future use

use std::f32::consts::PI;

/// 测量RMS电平
#[inline]
pub fn measure_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// 测量峰值电平 (绝对值最大)
#[inline]
pub fn measure_peak(samples: &[f32]) -> f32 {
    let mut max_val: f32 = 0.0;
    for &s in samples {
        let abs_val = s.abs();
        if abs_val > max_val {
            max_val = abs_val;
        }
    }
    max_val
}

/// 计算增益 (dB)
#[inline]
pub fn gain_db(gain: f32) -> f32 {
    if gain <= 0.0 {
        -100.0 // 极小值表示无声
    } else {
        20.0 * gain.log10()
    }
}

/// 计算增益比
#[inline]
pub fn gain_ratio(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

/// 测量RMS电平 (dB)
#[inline]
pub fn measure_rms_db(samples: &[f32]) -> f32 {
    gain_db(measure_rms(samples))
}

/// 测量峰值电平 (dB)
#[inline]
pub fn measure_peak_db(samples: &[f32]) -> f32 {
    gain_db(measure_peak(samples))
}

/// 测量动态范围 (dB)
#[inline]
pub fn measure_dynamic_range(peaks: &[f32]) -> f32 {
    let mut max_peak: f32 = 0.0;
    let mut min_peak: f32 = f32::MAX;

    for &p in peaks {
        if p > max_peak {
            max_peak = p;
        }
        if p < min_peak {
            min_peak = p;
        }
    }

    if min_peak <= 0.0 {
        max_peak // 如果有 silence，返回最大值
    } else {
        gain_db(max_peak / min_peak)
    }
}

/// 交叉相关测量 (用于延迟检测)
pub fn measure_cross_correlation(signal1: &[f32], signal2: &[f32], max_lag: usize) -> f32 {
    let len = signal1.len().min(signal2.len()).saturating_sub(max_lag);
    if len == 0 {
        return 0.0;
    }

    let mut max_corr: f32 = 0.0;

    for lag in 0..=max_lag {
        let mut sum: f32 = 0.0;
        for i in 0..len {
            let s1_idx = i;
            let s2_idx = i + lag;
            if s2_idx < signal2.len() {
                sum += signal1[s1_idx] * signal2[s2_idx];
            }
        }
        if sum.abs() > max_corr {
            max_corr = sum.abs();
        }
    }

    max_corr
}

/// 立体声相关性系数 (-1.0 到 1.0)
pub fn measure_stereo_correlation(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }

    let n = left.len() as f32;

    // 计算均值
    let mean_l: f32 = left.iter().sum();
    let mean_r: f32 = right.iter().sum();

    // 计算协方差
    let mut cov = 0.0;
    let mut var_l = 0.0;
    let mut var_r = 0.0;

    for (l, r) in left.iter().zip(right.iter()) {
        let dl = l - mean_l / n;
        let dr = r - mean_r / n;
        cov += dl * dr;
        var_l += dl * dl;
        var_r += dr * dr;
    }

    if var_l <= 0.0 || var_r <= 0.0 {
        return 0.0;
    }

    cov / (var_l.sqrt() * var_r.sqrt())
}

/// 简单的FFT分析 (返回频率箱电平)
/// 注意：这是一个简化的实现，实际项目中应该使用专业库
pub struct SpectrumAnalyzer {
    /// 频率箱数量
    bins: usize,

    /// 采样率
    sample_rate: f32,

    /// 汉宁窗缓存
    window: Vec<f32>,
}

impl SpectrumAnalyzer {
    /// 创建新的频谱分析仪
    pub fn new(bins: usize, sample_rate: f32) -> Self {
        let window: Vec<f32> = (0..bins)
            .map(|i| {
                let x = i as f32 / (bins - 1) as f32;
                0.5 * (1.0 - (2.0 * PI * x).cos())
            })
            .collect();

        Self {
            bins,
            sample_rate,
            window,
        }
    }

    /// 分析频谱，返回每个频率箱的电平 (dB)
    pub fn analyze(&self, samples: &[f32]) -> Vec<f32> {
        let len = samples.len().min(self.bins);

        // 预处理：应用窗函数
        let windowed: Vec<f32> = (0..len)
            .zip(&self.window)
            .map(|(i, &w)| samples.get(i).unwrap_or(&0.0) * w)
            .collect();

        // 简化的频谱计算 (幅度谱)
        let mut spectrum = vec![-100.0; self.bins];

        // 计算DC分量和Nyquist
        let sum: f32 = windowed.iter().sum();
        spectrum[0] = gain_db(sum.abs() / len as f32);

        if self.bins > 2 {
            // 简化的频率箱计算
            #[allow(clippy::needless_range_loop)]
            for i in 1..(self.bins - 1) {
                let freq_low = i as f32 * self.sample_rate / (2.0 * self.bins as f32);
                let freq_high = (i + 1) as f32 * self.sample_rate / (2.0 * self.bins as f32);

                // 计算该频段的RMS
                let sum_sq: f32 = windowed
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| {
                        let freq = j as f32 * self.sample_rate / (2.0 * len as f32);
                        freq >= freq_low && freq < freq_high
                    })
                    .map(|(_, &s)| s * s)
                    .sum();

                let count = windowed
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| {
                        let freq = j as f32 * self.sample_rate / (2.0 * len as f32);
                        freq >= freq_low && freq < freq_high
                    })
                    .count();

                if count > 0 {
                    spectrum[i] = gain_db((sum_sq / count as f32).sqrt());
                }
            }
        }

        spectrum
    }

    /// 获取频率分辨率 (Hz/bin)
    pub fn frequency_resolution(&self) -> f32 {
        self.sample_rate / (2.0 * self.bins as f32)
    }

    /// 获取指定频率所在的箱索引
    pub fn frequency_to_bin(&self, frequency: f32) -> usize {
        (frequency / self.frequency_resolution()) as usize
    }
}

/// 延迟测量器 - reserved for future latency measurement features
#[allow(dead_code)]
pub struct LatencyMeasurer {
    /// 已知延迟 (samples)
    known_delay: usize,

    /// 互相关结果
    cross_correlation: Vec<f32>,
}

#[allow(dead_code)]
impl LatencyMeasurer {
    /// 创建新的延迟测量器
    pub fn new(known_delay: usize) -> Self {
        Self {
            known_delay,
            cross_correlation: Vec::new(),
        }
    }

    /// 测量输入和输出的延迟
    /// 使用简单的互相关方法
    pub fn measure_latency(&mut self, input: &[f32], output: &[f32], max_lag: usize) -> usize {
        self.cross_correlation.clear();

        // 计算互相关
        for lag in 0..=max_lag {
            let effective_len = input.len().min(output.len().saturating_sub(lag));
            if effective_len == 0 {
                self.cross_correlation.push(0.0);
                continue;
            }

            let mut sum = 0.0;
            for i in 0..effective_len {
                sum += input[i] * output[i + lag];
            }
            self.cross_correlation.push(sum);
        }

        // 找到最大相关值的位置
        let mut max_idx = 0;
        let mut max_val = f32::MIN;

        for (i, &val) in self.cross_correlation.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }

        max_idx
    }

    /// 获取互相关结果
    pub fn get_cross_correlation(&self) -> &[f32] {
        &self.cross_correlation
    }
}

/// 谐波失真分析器 - reserved for future FFT-based analysis
#[allow(dead_code)]
pub struct HarmonicDistortionAnalyzer {
    /// 采样率
    sample_rate: f32,

    /// 基波频率
    fundamental_freq: f32,

    /// 分析窗口大小
    window_size: usize,
}

#[allow(dead_code)]
impl HarmonicDistortionAnalyzer {
    /// 创建新的谐波失真分析器
    pub fn new(sample_rate: f32, fundamental_freq: f32) -> Self {
        Self {
            sample_rate,
            fundamental_freq,
            window_size: (sample_rate / fundamental_freq * 4.0) as usize, // 至少4个周期
        }
    }

    /// 测量THD+N (总谐波失真 + 噪声)
    pub fn measure_thd_plus_n(&self, samples: &[f32]) -> f32 {
        // 简化的THD+N计算
        // 实际应该使用FFT分离基波和各次谐波

        let rms_total = measure_rms(samples);

        // 估计基波幅度 (简化版：使用带通滤波后的RMS)
        let fundamental_rms = self.estimate_fundamental_rms(samples);

        if fundamental_rms <= 0.0 {
            return 0.0;
        }

        // THD+N = sqrt(total^2 - fundamental^2) / fundamental
        let harmonic_noise = (rms_total * rms_total - fundamental_rms * fundamental_rms).sqrt();
        harmonic_noise / fundamental_rms
    }

    /// 估计基波RMS
    fn estimate_fundamental_rms(&self, samples: &[f32]) -> f32 {
        // 简化的基波估计：计算与正弦波的互相关
        let period = self.sample_rate / self.fundamental_freq;
        let n_periods = (samples.len() as f32 / period).floor() as usize;

        if n_periods == 0 {
            return measure_rms(samples);
        }

        let mut sum = 0.0;
        let mut count = 0;

        for i in 0..n_periods {
            let idx = (i as f32 * period) as usize;
            if idx < samples.len() {
                sum += samples[idx];
                count += 1;
            }
        }

        if count > 0 {
            // 简化：返回平均值作为基波估计
            sum / count as f32
        } else {
            measure_rms(samples)
        }
    }

    /// 计算THD (dB)
    pub fn measure_thd_db(&self, samples: &[f32]) -> f32 {
        let thd = self.measure_thd_plus_n(samples);
        if thd <= 0.0 {
            -100.0
        } else {
            20.0 * thd.log10()
        }
    }
}

/// 效果器测试断言辅助
pub struct AudioAssertions;

impl AudioAssertions {
    /// 断言输出在合理范围内
    pub fn assert_within_range(samples: &[f32], min: f32, max: f32) {
        for (i, &sample) in samples.iter().enumerate() {
            assert!(
                sample >= min && sample <= max,
                "Sample {} out of range: {} (expected [{}, {}])",
                i,
                sample,
                min,
                max
            );
        }
    }

    /// 断言RMS变化在预期范围内
    pub fn assert_rms_change(input: &[f32], output: &[f32], max_change_db: f32) {
        let input_rms = measure_rms_db(input);
        let output_rms = measure_rms_db(output);
        let change = (output_rms - input_rms).abs();

        assert!(
            change <= max_change_db,
            "RMS change {} dB exceeds limit {} dB",
            change,
            max_change_db
        );
    }

    /// 断言无直流偏移
    pub fn assert_no_dc_offset(samples: &[f32], tolerance: f32) {
        let mean: f32 = samples.iter().sum();
        let dc_offset = mean / samples.len() as f32;

        assert!(
            dc_offset.abs() <= tolerance,
            "DC offset detected: {} (tolerance: {})",
            dc_offset,
            tolerance
        );
    }

    /// 断言立体声相关性在预期范围内
    pub fn assert_stereo_correlation(left: &[f32], right: &[f32], min_corr: f32, max_corr: f32) {
        let correlation = measure_stereo_correlation(left, right);

        assert!(
            correlation >= min_corr && correlation <= max_corr,
            "Stereo correlation {} out of range [{}, {}]",
            correlation,
            min_corr,
            max_corr
        );
    }

    /// 断言频率响应在预期范围内
    pub fn assert_frequency_response(spectrum: &[f32], min_db: f32, max_db: f32) {
        for (i, &band_db) in spectrum.iter().enumerate() {
            assert!(
                band_db >= min_db && band_db <= max_db,
                "Band {} out of range: {} dB (expected [{} dB, {} dB])",
                i,
                band_db,
                min_db,
                max_db
            );
        }
    }

    /// 断言延迟在预期范围内
    pub fn assert_latency_measured(measured: usize, expected: usize, tolerance: usize) {
        let diff = measured.abs_diff(expected);

        assert!(
            diff <= tolerance,
            "Latency {} differs from expected {} by {} (tolerance: {})",
            measured,
            expected,
            diff,
            tolerance
        );
    }

    /// 断言THD在预期范围内
    pub fn assert_thd_below(
        samples: &[f32],
        max_thd_db: f32,
        sample_rate: f32,
        fundamental_freq: f32,
    ) {
        let analyzer = HarmonicDistortionAnalyzer::new(sample_rate, fundamental_freq);
        let thd_db = analyzer.measure_thd_db(samples);

        assert!(
            thd_db <= max_thd_db,
            "THD {} dB exceeds limit {} dB",
            thd_db,
            max_thd_db
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_rms() {
        // 0.5 的 RMS 应该是 0.5
        let samples = vec![0.5; 100];
        assert!((measure_rms(&samples) - 0.5).abs() < 0.001);

        // 静音应该返回 0
        let silence = vec![0.0; 100];
        assert_eq!(measure_rms(&silence), 0.0);
    }

    #[test]
    fn test_measure_peak() {
        let samples = vec![0.3, -0.8, 0.5, -0.2];
        assert_eq!(measure_peak(&samples), 0.8);
    }

    #[test]
    fn test_gain_db() {
        // 0 dB = 增益 1.0
        assert!((gain_db(1.0) - 0.0).abs() < 0.001);

        // -6 dB ≈ 0.5
        assert!((gain_db(0.5) - (-6.02)).abs() < 0.1);

        // -20 dB ≈ 0.1
        assert!((gain_db(0.1) - (-20.0)).abs() < 0.1);
    }

    #[test]
    fn test_measure_rms_db() {
        let samples = vec![1.0; 100];
        let db = measure_rms_db(&samples);
        assert!((db - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_stereo_correlation() {
        // 相同信号 = 应该高度相关
        let left: Vec<f32> = (0..100).map(|i| (i as f32 / 10.0).sin() * 0.5).collect();
        let right = left.clone();
        let corr = measure_stereo_correlation(&left, &right);
        assert!(
            corr > 0.99,
            "Same signals should have high correlation, got {}",
            corr
        );

        // 反相信号 = 应该高度负相关
        let right_inv: Vec<f32> = left.iter().map(|&s| -s).collect();
        let corr_inv = measure_stereo_correlation(&left, &right_inv);
        assert!(
            corr_inv < -0.99,
            "Inverted signals should have high negative correlation, got {}",
            corr_inv
        );

        // 独立信号 = 相关性在有效范围内
        let left_rand = vec![0.1, 0.2, -0.3, 0.4, -0.5];
        let right_rand = vec![-0.5, 0.4, -0.3, 0.2, 0.1];
        let corr_rand = measure_stereo_correlation(&left_rand, &right_rand);
        assert!(corr_rand >= -1.0 && corr_rand <= 1.0);
    }

    #[test]
    fn test_spectrum_analyzer() {
        let analyzer = SpectrumAnalyzer::new(1024, 44100.0);

        // 1kHz正弦波
        let freq = 1000.0;
        let samples: Vec<f32> = (0..1024)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * freq * t).sin() * 0.5
            })
            .collect();

        let spectrum = analyzer.analyze(&samples);

        let mut max_db: f32 = -1000.0;
        for &b in &spectrum {
            if b > max_db {
                max_db = b;
            }
        }

        // 应该有频率分辨率以上的变化
        let mut min_db: f32 = 1000.0;
        for &b in &spectrum {
            if b < min_db {
                min_db = b;
            }
        }
        assert!(
            max_db - min_db > 10.0,
            "Spectrum should have some variation"
        );
    }

    #[test]
    fn test_latency_measurer() {
        let mut measurer = LatencyMeasurer::new(100);

        // 使用脉冲信号测试延迟
        let input_len = 100;
        let delay = 50;
        let mut input = vec![0.0; input_len];
        input[0] = 1.0; // 脉冲

        let output_len = input_len + delay + 10;
        let mut output = vec![0.0; output_len];
        for i in 0..input_len {
            output[i + delay] = input[i]; // 50 samples延迟
        }

        let latency = measurer.measure_latency(&input, &output, 100);

        // 应该检测到50 samples的延迟
        assert!(
            latency >= 45 && latency <= 55,
            "Latency {} not in expected range [45, 55]",
            latency
        );
    }

    #[test]
    fn test_harmonic_distortion_analyzer() {
        // 纯净正弦波应该有很低的THD
        let freq = 440.0;
        let samples: Vec<f32> = (0..44100)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * freq * t).sin() * 0.5
            })
            .take(22050) // 0.5秒
            .collect();

        let analyzer = HarmonicDistortionAnalyzer::new(44100.0, freq);
        let thd_db = analyzer.measure_thd_db(&samples);

        // 纯净正弦波THD应该很低 (< -60 dB)
        assert!(thd_db < -60.0, "THD {} dB too high for pure sine", thd_db);
    }

    #[test]
    fn test_audio_assertions() {
        // 使用零均值信号测试DC offset (使用完整周期)
        let samples: Vec<f32> = (0..628) // 100个完整周期 (2π * 100)
            .map(|i| (i as f32 / 100.0).sin() * 0.1)
            .collect();

        AudioAssertions::assert_within_range(&samples, -1.0, 1.0);
        AudioAssertions::assert_no_dc_offset(&samples, 0.01);

        // 使用相同的零均值信号
        let left: Vec<f32> = (0..628).map(|i| (i as f32 / 10.0).sin() * 0.5).collect();
        let right = left.clone();
        AudioAssertions::assert_stereo_correlation(&left, &right, 0.99, 1.0);
    }

    #[test]
    fn test_empty_samples() {
        assert_eq!(measure_rms(&[]), 0.0);
        assert_eq!(measure_peak(&[]), 0.0);
        assert_eq!(measure_rms_db(&[]), -100.0);
        assert_eq!(measure_peak_db(&[]), -100.0);
    }
}
