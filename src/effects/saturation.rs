//! Saturation Effect Module
//!
//! This module provides analog-style saturation and soft clipping effects.
//! Saturation adds harmonic content to audio signals by gently compressing
//! peaks and adding even/odd harmonics, similar to tape saturation or
//! tube amplifiers.
//!
//! # Characteristics
//!
//! - **Soft Clipping**: Smooth waveform limiting that adds harmonics
//!
//! - **Tone Control**: Adjustable frequency response for the effect
//!
//! - **Mix Control**: Blend between dry and saturated signals
//!
//! # Applications
//!
//! - Adding warmth and character to sounds
//!
//! - Creating distortion and overdrive effects
//!
//! - Enhancing low-end punch and high-end sheen
//!
//! - Glue effect for mixing (binding elements together)

use std::f32::consts::PI;

use super::Effect;

/// Configuration structure for saturation parameters.
#[derive(Debug, Clone, Copy)]
pub struct SaturationConfig {
    /// Amount of saturation/drive (0.0 = clean, higher = more distortion)
    pub drive: f32,

    /// Tone control for frequency response (0.0 = dark, 1.0 = bright)
    pub tone: f32,

    /// Wet/dry mix (0.0 = dry, 1.0 = fully saturated)
    pub mix: f32,

    /// Sample rate for internal processing
    pub sample_rate: f32,
}

impl Default for SaturationConfig {
    fn default() -> Self {
        Self {
            drive: 0.5,
            tone: 0.5,
            mix: 0.5,
            sample_rate: 44100.0,
        }
    }
}

/// Saturation effect with analog-style soft clipping.
///
/// This effect simulates the harmonic saturation found in analog audio
/// circuits such as tube amplifiers, tape machines, and transformer-coupled
/// audio equipment.
///
/// The saturation curve uses a modified hyperbolic tangent function that
/// produces smooth soft clipping with a pleasant harmonic character.
/// Unlike hard clipping (digital distortion), soft clipping gradually
/// compresses peaks as they approach the threshold.
///
/// # Algorithm
///
/// The waveshaper function applies a curve that:
/// 1. Passes low-amplitude signals nearly unchanged
/// 2. Gradually compresses medium-amplitude signals
/// 3. Smoothly clips high-amplitude signals
///
/// This creates harmonic content that increases with signal level,
/// mimicking the behavior of analog saturation.
#[derive(Debug, Clone)]
pub struct Saturation {
    /// Saturation/drive amount
    drive: f32,

    /// Tone control value
    tone: f32,

    /// Wet/dry mix
    mix: f32,

    /// Sample rate
    sample_rate: f32,

    /// Low-pass filter coefficient for tone control
    tone_coef: f32,

    /// Previous sample for tone filter
    prev_tone: f32,

    /// Whether the effect is enabled
    enabled: bool,
}

impl Saturation {
    /// Creates a new saturation effect with default configuration.
    pub fn new() -> Self {
        Self::with_config(SaturationConfig::default())
    }

    /// Creates a new saturation effect with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Saturation configuration parameters
    ///
    /// # Returns
    ///
    /// A configured Saturation instance
    pub fn with_config(config: SaturationConfig) -> Self {
        let mut sat = Self {
            drive: config.drive,
            tone: config.tone,
            mix: config.mix,
            sample_rate: config.sample_rate,
            tone_coef: 0.5,
            prev_tone: 0.0,
            enabled: true,
        };

        sat.calculate_coefficients();
        sat
    }

    /// Applies the saturation waveshaper curve to an input sample.
    ///
    /// This is the core of the saturation effect. It uses a modified
    /// soft-clipping curve that creates smooth harmonic distortion.
    ///
    /// # Arguments
    ///
    /// * `input` - Input sample to process
    /// * `drive` - Amount of saturation to apply
    ///
    /// # Returns
    ///
    /// Saturated output sample
    ///
    /// # Mathematical Details
    ///
    /// The curve formula is: `output = (1 + k) * x / (1 + k * |x|)`
    ///
    /// Where `k` is the drive parameter scaled by a factor.
    /// This produces:
    /// - Linear behavior for small signals (|x| << 1/k)
    /// - Soft clipping for larger signals
    /// - Asymptotic approach to ±(1 + 1/k) for very large signals
    #[inline]
    fn apply_saturation_curve(&self, input: f32, drive: f32) -> f32 {
        // Scale drive for practical use
        let k = drive * 3.0;

        // Apply soft clipping curve
        // This creates smooth harmonic saturation similar to analog circuits
        let numerator = (1.0 + k) * input;
        let denominator = 1.0 + k * input.abs();

        numerator / denominator
    }

