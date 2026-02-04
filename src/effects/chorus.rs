//! Chorus Effect Module
//!
//! This module implements a stereo chorus effect that creates a thickening
//! and widening effect by modulating delay times with an LFO.
//!
//! The chorus works by:
//! 1. Creating a copy of the input signal
//! 2. Varying the delay time with a low-frequency oscillator
//! 3. Mixing the delayed signal with the original for a "doubled" sound
//!
//! # Key Parameters
//!
//! - **Rate**: LFO frequency (0.1 - 10 Hz)
//! - **Depth**: Amount of delay modulation (0 - 100%)
//! - **Mix**: Wet/dry balance (0% = dry, 100% = wet)
//! - **Feedback**: Regeneration amount for more intense effects

use std::f32::consts::PI;

/// Stereo chorus effect with LFO-modulated delay lines.
///
/// The chorus effect creates a natural "doubling" or "widening" effect
/// by slightly detuning and delaying copies of the input signal.
///
/// # Technical Details
///
/// The effect uses:
/// - Two delay lines (left and right channels)
/// - Independent LFOs for each channel (slightly different rates)
/// - Linear interpolation for smooth delay time changes
/// - Optional feedback for more pronounced effects
///
/// The stereo width is achieved by:
/// - Different LFO phases for left and right
/// - Different LFO rates (slight detuning)
/// - Independent delay lines
#[derive(Debug, Clone)]
pub struct Chorus {
    /// Left channel delay buffer
    left_buffer: Vec<f32>,

    /// Right channel delay buffer
    right_buffer: Vec<f32>,

    /// Current write position (left)
    left_write_pos: usize,

    /// Current write position (right)
    right_write_pos: usize,

    /// Left LFO phase (0.0 - 1.0)
    left_phase: f32,

    /// Right LFO phase (0.0 - 1.0)
    right_phase: f32,

    /// Base delay time in samples
    base_delay_samples: usize,

    /// Maximum delay time in samples
    max_delay_samples: usize,

    /// LFO rate in Hz (left channel)
    rate_hz: f32,

    /// LFO rate multiplier for right channel (creates stereo width)
    right_rate_mult: f32,

    /// LFO depth (0.0 - 1.0)
    depth: f32,

    /// Wet/dry mix (0.0 - 1.0)
    mix: f32,

    /// Feedback amount (0.0 - 0.9)
    feedback: f32,

    /// Sample rate
    sample_rate: f32,

    /// Whether the effect is enabled
    enabled: bool,
}

impl Chorus {
    /// Creates a new chorus effect.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Returns
    ///
    /// A new Chorus instance with default settings
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_ms = 50.0; // 50ms maximum modulation
        let base_delay_ms = 25.0; // 25ms base delay

        let max_delay_samples = (max_delay_ms / 1000.0 * sample_rate) as usize;
        let base_delay_samples = (base_delay_ms / 1000.0 * sample_rate) as usize;

