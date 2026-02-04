// WAVELET - Filter Bank Effect Module
// 参考: Elektron Tonverk Filter Bank
//
// Filter Bank (滤波器组) 提供8个并行滤波器，每个都有独立的频率和增益控制

use std::f32::consts::PI;

/// 滤波器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterBankType {
    LowShelf,
    Peak,
    HighShelf,
    BandPass,
}

/// 单个滤波器带配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterBandConfig {
    pub filter_type: FilterBankType,
    pub frequency: f32,
    pub gain_db: f32,
    pub q: f32,
}

impl Default for FilterBandConfig {
    fn default() -> Self {
        Self {
            filter_type: FilterBankType::Peak,
            frequency: 1000.0,
            gain_db: 0.0,
            q: 1.0,
        }
    }
}

/// Filter Bank配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterBankConfig {
    pub filters: [FilterBandConfig; 8],
    pub input_gain_db: f32,
    pub output_gain_db: f32,
}

impl Default for FilterBankConfig {
    fn default() -> Self {
        Self {
            filters: [
                FilterBandConfig {
                    filter_type: FilterBankType::LowShelf,
                    frequency: 31.5,
                    gain_db: 0.0,
                    q: 0.7,
                },
                FilterBandConfig {
                    filter_type: FilterBankType::Peak,
                    frequency: 63.0,
                    gain_db: 0.0,
                    q: 1.0,
                },
                FilterBandConfig {
                    filter_type: FilterBankType::Peak,
                    frequency: 125.0,
                    gain_db: 0.0,
                    q: 1.0,
                },
                FilterBandConfig {
                    filter_type: FilterBankType::Peak,
                    frequency: 250.0,
                    gain_db: 0.0,
                    q: 1.0,
                },
                FilterBandConfig {
                    filter_type: FilterBankType::Peak,
                    frequency: 500.0,
                    gain_db: 0.0,
                    q: 1.0,
                },
                FilterBandConfig {
                    filter_type: FilterBankType::Peak,
                    frequency: 1000.0,
                    gain_db: 0.0,
                    q: 1.0,
                },
                FilterBandConfig {
                    filter_type: FilterBankType::Peak,
                    frequency: 2000.0,
                    gain_db: 0.0,
                    q: 1.0,
                },
                FilterBandConfig {
                    filter_type: FilterBankType::HighShelf,
                    frequency: 4000.0,
                    gain_db: 0.0,
                    q: 0.7,
                },
            ],
            input_gain_db: 0.0,
            output_gain_db: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn configure_peak(&mut self, freq: f32, q: f32, gain_db: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let a = 10.0f32.powf(gain_db / 40.0);
        let a_sqrt = a.sqrt();

        self.b0 = 1.0 + alpha * a_sqrt;
        self.b1 = -2.0 * cos_omega;
        self.b2 = 1.0 - alpha * a_sqrt;
        self.a1 = -2.0 * cos_omega;
        self.a2 = 1.0 - alpha * a_sqrt;

        let norm = 1.0 + alpha / a_sqrt;
        self.b0 /= norm;
        self.b1 /= norm;
        self.b2 /= norm;
        self.a1 /= norm;
        self.a2 /= norm;
    }

    fn configure_low_shelf(&mut self, freq: f32, q: f32, gain_db: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let _alpha = sin_omega / (2.0 * q);
        let a = 10.0f32.powf(gain_db / 40.0);

        let a_plus_1 = a + 1.0;
        let a_minus_1 = a - 1.0;
        let cos_omega_2 = cos_omega * 2.0;
        let beta = sin_omega * a.sqrt();

        self.b0 = a * (a_plus_1 - a_minus_1 * cos_omega_2 + beta) / 2.0;
        self.b1 = a * 2.0 * (a_minus_1 - a_plus_1 * cos_omega_2) / 2.0;
        self.b2 = a * (a_plus_1 - a_minus_1 * cos_omega_2 - beta) / 2.0;
        self.a1 = -2.0 * (a_minus_1 + a_plus_1 * cos_omega_2) / 2.0;
        self.a2 = (a_plus_1 + a_minus_1 * cos_omega_2 - beta) / 2.0;

        let norm = a_plus_1 + a_minus_1 * cos_omega_2 + beta;
        self.b0 /= norm;
        self.b1 /= norm;
        self.b2 /= norm;
        self.a1 /= norm;
        self.a2 /= norm;
    }

    fn configure_high_shelf(&mut self, freq: f32, q: f32, gain_db: f32, sample_rate: f32) {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let _alpha = sin_omega / (2.0 * q);
        let a = 10.0f32.powf(gain_db / 40.0);

        let a_plus_1 = a + 1.0;
        let a_minus_1 = a - 1.0;
        let cos_omega_2 = cos_omega * 2.0;
        let beta = sin_omega * a.sqrt();

        self.b0 = a * (a_plus_1 + a_minus_1 * cos_omega_2 + beta) / 2.0;
        self.b1 = -a * 2.0 * (a_minus_1 + a_plus_1 * cos_omega_2) / 2.0;
        self.b2 = a * (a_plus_1 + a_minus_1 * cos_omega_2 - beta) / 2.0;
        self.a1 = 2.0 * (a_minus_1 + a_plus_1 * cos_omega_2) / 2.0;
        self.a2 = -(a_plus_1 + a_minus_1 * cos_omega_2 - beta) / 2.0;

        let norm = a_plus_1 + a_minus_1 * cos_omega_2 + beta;
        self.b0 /= norm;
        self.b1 /= norm;
        self.b2 /= norm;
        self.a1 /= norm;
        self.a2 /= norm;
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        // 防止数值不稳定
        if !output.is_finite() {
            0.0
        } else {
            output
        }
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct FilterBank {
    config: FilterBankConfig,
    sample_rate: f32,
    filters: [BiquadFilter; 8],
    input_gain: f32,
    output_gain: f32,
}

impl Default for FilterBank {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterBank {
    pub fn new() -> Self {
        Self::new_with_sample_rate(44100.0)
    }

    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let filters: [BiquadFilter; 8] = [BiquadFilter::new(); 8];
        let mut bank = Self {
            config: FilterBankConfig::default(),
            sample_rate,
            filters,
            input_gain: 1.0,
            output_gain: 1.0,
        };
        bank.update_filters();
        bank
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_filters();
    }

    pub fn set_config(&mut self, config: FilterBankConfig) {
        self.config = config;
        self.input_gain = 10.0f32.powf(self.config.input_gain_db / 20.0);
        self.output_gain = 10.0f32.powf(self.config.output_gain_db / 20.0);
        self.update_filters();
    }

    pub fn get_config(&self) -> FilterBankConfig {
        self.config
    }

    pub fn set_filter(&mut self, index: usize, mut config: FilterBandConfig) {
        if index >= 8 {
            return;
        }
        // 边界检查
        config.gain_db = config.gain_db.clamp(-12.0, 12.0);
        config.q = config.q.clamp(0.1, 20.0);
        config.frequency = config.frequency.clamp(20.0, 20000.0);
        self.config.filters[index] = config;
        self.update_single_filter(index);
    }

    fn update_filters(&mut self) {
        for i in 0..8 {
            self.update_single_filter(i);
        }
    }

    fn update_single_filter(&mut self, index: usize) {
        if index >= 8 {
            return;
        }
        let filter = self.config.filters[index];
        let f = &mut self.filters[index];
        match filter.filter_type {
            FilterBankType::Peak => {
                f.configure_peak(filter.frequency, filter.q, filter.gain_db, self.sample_rate)
            }
            FilterBankType::LowShelf => {
                f.configure_low_shelf(filter.frequency, filter.q, filter.gain_db, self.sample_rate)
            }
            FilterBankType::HighShelf => {
                f.configure_high_shelf(filter.frequency, filter.q, filter.gain_db, self.sample_rate)
            }
            FilterBankType::BandPass => {
                f.configure_peak(filter.frequency, filter.q, 0.0, self.sample_rate)
            }
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let input = input * self.input_gain;
        let mut sum = 0.0;
        for filter in &mut self.filters {
            sum += filter.process(input);
        }
        let output = sum / 8.0 * self.output_gain;
        // 钳制输出到合理范围
        output.clamp(-10.0, 10.0)
    }

    #[inline]
    pub fn process_stereo(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        (self.process(input_left), self.process(input_right))
    }

    pub fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
        }
    }
    pub fn get_latency(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_analysis::measure_rms;

    // ============ 理论验证测试 ============

    #[test]
    fn test_filter_bank_zero_gain() {
        let input: Vec<f32> = (0..44100)
            .take(22050)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * PI * 440.0 * t).sin() * 0.5
            })
            .collect();

        let mut bank = FilterBank::new_with_sample_rate(44100.0);
        for i in 0..8 {
            let mut config = FilterBandConfig::default();
            config.gain_db = 0.0;
            bank.set_filter(i, config);
        }

        let output: Vec<f32> = input.iter().map(|&s| bank.process(s)).collect();
        let diff = (measure_rms(&input) - measure_rms(&output)).abs();
        assert!(
            diff < 0.1,
            "Zero gain should preserve signal, RMS diff = {}",
            diff
        );
    }

    #[test]
    fn test_filter_bank_silence() {
        let mut bank = FilterBank::new_with_sample_rate(44100.0);
        let mut config = FilterBandConfig::default();
        config.gain_db = 12.0;
        bank.set_filter(0, config);

        let silence: Vec<f32> = vec![0.0; 1000];
        let output: Vec<f32> = silence.iter().map(|&s| bank.process(s)).collect();
        let output_rms = measure_rms(&output);
        assert!(
            output_rms < 0.0001,
            "Silence input should produce silence output"
        );
    }

    #[test]
    fn test_filter_bank_latency() {
        let mut bank = FilterBank::new_with_sample_rate(44100.0);
        let mut input = vec![0.0; 100];
        input[0] = 1.0;
        let output: Vec<f32> = input.iter().map(|&s| bank.process(s)).collect();
        let max_idx = output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        assert!(max_idx <= 1, "Filter bank should have near-zero latency");
    }

    // ============ 边界测试 ============

    #[test]
    fn test_filter_bank_creation() {
        let bank = FilterBank::new();
        assert_eq!(bank.config.filters.len(), 8);
    }

    #[test]
    fn test_filter_bank_set_filter() {
        let mut bank = FilterBank::new();
        let config = FilterBandConfig {
            filter_type: FilterBankType::HighShelf,
            frequency: 8000.0,
            gain_db: 6.0,
            q: 2.0,
        };
        bank.set_filter(0, config);
        assert_eq!(
            bank.config.filters[0].filter_type,
            FilterBankType::HighShelf
        );
    }

    #[test]
    fn test_filter_bank_invalid_index() {
        let mut bank = FilterBank::new();
        let config = FilterBandConfig::default();
        bank.set_filter(10, config);
        bank.set_filter(255, config);
        let output = bank.process(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_filter_bank_process() {
        let mut bank = FilterBank::new_with_sample_rate(44100.0);
        for _ in 0..1000 {
            let output = bank.process(0.5);
            assert!(output.is_finite(), "Output should be finite");
            // 滤波器可能产生较大增益，但应该在合理范围内
            assert!(
                output.abs() <= 500.0,
                "Output should be bounded, got {}",
                output
            );
        }
    }

    #[test]
    fn test_filter_bank_stereo() {
        let mut bank = FilterBank::new_with_sample_rate(44100.0);
        for _ in 0..1000 {
            let (left, right) = bank.process_stereo(0.5, 0.3);
            assert!(left.is_finite(), "Left output should be finite");
            assert!(right.is_finite(), "Right output should be finite");
        }
    }

    #[test]
    fn test_filter_bank_reset() {
        let mut bank = FilterBank::new_with_sample_rate(44100.0);
        for _ in 0..100 {
            bank.process(0.5);
        }
        bank.reset();
        let output = bank.process(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_filter_bank_all_types() {
        let freqs = [31.5, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0];
        for (i, &freq) in freqs.iter().enumerate() {
            for filter_type in [
                FilterBankType::LowShelf,
                FilterBankType::Peak,
                FilterBankType::HighShelf,
            ] {
                let mut bank = FilterBank::new_with_sample_rate(44100.0);
                let config = FilterBandConfig {
                    filter_type,
                    frequency: freq,
                    gain_db: 3.0,
                    q: 1.0,
                };
                bank.set_filter(i, config);
                let output = bank.process(0.5);
                assert!(output.is_finite());
            }
        }
    }

    #[test]
    fn test_filter_bank_gain_bounds() {
        let mut bank = FilterBank::new_with_sample_rate(44100.0);
        let mut config = FilterBandConfig::default();

        // 测试增益被正确限制
        config.gain_db = 100.0;
        bank.set_filter(0, config);
        assert!(
            bank.config.filters[0].gain_db <= 12.0,
            "Gain should be clamped to 12.0"
        );

        config.gain_db = -100.0;
        bank.set_filter(0, config);
        assert!(
            bank.config.filters[0].gain_db >= -12.0,
            "Gain should be clamped to -12.0"
        );
    }
}