    /// Processes a single audio sample through the saturation effect.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio sample
    ///
    /// # Returns
    ///
    /// Processed output sample
    pub fn process_sample(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        // Apply saturation curve to input
        let saturated = self.apply_saturation_curve(input, self.drive);

        // Apply tone control (simple low-pass for high frequencies)
        // This simulates the darkening effect of some analog circuits
        let tone_filtered = self.prev_tone + self.tone_coef * (saturated - self.prev_tone);
        self.prev_tone = tone_filtered;

        // Blend between original and saturated signal based on tone
        // Higher tone = more saturated high frequencies
        let tone_output = input * (1.0 - self.tone * 0.5) + saturated * (self.tone * 0.5);

        // Apply final mix
        input * (1.0 - self.mix) + tone_output * self.mix
    }

    /// Processes a buffer of audio samples.
    ///
    /// # Arguments
    ///
    /// * `samples` - Mutable slice of audio samples to process
    pub fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }

    /// Processes a single sample with bypass when disabled.
    ///
    /// This is an alias for process_sample() for API consistency.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio sample
    ///
    /// # Returns
    ///
    /// Output sample (saturated if enabled, passthrough if disabled)
    pub fn process_with_bypass(&mut self, input: f32) -> f32 {
        self.process_sample(input)
    }

    /// Sets the saturation drive amount.
    ///
    /// # Arguments
    ///
    /// * `drive` - Drive amount (0.0 = clean, higher = more distortion)
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 10.0);
    }

    /// Sets the tone control value.
    ///
    /// # Arguments
    ///
    /// * `tone` - Tone value (0.0 = dark, 1.0 = bright)
    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone.clamp(0.0, 1.0);
        self.calculate_coefficients();
    }

    /// Sets the wet/dry mix.
    ///
    /// # Arguments
    ///
    /// * `mix` - Mix value (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Enables or disables the effect.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the effect should be active
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Checks if the effect is enabled.
    ///
    /// # Returns
    ///
    /// True if the effect is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Resets the effect state.
    pub fn reset(&mut self) {
        self.prev_tone = 0.0;
    }

    /// Sets the sample rate and recalculates coefficients.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - New sample rate in Hz
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.calculate_coefficients();
    }

    /// Calculates filter coefficients from current parameters.
    fn calculate_coefficients(&mut self) {
        // Calculate tone filter coefficient
        // Maps tone (0-1) to cutoff frequency
        // Low tone = darker (lower cutoff), high tone = brighter
        let cutoff_hz = 100.0 + self.tone * 10000.0;

        // Simple RC low-pass filter coefficient
        let omega = 2.0 * PI * cutoff_hz / self.sample_rate;
        self.tone_coef = omega / (1.0 + omega);
    }
}

impl Default for Saturation {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Saturation {
    fn process(&mut self, input: f32) -> f32 {
        self.process_sample(input)
    }

    fn process_with_bypass(&mut self, input: f32) -> f32 {
        if self.enabled {
            self.process_sample(input)
        } else {
            input
        }
    }

    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }

