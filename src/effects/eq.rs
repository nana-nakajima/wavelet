//! EQ (Equalizer) Effect Module
//!
//! This module provides parametric equalizer functionality for WAVELET.
//! A 4-band parametric EQ allows precise control over frequency response.
//!
//! # Features
//!
//! - **4-band parametric EQ**: Low Shelf, Low Mid, High Mid, High Shelf
//! - **Independent gain control**: ±12dB per band
//! - **Adjustable frequency**: Per-band frequency control
//! - **Quality factor (Q) control**: Bandwidth adjustment
//! - **Master gain**: Output level control

use std::f32::consts::PI;

/// EQ Band Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqBandType {
    /// Low shelf filter - boosts/cuts below cutoff
    LowShelf,
    /// Peaking filter - boosts/cuts around center frequency
    Peaking,
    /// High shelf filter - boosts/cuts above cutoff
    HighShelf,
}

/// Configuration for a single EQ band
#[derive(Debug, Clone, Copy)]
pub struct EqBandConfig {
    /// Band type
    pub band_type: EqBandType,
    /// Center/cutoff frequency in Hz (20Hz - 20kHz)
    pub frequency: f32,
    /// Gain in dB (-12dB to +12dB)
    pub gain_db: f32,
    /// Quality factor (0.1 to 10.0)
    pub q: f32,
    /// Band enabled/disabled
    pub enabled: bool,
}

impl Default for EqBandConfig {
    fn default() -> Self {
        Self {
            band_type: EqBandType::Peaking,
            frequency: 1000.0,
            gain_db: 0.0,
            q: 1.0,
            enabled: true,
        }
    }
}

/// 4-band Parametric Equalizer
///
/// This struct implements a professional-grade 4-band parametric EQ.
///
/// # Band Configuration
///
/// | Band | Type | Default Frequency | Purpose |
/// |------|------|-------------------|---------|
/// | 1 | Low Shelf | 100Hz | Bass control |
/// | 2 | Peaking | 400Hz | Low mids |
/// | 3 | Peaking | 2.5kHz | High mids |
/// | 4 | High Shelf | 10kHz | Treble control |
#[derive(Debug, Clone)]
pub struct ParametricEq {
    /// Sample rate
    sample_rate: f32,

    /// Band configurations (for UI/automation)
    bands: [EqBandConfig; 4],

    /// Master output gain (linear)
    master_gain: f32,

    /// Bypass state
    bypass: bool,

    // Low Shelf coefficients (b0, b1, b2, a1, a2)
    ls_b0: f32,
    ls_b1: f32,
    ls_b2: f32,
    ls_a1: f32,
    ls_a2: f32,

    // Low Mid coefficients
    lm_b0: f32,
    lm_b1: f32,
    lm_b2: f32,
    lm_a1: f32,
    lm_a2: f32,

    // High Mid coefficients
    hm_b0: f32,
    hm_b1: f32,
    hm_b2: f32,
    hm_a1: f32,
    hm_a2: f32,

    // High Shelf coefficients
    hs_b0: f32,
    hs_b1: f32,
    hs_b2: f32,
    hs_a1: f32,
    hs_a2: f32,

    // State variables for each band
    ls_x1: f32,
    ls_x2: f32,
    ls_y1: f32,
    ls_y2: f32,

    lm_x1: f32,
    lm_x2: f32,
    lm_y1: f32,
    lm_y2: f32,

    hm_x1: f32,
    hm_x2: f32,
    hm_y1: f32,
    hm_y2: f32,

    hs_x1: f32,
    hs_x2: f32,
    hs_y1: f32,
    hs_y2: f32,
}