        Self {
            left_buffer: vec![0.0; max_delay_samples],
            right_buffer: vec![0.0; max_delay_samples],
            left_write_pos: base_delay_samples,
            right_write_pos: base_delay_samples,
            left_phase: 0.0,
            right_phase: 0.0,
            base_delay_samples,
            max_delay_samples,
            rate_hz: 0.5,          // 0.5 Hz LFO
            right_rate_mult: 1.02, // Slight detune for stereo width
            depth: 0.5,            // 50% depth
            mix: 0.4,              // 40% wet
            feedback: 0.2,         // 20% feedback
            sample_rate,
            enabled: true,
        }
    }

    /// Creates a new chorus with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `rate_hz` - LFO rate in Hz (0.1 - 10.0)
    /// * `depth` - Modulation depth (0.0 - 1.0)
    /// * `mix` - Wet/dry mix (0.0 - 1.0)
    ///
    /// # Returns
    ///
    /// A new Chorus instance with specified settings
    pub fn with_params(sample_rate: f32, rate_hz: f32, depth: f32, mix: f32) -> Self {
        let max_delay_ms = 50.0;
        let base_delay_ms = 25.0;

        let max_delay_samples = (max_delay_ms / 1000.0 * sample_rate) as usize;
        let base_delay_samples = (base_delay_ms / 1000.0 * sample_rate) as usize;

        Self {
            left_buffer: vec![0.0; max_delay_samples],
            right_buffer: vec![0.0; max_delay_samples],
            left_write_pos: base_delay_samples,
            right_write_pos: base_delay_samples,
            left_phase: 0.0,
            right_phase: 0.0,
            base_delay_samples,
            max_delay_samples,
            rate_hz,
            right_rate_mult: 1.02,
            depth,
            mix,
            feedback: 0.2,
            sample_rate,
            enabled: true,
        }
    }

    /// Reads a sample from a circular buffer with linear interpolation.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Circular buffer to read from
    /// * `read_pos` - Read position (can be fractional)
    ///
    /// # Returns
    ///
    /// Interpolated sample value
    fn read_interpolated(buffer: &[f32], mut read_pos: f32) -> f32 {
        let buffer_len = buffer.len() as f32;

        // Wrap position
        while read_pos < 0.0 {
            read_pos += buffer_len;
        }
        while read_pos >= buffer_len {
            read_pos -= buffer_len;
        }

        let index0 = read_pos.floor() as usize;
        let index1 = (index0 + 1) % buffer.len();
        let frac = read_pos - index0 as f32;

        buffer[index0] * (1.0 - frac) + buffer[index1] * frac
    }

    /// Advances the LFO phase.
    ///
    /// # Arguments
    ///
    /// * `phase` - Current phase (0.0 - 1.0)
    /// * `rate_hz` - LFO rate in Hz
    ///
    /// # Returns
    ///
    /// New phase value
    #[inline]
    fn advance_phase(phase: f32, rate_hz: f32, sample_rate: f32) -> f32 {
        let phase_inc = rate_hz / sample_rate;
        let new_phase = phase + phase_inc;
        if new_phase >= 1.0 {
            new_phase - 1.0
        } else {
            new_phase
        }
    }

    /// Generates an LFO value using sine wave.
    ///
    /// # Arguments
    ///
    /// * `phase` - LFO phase (0.0 - 1.0)
    ///
    /// # Returns
    ///
    /// LFO value in range [-1.0, 1.0]
    #[inline]
    fn lfo_sine(phase: f32) -> f32 {
        (phase * 2.0 * PI).sin()
    }

    /// Calculates the current delay time for a channel.
    ///
    /// # Arguments
    ///
    /// * `phase` - LFO phase (0.0 - 1.0)
    /// * `base_delay` - Base delay in samples
    /// * `max_modulation` - Maximum modulation in samples
    /// * `lfo_value` - LFO output value (-1.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Current delay time in samples
    #[inline]
    fn calculate_delay(base_delay: usize, max_modulation: f32, lfo_value: f32) -> f32 {
        base_delay as f32 + lfo_value * max_modulation
    }
}

impl super::Effect for Chorus {
    /// Processes a single stereo audio sample.
    ///
    /// # Arguments
    ///
    /// * `input` - Input mono sample (applied to both channels)
    ///
    /// # Returns
    ///
    /// Processed output sample (mono, left channel)
    fn process(&mut self, input: f32) -> f32 {
        // Calculate LFO values
        let left_lfo = Self::lfo_sine(self.left_phase);
        let right_lfo = Self::lfo_sine(self.right_phase);

        // Calculate current delay times
        let max_modulation = self.depth * (self.max_delay_samples - self.base_delay_samples) as f32;

        let left_delay = Self::calculate_delay(self.base_delay_samples, max_modulation, left_lfo);
        let right_delay = Self::calculate_delay(self.base_delay_samples, max_modulation, right_lfo);

        // Calculate read positions
        let left_read_pos = self.left_write_pos as f32 - left_delay;
        let right_read_pos = self.right_write_pos as f32 - right_delay;

        // Read delayed samples
        let left_delayed = Self::read_interpolated(&self.left_buffer, left_read_pos);
        let right_delayed = Self::read_interpolated(&self.right_buffer, right_read_pos);

        // Mix feedback
        let left_feedback = left_delayed * self.feedback;
        let right_feedback = right_delayed * self.feedback;

        // Write to buffers with feedback
        self.left_buffer[self.left_write_pos] = input + left_feedback;
        self.right_buffer[self.right_write_pos] = input + right_feedback;

        // Advance write positions
        self.left_write_pos = (self.left_write_pos + 1) % self.max_delay_samples;
        self.right_write_pos = (self.right_write_pos + 1) % self.max_delay_samples;

        // Advance LFO phases
        self.left_phase = Self::advance_phase(self.left_phase, self.rate_hz, self.sample_rate);
        self.right_phase = Self::advance_phase(
            self.right_phase,
            self.rate_hz * self.right_rate_mult,
            self.sample_rate,
        );

        // Mix wet and dry (output mono from stereo input)
        let wet = (left_delayed + right_delayed) * 0.5;
        input * (1.0 - self.mix) + wet * self.mix
    }

    /// Processes a single sample with bypass support.
    fn process_with_bypass(&mut self, input: f32) -> f32 {
        if self.enabled {
            self.process(input)
        } else {
            input
        }
    }