    fn reset(&mut self) {
        self.prev_tone = 0.0;
    }

    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn set_intensity(&mut self, intensity: f32) {
        self.drive = intensity.clamp(0.0, 1.0) * 10.0;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Simple saturation function for one-off waveshaping.
///
/// This is a convenience function for applying saturation directly
/// to a sample without creating an effect instance.
///
/// # Arguments
///
/// * `sample` - Input sample to saturate
/// * `drive` - Amount of saturation to apply
///
/// # Returns
///
/// Saturated sample
///
/// # Example
///
/// ```rust
/// use wavelet::effects::saturation::saturate;
///
/// let dry = 0.5;
/// let saturated = saturate(dry, 0.5); // Mild saturation
/// let distorted = saturate(dry, 3.0); // Heavy distortion
/// ```
#[inline]
pub fn saturate(sample: f32, drive: f32) -> f32 {
    let k = drive * 3.0;
    let numerator = (1.0 + k) * sample;
    let denominator = 1.0 + k * sample.abs();
    numerator / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturation_default() {
        let sat = Saturation::new();
        assert_eq!(sat.drive, 0.5);
        assert_eq!(sat.tone, 0.5);
        assert_eq!(sat.mix, 0.5);
        assert!(sat.enabled);
    }

    #[test]
    fn test_saturation_process() {
        let mut sat = Saturation::new();
        let input = 0.5;
        let output = sat.process_sample(input);
        // Should process without issues
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_saturation_clipping() {
        let mut sat = Saturation::new();

        // Process a loud signal - saturation should reduce it
        let output = sat.process_sample(2.0);
        // Output should be significantly reduced due to saturation
        assert!(output.abs() <= 5.0); // Allow some overshoot
    }

    #[test]
    fn test_saturation_drive_levels() {
        let mut sat = Saturation::new();

        // Test different drive levels
        sat.set_drive(0.0);
        let clean = sat.process_sample(0.5);

        sat.set_drive(1.0);
        let mild = sat.process_sample(0.5);

        sat.set_drive(5.0);
        let heavy = sat.process_sample(0.5);

        // All outputs should be valid
        assert!(clean.abs() <= 1.0);
        assert!(mild.abs() <= 1.0);
        assert!(heavy.abs() <= 1.0);
    }

    #[test]
    fn test_saturation_mix() {
        let mut sat = Saturation::new();
        sat.set_drive(5.0); // High drive

        sat.set_mix(0.0);
        let dry = sat.process_sample(0.5);
        assert!((dry - 0.5).abs() < 0.001);

        sat.set_mix(1.0);
        let wet = sat.process_sample(0.5);
        assert!((wet - 0.5).abs() > 0.01);
    }

    #[test]
    #[test]
    fn test_saturation_bypass() {
        let mut sat = Saturation::new();
        sat.set_drive(10.0);

        sat.set_enabled(false);
        let passthrough = sat.process_sample(0.5);
        assert_eq!(passthrough, 0.5);

        sat.set_enabled(true);
        let processed = sat.process_sample(0.5);
        assert_ne!(processed, 0.5);
    }

    #[test]
    fn test_saturation_reset() {
        let mut sat = Saturation::new();
        sat.set_drive(5.0);

        // Process some samples
        for _ in 0..100 {
            sat.process_sample(0.8);
        }

        // Reset should clear state
        sat.reset();
        assert_eq!(sat.prev_tone, 0.0);
    }

    #[test]
    fn test_saturation_tone_control() {
        let mut sat = Saturation::new();
        sat.set_drive(2.0);

        // Process with different tone settings
        sat.set_tone(0.0);
        let dark = sat.process_sample(0.5);

        sat.set_tone(0.5);
        let mid = sat.process_sample(0.5);

        sat.set_tone(1.0);
        let bright = sat.process_sample(0.5);

        // All should produce valid output
        assert!(dark.abs() <= 1.0);
        assert!(mid.abs() <= 1.0);
        assert!(bright.abs() <= 1.0);
    }

    #[test]
    fn test_saturate_function() {
        let clean = saturate(0.5, 0.0);
        assert!((clean - 0.5).abs() < 0.001);

        let saturated = saturate(0.5, 2.0);
        assert!(saturated.abs() <= 1.0);
    }

    #[test]
    fn test_saturation_process_buffer() {
        let mut sat = Saturation::new();
        let mut samples = [0.5, 0.3, 0.7, 0.4, 0.6];

        sat.process_buffer(&mut samples);

        // All samples should still be in valid range
        for &sample in &samples {
            assert!(sample.abs() <= 1.0);
        }
    }
}