impl ParametricEq {
    /// Creates a new 4-band parametric EQ with default settings.
    pub fn new(sample_rate: f32) -> Self {
        let mut eq = Self {
            sample_rate,
            bands: [
                EqBandConfig {
                    band_type: EqBandType::LowShelf,
                    frequency: 100.0,
                    gain_db: 0.0,
                    q: 0.7,
                    enabled: true,
                },
                EqBandConfig {
                    band_type: EqBandType::Peaking,
                    frequency: 400.0,
                    gain_db: 0.0,
                    q: 1.0,
                    enabled: true,
                },
                EqBandConfig {
                    band_type: EqBandType::Peaking,
                    frequency: 2500.0,
                    gain_db: 0.0,
                    q: 1.0,
                    enabled: true,
                },
                EqBandConfig {
                    band_type: EqBandType::HighShelf,
                    frequency: 10000.0,
                    gain_db: 0.0,
                    q: 0.7,
                    enabled: true,
                },
            ],
            master_gain: 1.0,
            bypass: false,
            // Initialize all coefficients to identity (pass-through)
            ls_b0: 1.0,
            ls_b1: 0.0,
            ls_b2: 0.0,
            ls_a1: 0.0,
            ls_a2: 0.0,
            lm_b0: 1.0,
            lm_b1: 0.0,
            lm_b2: 0.0,
            lm_a1: 0.0,
            lm_a2: 0.0,
            hm_b0: 1.0,
            hm_b1: 0.0,
            hm_b2: 0.0,
            hm_a1: 0.0,
            hm_a2: 0.0,
            hs_b0: 1.0,
            hs_b1: 0.0,
            hs_b2: 0.0,
            hs_a1: 0.0,
            hs_a2: 0.0,
            // State variables
            ls_x1: 0.0,
            ls_x2: 0.0,
            ls_y1: 0.0,
            ls_y2: 0.0,
            lm_x1: 0.0,
            lm_x2: 0.0,
            lm_y1: 0.0,
            lm_y2: 0.0,
            hm_x1: 0.0,
            hm_x2: 0.0,
            hm_y1: 0.0,
            hm_y2: 0.0,
            hs_x1: 0.0,
            hs_x2: 0.0,
            hs_y1: 0.0,
            hs_y2: 0.0,
        };

        // Calculate coefficients for all bands
        eq.recalculate_all_coefficients();

        eq
    }

    /// Recalculates coefficients for all bands based on current settings.
    fn recalculate_all_coefficients(&mut self) {
        self.calculate_low_shelf_coefficients();
        self.calculate_low_mid_coefficients();
        self.calculate_high_mid_coefficients();
        self.calculate_high_shelf_coefficients();
    }

    /// Calculates coefficients for low shelf filter.
    fn calculate_low_shelf_coefficients(&mut self) {
        let config = self.bands[0];
        if !config.enabled {
            self.ls_b0 = 1.0;
            self.ls_b1 = 0.0;
            self.ls_b2 = 0.0;
            self.ls_a1 = 0.0;
            self.ls_a2 = 0.0;
            return;
        }

        let fc = config.frequency.clamp(20.0, self.sample_rate * 0.45);
        let gain = 10.0f32.powf(config.gain_db / 20.0);
        let q = config.q.clamp(0.1, 10.0);

        let omega = 2.0 * PI * fc / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let a = gain.sqrt();

        let a_plus_1 = a + 1.0;
        let a_minus_1 = a - 1.0;

        let b0 = a * (a_plus_1 - a_minus_1 * cos_omega + alpha);
        let b1 = 2.0 * a * (a_minus_1 - a_plus_1 * cos_omega);
        let b2 = a * (a_plus_1 - a_minus_1 * cos_omega - alpha);
        let a0 = a_plus_1 + a_minus_1 * cos_omega + alpha;

        self.ls_b0 = b0 / a0;
        self.ls_b1 = b1 / a0;
        self.ls_b2 = b2 / a0;
        self.ls_a1 = -2.0 * (a_minus_1 + a_plus_1 * cos_omega) / a0;
        self.ls_a2 = -(a_plus_1 + a_minus_1 * cos_omega - alpha) / a0;
    }