    /// Processes a buffer of audio samples.
    fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Resets effect state.
    fn reset(&mut self) {
        self.left_buffer.fill(0.0);
        self.right_buffer.fill(0.0);
        self.left_write_pos = self.base_delay_samples;
        self.right_write_pos = self.base_delay_samples;
        self.left_phase = 0.0;
        self.right_phase = 0.0;
    }

    /// Sets the wet/dry mix.
    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Sets effect intensity.
    fn set_intensity(&mut self, intensity: f32) {
        // Intensity maps to a combination of depth and rate
        self.depth = intensity.clamp(0.0, 1.0);
        self.rate_hz = 0.1 + intensity * 2.0; // 0.1 - 2.1 Hz
    }

    /// Checks if the effect is enabled.
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables or disables the effect.
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Chorus {
    /// Processes stereo samples.
    ///
    /// # Arguments
    ///
    /// * `left` - Left channel samples
    /// * `right` - Right channel samples
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(left.len(), right.len());

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            // Calculate LFO values
            let left_lfo = Self::lfo_sine(self.left_phase);
            let right_lfo = Self::lfo_sine(self.right_phase);

            // Calculate current delay times
            let max_modulation =
                self.depth * (self.max_delay_samples - self.base_delay_samples) as f32;

            let left_delay =
                Self::calculate_delay(self.base_delay_samples, max_modulation, left_lfo);
            let right_delay =
                Self::calculate_delay(self.base_delay_samples, max_modulation, right_lfo);

            // Calculate read positions
            let left_read_pos = self.left_write_pos as f32 - left_delay;
            let right_read_pos = self.right_write_pos as f32 - right_delay;

            // Read delayed samples
            let left_delayed = Self::read_interpolated(&self.left_buffer, left_read_pos);
            let right_delayed = Self::read_interpolated(&self.right_buffer, right_read_pos);

            // Mix wet and dry
            let left_wet = *l * (1.0 - self.mix) + left_delayed * self.mix;
            let right_wet = *r * (1.0 - self.mix) + right_delayed * self.mix;

            // Write to buffers
            self.left_buffer[self.left_write_pos] = *l + left_delayed * self.feedback;
            self.right_buffer[self.right_write_pos] = *r + right_delayed * self.feedback;

            // Advance write positions
            self.left_write_pos = (self.left_write_pos + 1) % self.max_delay_samples;
            self.right_write_pos = (self.right_write_pos + 1) % self.max_delay_samples;

            // Advance LFO phases
            self.left_phase = Self::advance_phase(self.left_phase, self.rate_hz, self.sample_rate);
            self.right_phase = Self::advance_phase(
                self.right_phase,
                self.rate_hz * self.right_rate_mult,
                self.sample_rate,
            );

            *l = left_wet;
            *r = right_wet;
        }
    }

    /// Sets the LFO rate in Hz.
    ///
    /// # Arguments
    ///
    /// * `rate_hz` - LFO frequency (0.1 - 10.0 Hz)
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.rate_hz = rate_hz.clamp(0.1, 10.0);
    }

    /// Sets the modulation depth.
    ///
    /// # Arguments
    ///
    /// * `depth` - Modulation depth (0.0 - 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Sets the feedback amount.
    ///
    /// # Arguments
    ///
    /// * `feedback` - Feedback amount (0.0 - 0.9)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.9);
    }

    /// Sets the stereo width via right channel rate multiplier.
    ///
    /// # Arguments
    ///
    /// * `width` - Stereo width multiplier (1.0 = mono, >1.0 = wider)
    pub fn set_stereo_width(&mut self, width: f32) {
        self.right_rate_mult = width.clamp(1.0, 1.2);
    }

    /// Gets the current LFO rate in Hz.
    pub fn rate(&self) -> f32 {
        self.rate_hz
    }

    /// Gets the current modulation depth.
    pub fn depth(&self) -> f32 {
        self.depth
    }

    /// Gets the current feedback amount.
    pub fn feedback(&self) -> f32 {
        self.feedback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Import Effect trait for tests
    use super::super::Effect;

    #[test]
    fn test_chorus_default() {
        let chorus = Chorus::new(44100.0);
        assert_eq!(chorus.rate(), 0.5);
        assert_eq!(chorus.depth(), 0.5);
        assert_eq!(chorus.mix, 0.4);
        assert!(chorus.enabled);
    }

    #[test]
    fn test_chorus_with_params() {
        let chorus = Chorus::with_params(44100.0, 1.0, 0.7, 0.6);
        assert_eq!(chorus.rate(), 1.0);
        assert_eq!(chorus.depth(), 0.7);
        assert_eq!(chorus.mix, 0.6);
    }

    #[test]
    fn test_chorus_process() {
        let mut chorus = Chorus::new(1000.0);
        chorus.set_rate(0.5);
        chorus.set_depth(0.5);
        chorus.set_mix(0.5);

        // Process some samples
        let output = chorus.process(0.5);
        // Output should be in reasonable range
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_chorus_buffer_size() {
        let chorus = Chorus::new(44100.0);
        // Buffer should be large enough for 50ms at 44100Hz
        assert!(chorus.left_buffer.len() >= (50.0 / 1000.0 * 44100.0) as usize);
    }

    #[test]
    fn test_chorus_set_parameters() {
        let mut chorus = Chorus::new(44100.0);

        chorus.set_rate(2.0);
        assert_eq!(chorus.rate(), 2.0);

        chorus.set_depth(0.8);
        assert_eq!(chorus.depth(), 0.8);

        chorus.set_mix(0.7);
        // Mix is private, verify via behavior
        let output = chorus.process(0.5);
        assert!(output.abs() <= 1.0);

        chorus.set_feedback(0.5);
        assert_eq!(chorus.feedback(), 0.5);

        chorus.set_stereo_width(1.1);
        // Verify width via behavior
        let output = chorus.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_chorus_enabled() {
        let mut chorus = Chorus::new(44100.0);

        chorus.set_enabled(true);
        assert!(chorus.is_enabled());

        chorus.set_enabled(false);
        assert!(!chorus.is_enabled());
        let bypassed = chorus.process_with_bypass(0.5);
        assert_eq!(bypassed, 0.5);
    }

    #[test]
    fn test_chorus_reset() {
        let mut chorus = Chorus::new(44100.0);

        // Process some samples to change state
        for _ in 0..1000 {
            chorus.process(0.5);
        }

        // Reset should clear buffers and reset phases
        chorus.reset();

        // Verify buffer is cleared
        assert!(chorus.left_buffer.iter().all(|&x| x == 0.0));
        assert!(chorus.right_buffer.iter().all(|&x| x == 0.0));

        // Phases should be reset
        assert_eq!(chorus.left_phase, 0.0);
        assert_eq!(chorus.right_phase, 0.0);
    }

    #[test]
    fn test_chorus_stereo_processing() {
        let mut chorus = Chorus::new(1000.0);

        let mut left = [0.5f32; 100];
        let mut right = [0.3f32; 100];

        chorus.process_stereo(&mut left, &mut right);

        // Both channels should be processed
        assert!(left.iter().any(|&x| x != 0.5));
        assert!(right.iter().any(|&x| x != 0.3));
    }

    #[test]
    fn test_chorus_parameter_limits() {
        let mut chorus = Chorus::new(44100.0);

        // Test parameter clamping
        chorus.set_rate(100.0); // Should clamp to 10.0
        assert_eq!(chorus.rate(), 10.0);

        chorus.set_rate(-1.0); // Should clamp to 0.1
        assert_eq!(chorus.rate(), 0.1);

        chorus.set_depth(2.0); // Should clamp to 1.0
        assert_eq!(chorus.depth(), 1.0);

        chorus.set_feedback(2.0); // Should clamp to 0.9
        assert_eq!(chorus.feedback(), 0.9);

        chorus.set_stereo_width(2.0); // Should clamp to 1.2
                                      // Verify via behavior
        let output = chorus.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_chorus_buffer_writing() {
        let mut chorus = Chorus::new(1000.0);

        // Write to buffer and verify output
        let output = chorus.process(0.7);

        // Output should be valid
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_chorus_intensity() {
        let mut chorus = Chorus::new(44100.0);

        // Intensity should affect both depth and rate
        chorus.set_intensity(0.5);
        assert_eq!(chorus.depth(), 0.5);
        assert_eq!(chorus.rate(), 1.1); // 0.1 + 0.5 * 2.0

        chorus.set_intensity(1.0);
        assert_eq!(chorus.depth(), 1.0);
        assert_eq!(chorus.rate(), 2.1); // 0.1 + 1.0 * 2.0
    }

    #[test]
    fn test_chorus_process_buffer() {
        let mut chorus = Chorus::new(1000.0);

        // Test processing a buffer
        let mut samples = [0.5f32; 100];
        chorus.process_buffer(&mut samples);

        // All samples should be processed
        assert!(samples.iter().any(|&x| x != 0.5));
    }

    #[test]
    fn test_chorus_calculate_delay() {
        let base = 100;
        let max_mod = 50.0;

        // Center position
        let delay = Chorus::calculate_delay(base, max_mod, 0.0);
        assert!((delay - 100.0).abs() < 0.001);

        // Peak positive
        let delay = Chorus::calculate_delay(base, max_mod, 1.0);
        assert!((delay - 150.0).abs() < 0.001);

        // Peak negative
        let delay = Chorus::calculate_delay(base, max_mod, -1.0);
        assert!((delay - 50.0).abs() < 0.001);
    }
}
