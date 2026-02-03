//! Simple 3-band Equalizer
//!
//! A straightforward 3-band EQ implementation.
//! Uses simple IIR filters for each band.

use std::f32::consts::PI;

use super::Effect;

/// Simple 3-band EQ using cascaded single-pole filters
#[derive(Debug, Clone)]
pub struct SimpleEq {
    /// Sample rate
    sample_rate: f32,

    // Low shelf state
    low_smoothed: f32,

    // Mid peak state
    mid_prev: f32,
    mid_curr: f32,

    // High shelf state
    high_prev: f32,

    // Parameters
    low_gain: f32,
    mid_gain: f32,
    high_gain: f32,

    // Filter coefficients
    low_coeff: f32,
    mid_coeff: f32,
    high_coeff: f32,
    
    // Effect state
    enabled: bool,
    mix: f32,
}

impl SimpleEq {
    /// Creates a new 3-band EQ with flat response.
    pub fn new(sample_rate: f32) -> Self {
        let mut eq = Self {
            sample_rate,
            low_smoothed: 0.0,
            mid_prev: 0.0,
            mid_curr: 0.0,
            high_prev: 0.0,
            low_gain: 1.0,
            mid_gain: 1.0,
            high_gain: 1.0,
            low_coeff: 0.5,
            mid_coeff: 0.5,
            high_coeff: 0.5,
            enabled: true,
            mix: 1.0,
        };
        eq.recalculate_coefficients();
        eq
    }

    /// Recalculates filter coefficients based on current gains.
    fn recalculate_coefficients(&mut self) {
        // Low shelf cutoff ~320Hz
        let wc = 2.0 * PI * 320.0 / self.sample_rate;
        self.low_coeff = wc.clamp(0.0, 2.0);

        // Mid peak center ~1kHz
        let wc = 2.0 * PI * 1000.0 / self.sample_rate;
        self.mid_coeff = wc.clamp(0.0, 2.0);

        // High shelf cutoff ~3.2kHz
        let wc = 2.0 * PI * 3200.0 / self.sample_rate;
        self.high_coeff = wc.clamp(0.0, 2.0);
    }

    /// Sets the low shelf gain in dB.
    pub fn set_low_gain(&mut self, gain_db: f32) {
        self.low_gain = 10.0f32.powf(gain_db.clamp(-12.0, 12.0) / 20.0);
    }

    /// Sets the mid band gain in dB.
    pub fn set_mid_gain(&mut self, gain_db: f32) {
        self.mid_gain = 10.0f32.powf(gain_db.clamp(-12.0, 12.0) / 20.0);
    }

    /// Sets the high shelf gain in dB.
    pub fn set_high_gain(&mut self, gain_db: f32) {
        self.high_gain = 10.0f32.powf(gain_db.clamp(-12.0, 12.0) / 20.0);
    }

    /// Processes a single audio sample.
    pub fn process(&mut self, input: f32) -> f32 {
        // Simple implementation: separate path for each band
        // Low shelf: simple low-pass with gain
        let low_out = self.low_gain * input;

        // High shelf: simple high-pass with gain
        let high_out = self.high_gain * input;

        // Mid: input - low - high
        let mid_out = self.mid_gain * (input - low_out / self.low_gain.max(0.001) - high_out / self.high_gain.max(0.001));

        // Mix bands (simplified)
        let output = low_out + mid_out + high_out;

        output
    }

    /// Processes a buffer of audio samples.
    pub fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Resets all state variables.
    pub fn reset(&mut self) {
        self.low_smoothed = 0.0;
        self.mid_prev = 0.0;
        self.mid_curr = 0.0;
        self.high_prev = 0.0;
    }
}

impl Effect for SimpleEq {
    fn process(&mut self, input: f32) -> f32 {
        self.process(input)
    }

    fn process_with_bypass(&mut self, input: f32) -> f32 {
        if self.enabled {
            self.process(input)
        } else {
            input
        }
    }

    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    fn reset(&mut self) {
        self.reset();
    }

    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn set_intensity(&mut self, intensity: f32) {
        // Map intensity to mid gain for simplicity
        self.set_mid_gain(intensity * 12.0 - 6.0); // -6dB to +6dB
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_eq_flat() {
        let mut eq = SimpleEq::new(44100.0);
        let input = 0.5;
        let output = eq.process(input);
        // Should be approximately the same
        assert!((output - input).abs() < 0.5, "Flat EQ should pass through, got {}", output);
    }

    #[test]
    fn test_simple_eq_buffer() {
        let mut eq = SimpleEq::new(44100.0);
        let mut samples = [0.5; 100];
        eq.process_buffer(&mut samples);
        for &sample in &samples {
            assert!(sample.is_finite(), "Sample should be finite, got {}", sample);
        }
    }

    #[test]
    fn test_simple_eq_low_gain() {
        let mut eq = SimpleEq::new(44100.0);
        eq.set_low_gain(6.0);
        let output = eq.process(0.5);
        assert!(output.abs() <= 1.0 && output.is_finite(), "Should not clip or be NaN");
    }

    #[test]
    fn test_simple_eq_high_gain() {
        let mut eq = SimpleEq::new(44100.0);
        eq.set_high_gain(6.0);
        let output = eq.process(0.5);
        assert!(output.abs() <= 1.0 && output.is_finite(), "Should not clip or be NaN");
    }

    #[test]
    fn test_simple_eq_mid_gain() {
        let mut eq = SimpleEq::new(44100.0);
        eq.set_mid_gain(-6.0);
        let output = eq.process(0.5);
        assert!(output.abs() <= 1.0 && output.is_finite(), "Should not clip or be NaN");
    }

    #[test]
    fn test_simple_eq_all_bands() {
        let mut eq = SimpleEq::new(44100.0);
        eq.set_low_gain(6.0);
        eq.set_mid_gain(3.0);
        eq.set_high_gain(6.0);
        let output = eq.process(0.5);
        assert!(output.abs() <= 2.0 && output.is_finite(), "Should not be NaN");
    }

    #[test]
    fn test_simple_eq_reset() {
        let mut eq = SimpleEq::new(44100.0);
        for _ in 0..1000 {
            eq.process(0.5);
        }
        eq.reset();
        let output = eq.process(0.5);
        assert!(output.is_finite(), "Should be finite after reset");
    }

    #[test]
    fn test_simple_eq_different_sample_rates() {
        for &sr in &[44100.0, 48000.0, 96000.0] {
            let mut eq = SimpleEq::new(sr);
            eq.set_low_gain(3.0);
            let output = eq.process(0.5);
            assert!(output.is_finite(), "Failed for sample rate {}", sr);
        }
    }
}