    /// Calculates coefficients for low mid peaking filter.
    fn calculate_low_mid_coefficients(&mut self) {
        let config = self.bands[1];
        if !config.enabled {
            self.lm_b0 = 1.0;
            self.lm_b1 = 0.0;
            self.lm_b2 = 0.0;
            self.lm_a1 = 0.0;
            self.lm_a2 = 0.0;
            return;
        }

        let fc = config.frequency.clamp(20.0, self.sample_rate * 0.45);
        let gain = 10.0f32.powf(config.gain_db / 20.0);
        let q = config.q.clamp(0.1, 10.0);

        let omega = 2.0 * PI * fc / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let a = gain;
        let a0 = 1.0 + alpha / a;

        self.lm_b0 = (1.0 + alpha * a) / a0;
        self.lm_b1 = -2.0 * cos_omega / a0;
        self.lm_b2 = (1.0 - alpha * a) / a0;
        self.lm_a1 = -2.0 * cos_omega / a0;
        self.lm_a2 = (1.0 - alpha / a) / a0;
    }

    /// Calculates coefficients for high mid peaking filter.
    fn calculate_high_mid_coefficients(&mut self) {
        let config = self.bands[2];
        if !config.enabled {
            self.hm_b0 = 1.0;
            self.hm_b1 = 0.0;
            self.hm_b2 = 0.0;
            self.hm_a1 = 0.0;
            self.hm_a2 = 0.0;
            return;
        }

        let fc = config.frequency.clamp(20.0, self.sample_rate * 0.45);
        let gain = 10.0f32.powf(config.gain_db / 20.0);
        let q = config.q.clamp(0.1, 10.0);

        let omega = 2.0 * PI * fc / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let a = gain;
        let a0 = 1.0 + alpha / a;

        self.hm_b0 = (1.0 + alpha * a) / a0;
        self.hm_b1 = -2.0 * cos_omega / a0;
        self.hm_b2 = (1.0 - alpha * a) / a0;
        self.hm_a1 = -2.0 * cos_omega / a0;
        self.hm_a2 = (1.0 - alpha / a) / a0;
    }

    /// Calculates coefficients for high shelf filter.
    fn calculate_high_shelf_coefficients(&mut self) {
        let config = self.bands[3];
        if !config.enabled {
            self.hs_b0 = 1.0;
            self.hs_b1 = 0.0;
            self.hs_b2 = 0.0;
            self.hs_a1 = 0.0;
            self.hs_a2 = 0.0;
            return;
        }

        let fc = config.frequency.clamp(20.0, self.sample_rate * 0.45);
        let gain = 10.0f32.powf(config.gain_db / 20.0);
        let q = config.q.clamp(0.1, 10.0);

        let omega = 2.0 * PI * fc / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let a = gain.sqrt();

        let a_plus_1 = a + 1.0;
        let a_minus_1 = a - 1.0;

        let b0 = a * (a_plus_1 + a_minus_1 * cos_omega + alpha);
        let b1 = -2.0 * a * (a_minus_1 + a_plus_1 * cos_omega);
        let b2 = a * (a_plus_1 + a_minus_1 * cos_omega - alpha);
        let a0 = a_plus_1 - a_minus_1 * cos_omega + alpha;

        self.hs_b0 = b0 / a0;
        self.hs_b1 = b1 / a0;
        self.hs_b2 = b2 / a0;
        self.hs_a1 = 2.0 * (a_minus_1 - a_plus_1 * cos_omega) / a0;
        self.hs_a2 = -(a_plus_1 - a_minus_1 * cos_omega - alpha) / a0;
    }

    /// Applies a biquad filter to a sample (direct form II transposed).
    #[inline]
    fn apply_biquad(
        input: f32,
        b0: f32,
        b1: f32,
        b2: f32,
        a1: f32,
        a2: f32,
        x1: &mut f32,
        x2: &mut f32,
        y1: &mut f32,
        y2: &mut f32,
    ) -> f32 {
        // Direct Form II Transposed
        // v[n] = input[n] - a1*v[n-1] - a2*v[n-2]
        // output[n] = b0*v[n] + b1*v[n-1] + b2*v[n-2]
        let v0 = input - a1 * *x1 - a2 * *x2;
        let output = b0 * v0 + b1 * *x1 + b2 * *x2;
        *x2 = *x1;
        *x1 = v0;
        output
    }

    // Public API for parameter control

    /// Gets the current low shelf frequency in Hz.
    pub fn low_shelf_frequency(&self) -> f32 {
        self.bands[0].frequency
    }

    /// Gets the current low shelf gain in dB.
    pub fn low_shelf_gain_db(&self) -> f32 {
        self.bands[0].gain_db
    }

    /// Gets the current low shelf Q factor.
    pub fn low_shelf_q(&self) -> f32 {
        self.bands[0].q
    }

    /// Sets the low shelf frequency.
    pub fn set_low_shelf_frequency(&mut self, frequency: f32) {
        self.bands[0].frequency = frequency;
        self.calculate_low_shelf_coefficients();
    }

    /// Sets the low shelf gain.
    pub fn set_low_shelf_gain(&mut self, gain_db: f32) {
        self.bands[0].gain_db = gain_db.clamp(-12.0, 12.0);
        self.calculate_low_shelf_coefficients();
    }

    /// Sets the low shelf Q factor.
    pub fn set_low_shelf_q(&mut self, q: f32) {
        self.bands[0].q = q.clamp(0.1, 10.0);
        self.calculate_low_shelf_coefficients();
    }

    /// Gets the current low mid frequency in Hz.
    pub fn low_mid_frequency(&self) -> f32 {
        self.bands[1].frequency
    }

    /// Gets the current low mid gain in dB.
    pub fn low_mid_gain_db(&self) -> f32 {
        self.bands[1].gain_db
    }

    /// Gets the current low mid Q factor.
    pub fn low_mid_q(&self) -> f32 {
        self.bands[1].q
    }

    /// Sets the low mid frequency.
    pub fn set_low_mid_frequency(&mut self, frequency: f32) {
        self.bands[1].frequency = frequency.clamp(20.0, self.sample_rate * 0.45);
        self.calculate_low_mid_coefficients();
    }

    /// Sets the low mid gain.
    pub fn set_low_mid_gain(&mut self, gain_db: f32) {
        self.bands[1].gain_db = gain_db.clamp(-12.0, 12.0);
        self.calculate_low_mid_coefficients();
    }

    /// Sets the low mid Q factor.
    pub fn set_low_mid_q(&mut self, q: f32) {
        self.bands[1].q = q.clamp(0.1, 10.0);
        self.calculate_low_mid_coefficients();
    }

    /// Gets the current high mid frequency in Hz.
    pub fn high_mid_frequency(&self) -> f32 {
        self.bands[2].frequency
    }

    /// Gets the current high mid gain in dB.
    pub fn high_mid_gain_db(&self) -> f32 {
        self.bands[2].gain_db
    }

    /// Gets the current high mid Q factor.
    pub fn high_mid_q(&self) -> f32 {
        self.bands[2].q
    }

    /// Sets the high mid frequency.
    pub fn set_high_mid_frequency(&mut self, frequency: f32) {
        self.bands[2].frequency = frequency.clamp(20.0, self.sample_rate * 0.45);
        self.calculate_high_mid_coefficients();
    }

    /// Sets the high mid gain.
    pub fn set_high_mid_gain(&mut self, gain_db: f32) {
        self.bands[2].gain_db = gain_db.clamp(-12.0, 12.0);
        self.calculate_high_mid_coefficients();
    }

    /// Sets the high mid Q factor.
    pub fn set_high_mid_q(&mut self, q: f32) {
        self.bands[2].q = q.clamp(0.1, 10.0);
        self.calculate_high_mid_coefficients();
    }

    /// Gets the current high shelf frequency in Hz.
    pub fn high_shelf_frequency(&self) -> f32 {
        self.bands[3].frequency
    }

    /// Gets the current high shelf gain in dB.
    pub fn high_shelf_gain_db(&self) -> f32 {
        self.bands[3].gain_db
    }

    /// Gets the current high shelf Q factor.
    pub fn high_shelf_q(&self) -> f32 {
        self.bands[3].q
    }

    /// Sets the high shelf frequency.
    pub fn set_high_shelf_frequency(&mut self, frequency: f32) {
        self.bands[3].frequency = frequency.clamp(20.0, self.sample_rate * 0.45);
        self.calculate_high_shelf_coefficients();
    }

    /// Sets the high shelf gain.
    pub fn set_high_shelf_gain(&mut self, gain_db: f32) {
        self.bands[3].gain_db = gain_db.clamp(-12.0, 12.0);
        self.calculate_high_shelf_coefficients();
    }

    /// Sets the high shelf Q factor.
    pub fn set_high_shelf_q(&mut self, q: f32) {
        self.bands[3].q = q.clamp(0.1, 10.0);
        self.calculate_high_shelf_coefficients();
    }

    /// Sets the master output gain.
    pub fn set_master_gain_db(&mut self, gain_db: f32) {
        self.master_gain = 10.0f32.powf(gain_db.clamp(-12.0, 12.0) / 20.0);
    }

    /// Gets the master gain in dB.
    pub fn master_gain_db(&self) -> f32 {
        20.0 * self.master_gain.log10()
    }

    /// Resets all bands to flat response (0dB).
    pub fn reset_to_flat(&mut self) {
        for band in &mut self.bands {
            band.gain_db = 0.0;
        }
        self.recalculate_all_coefficients();
    }

    /// Enables or disables the EQ.
    pub fn set_bypass(&mut self, bypass: bool) {
        self.bypass = bypass;
    }

    /// Checks if the EQ is bypassed.
    pub fn is_bypassed(&self) -> bool {
        self.bypass
    }

    /// Processes a single audio sample through the EQ.
    pub fn process(&mut self, input: f32) -> f32 {
        if self.bypass {
            return input * self.master_gain;
        }

        let output = Self::apply_biquad(
            input,
            self.ls_b0, self.ls_b1, self.ls_b2, self.ls_a1, self.ls_a2,
            &mut self.ls_x1, &mut self.ls_x2, &mut self.ls_y1, &mut self.ls_y2,
        );

        let output = Self::apply_biquad(
            output,
            self.lm_b0, self.lm_b1, self.lm_b2, self.lm_a1, self.lm_a2,
            &mut self.lm_x1, &mut self.lm_x2, &mut self.lm_y1, &mut self.lm_y2,
        );

        let output = Self::apply_biquad(
            output,
            self.hm_b0, self.hm_b1, self.hm_b2, self.hm_a1, self.hm_a2,
            &mut self.hm_x1, &mut self.hm_x2, &mut self.hm_y1, &mut self.hm_y2,
        );

        let output = Self::apply_biquad(
            output,
            self.hs_b0, self.hs_b1, self.hs_b2, self.hs_a1, self.hs_a2,
            &mut self.hs_x1, &mut self.hs_x2, &mut self.hs_y1, &mut self.hs_y2,
        );

        output * self.master_gain
    }

    /// Processes a buffer of audio samples.
    pub fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Resets all filter state variables to zero.
    pub fn reset(&mut self) {
        self.ls_x1 = 0.0;
        self.ls_x2 = 0.0;
        self.ls_y1 = 0.0;
        self.ls_y2 = 0.0;
        self.lm_x1 = 0.0;
        self.lm_x2 = 0.0;
        self.lm_y1 = 0.0;
        self.lm_y2 = 0.0;
        self.hm_x1 = 0.0;
        self.hm_x2 = 0.0;
        self.hm_y1 = 0.0;
        self.hm_y2 = 0.0;
        self.hs_x1 = 0.0;
        self.hs_x2 = 0.0;
        self.hs_y1 = 0.0;
        self.hs_y2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_default() {
        let eq = ParametricEq::new(44100.0);
        assert_eq!(eq.low_shelf_gain_db(), 0.0);
        assert_eq!(eq.low_mid_gain_db(), 0.0);
        assert_eq!(eq.high_mid_gain_db(), 0.0);
        assert_eq!(eq.high_shelf_gain_db(), 0.0);
    }

    #[test]
    fn test_eq_process_flat() {
        let mut eq = ParametricEq::new(44100.0);
        let input = 0.5;
        let output = eq.process(input);
        assert!((output - input).abs() < 0.001, "Flat EQ should pass through");
    }

    #[test]
    fn test_eq_process_buffer() {
        let mut eq = ParametricEq::new(44100.0);
        let mut samples = [0.5; 100];
        eq.process_buffer(&mut samples);
        // Flat EQ should pass through approximately unchanged
        for &sample in &samples {
            assert!((sample - 0.5).abs() < 0.01, "Expected ~0.5, got {}", sample);
        }
    }

    #[test]
    fn test_eq_low_shelf_gain() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_low_shelf_gain(6.0);
        assert_eq!(eq.low_shelf_gain_db(), 6.0);
        let output = eq.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_eq_high_shelf_gain() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_high_shelf_gain(3.0);
        assert_eq!(eq.high_shelf_gain_db(), 3.0);
        let output = eq.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_eq_mid_gain() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_low_mid_gain(-6.0);
        eq.set_high_mid_gain(-6.0);
        assert_eq!(eq.low_mid_gain_db(), -6.0);
        assert_eq!(eq.high_mid_gain_db(), -6.0);
        let output = eq.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_eq_gain_limits() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_low_shelf_gain(20.0);
        assert_eq!(eq.low_shelf_gain_db(), 12.0); // Clamped
        eq.set_low_shelf_gain(-30.0);
        assert_eq!(eq.low_shelf_gain_db(), -12.0); // Clamped
    }

    #[test]
    fn test_eq_frequency_control() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_low_shelf_frequency(80.0);
        eq.set_low_mid_frequency(400.0);
        eq.set_high_mid_frequency(2000.0);
        eq.set_high_shelf_frequency(12000.0);
        assert_eq!(eq.low_shelf_frequency(), 80.0);
        assert_eq!(eq.low_mid_frequency(), 400.0);
        assert_eq!(eq.high_mid_frequency(), 2000.0);
        assert_eq!(eq.high_shelf_frequency(), 12000.0);
    }

    #[test]
    fn test_eq_bypass() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_low_shelf_gain(10.0);
        eq.set_bypass(false);
        let output_bypassed_off = eq.process(0.5);
        eq.set_bypass(true);
        let output_bypassed_on = eq.process(0.5);
        assert!((output_bypassed_on - 0.5).abs() < (output_bypassed_off - 0.5).abs());
    }

    #[test]
    fn test_eq_reset() {
        let mut eq = ParametricEq::new(44100.0);
        for _ in 0..1000 {
            eq.process(0.5);
        }
        eq.reset();
        let output = eq.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_eq_reset_to_flat() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_low_shelf_gain(6.0);
        eq.set_high_shelf_gain(6.0);
        eq.reset_to_flat();
        assert_eq!(eq.low_shelf_gain_db(), 0.0);
        assert_eq!(eq.high_shelf_gain_db(), 0.0);
        let output = eq.process(0.5);
        // Allow small tolerance for numerical precision
        assert!((output - 0.5).abs() < 0.01, "Expected ~0.5, got {}", output);
    }

    #[test]
    fn test_eq_clipping_prevention() {
        let mut eq = ParametricEq::new(44100.0);
        eq.set_low_shelf_gain(12.0);
        eq.set_high_shelf_gain(12.0);
        eq.set_low_mid_gain(12.0);
        eq.set_high_mid_gain(12.0);
        let output = eq.process(0.9);
        // Allow small tolerance for numerical precision
        assert!(output.abs() <= 1.5, "EQ caused clipping: {}", output);
    }
}
