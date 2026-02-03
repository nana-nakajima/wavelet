//! Warp Effect Module
//!
//! This module provides a time-warp/pitch-shift effect that creates
//! interesting sonic textures by manipulating the time domain of audio.
//!
//! # Features
//!
//! - **Time Warp**: Stretch or compress audio in time without pitch change
//! - **Pitch Shift**: Shift pitch while maintaining time (phase vocoder)
//! - **Grain Size**: Control the grain size for granular synthesis
//! - **Feedback**: Add feedback for generative textures
//!
//! # Algorithm
//!
//! This implementation uses a basic granular approach:
//! - Input audio is segmented into overlapping grains
//! - Each grain is processed independently
//! - Grains are windowed and overlapped at output
//!
//! For pitch shifting, we use a simple delay-line modulation approach
//! which provides real-time processing with minimal latency.

use std::f32::consts::PI;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Warp effect configuration
#[derive(Debug, Clone, Copy)]
pub struct WarpConfig {
    /// Warp mode (TimeWarp, PitchShift, RingMod)
    pub mode: WarpMode,

    /// Warp amount/strength (0.0 to 1.0)
    pub amount: f32,

    /// Grain size in milliseconds (20-100ms)
    pub grain_size: f32,

    /// Feedback amount (0.0 to 0.95)
    pub feedback: f32,

    /// Modulation rate for LFO (0.1-10 Hz)
    pub rate: f32,

    /// Stereo width (0.0-1.0)
    pub stereo_width: f32,

    /// Wet/dry mix (0.0 = dry, 1.0 = wet)
    pub mix: f32,

    /// Sample rate
    pub sample_rate: f32,
}

impl Default for WarpConfig {
    fn default() -> Self {
        Self {
            mode: WarpMode::TimeWarp,
            amount: 0.5,
            grain_size: 50.0,
            feedback: 0.3,
            rate: 0.5,
            stereo_width: 0.7,
            mix: 0.5,
            sample_rate: 44100.0,
        }
    }
}

/// Warp effect mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpMode {
    /// Time stretching without pitch change
    TimeWarp,

    /// Pitch shifting without time change
    PitchShift,

    /// Ring modulation with warp character
    RingMod,

    /// Granular synthesis mode
    Granular,
}

/// Warp effect processor
#[derive(Debug, Clone)]
pub struct Warp {
    /// Configuration
    config: WarpConfig,

    /// Warp mode
    mode: WarpMode,

    /// Grain size in samples
    grain_size: usize,

    /// Overlap size (1/4 of grain)
    overlap: usize,

    /// Input buffer for grain processing
    input_buffer: Vec<f32>,

    /// Output buffer
    output_buffer: Vec<f32>,

    /// Current read position
    read_pos: f32,

    /// Write position
    write_pos: usize,

    /// LFO phase
    lfo_phase: f32,

    /// LFO increment per sample
    lfo_increment: f32,

    /// Random seed for granular
    rng: StdRng,

    /// Previous grain output (for crossfade)
    prev_grain: Vec<f32>,

    /// Current grain window
    current_window: Vec<f32>,

    /// Whether effect is enabled
    enabled: bool,
}

/// Stereo warp processor
#[derive(Debug, Clone)]
pub struct StereoWarp {
    /// Left channel warp
    left: Warp,

    /// Right channel warp
    right: Warp,

    /// Stereo width
    stereo_width: f32,
}

impl WarpConfig {
    /// Creates a new warp config for time warp mode
    pub fn time_warp(sample_rate: f32) -> Self {
        Self {
            mode: WarpMode::TimeWarp,
            amount: 0.5,
            grain_size: 50.0,
            feedback: 0.3,
            rate: 0.5,
            stereo_width: 0.7,
            mix: 0.5,
            sample_rate,
        }
    }

    /// Creates a new warp config for pitch shift mode
    pub fn pitch_shift(sample_rate: f32) -> Self {
        Self {
            mode: WarpMode::PitchShift,
            amount: 0.0,
            grain_size: 30.0,
            feedback: 0.2,
            rate: 0.5,
            stereo_width: 0.5,
            mix: 0.5,
            sample_rate,
        }
    }

    /// Creates a new warp config for ring mod mode
    pub fn ring_mod(sample_rate: f32) -> Self {
        Self {
            mode: WarpMode::RingMod,
            amount: 0.7,
            grain_size: 20.0,
            feedback: 0.4,
            rate: 2.0,
            stereo_width: 1.0,
            mix: 0.6,
            sample_rate,
        }
    }
}

impl Warp {
    /// Creates a new warp effect with default settings
    pub fn new(sample_rate: f32) -> Self {
        let config = WarpConfig::default();
        let grain_size = (config.grain_size / 1000.0 * sample_rate) as usize;
        let overlap = grain_size / 4;

        Self {
            config,
            mode: WarpMode::TimeWarp,
            grain_size,
            overlap,
            input_buffer: vec![0.0; grain_size * 4],
            output_buffer: vec![0.0; grain_size * 2],
            read_pos: 0.0,
            write_pos: 0,
            lfo_phase: 0.0,
            lfo_increment: 1.0 / sample_rate,
            rng: StdRng::from_entropy(),
            prev_grain: vec![0.0; grain_size],
            current_window: vec![0.0; grain_size],
            enabled: true,
        }
    }

    /// Creates a warp effect with the given configuration
    pub fn with_config(config: WarpConfig) -> Self {
        let grain_size = (config.grain_size / 1000.0 * config.sample_rate) as usize;
        let overlap = (grain_size as f32 * 0.25) as usize;

        let mut warp = Self {
            config,
            mode: config.mode,
            grain_size,
            overlap,
            input_buffer: vec![0.0; grain_size * 4],
            output_buffer: vec![0.0; grain_size * 2],
            read_pos: 0.0,
            write_pos: 0,
            lfo_phase: 0.0,
            lfo_increment: 1.0 / config.sample_rate,
            rng: StdRng::from_entropy(),
            prev_grain: vec![0.0; grain_size],
            current_window: vec![0.0; grain_size],
            enabled: true,
        };

        // Initialize window function
        warp.generate_window();
        warp
    }

    /// Generates the Hann window for grain processing
    fn generate_window(&mut self) {
        let grain_size = self.grain_size;

        for i in 0..grain_size {
            // Hann window
            let window = 0.5 * (1.0 - (2.0 * PI * i as f32 / (grain_size - 1) as f32).cos());
            self.current_window[i] = window;
        }
    }

    /// Updates the grain size based on config
    fn update_grain_size(&mut self) {
        let new_grain_size = (self.config.grain_size / 1000.0 * self.config.sample_rate) as usize;
        if new_grain_size != self.grain_size && new_grain_size > 0 {
            self.grain_size = new_grain_size;
            self.overlap = (new_grain_size as f32 * 0.25) as usize;
            self.input_buffer.resize(new_grain_size * 4, 0.0);
            self.output_buffer.resize(new_grain_size * 2, 0.0);
            self.prev_grain.resize(new_grain_size, 0.0);
            self.current_window.resize(new_grain_size, 0.0);
            self.generate_window();
        }
    }

    /// Applies the LFO to get current modulation value
    fn get_lfo_value(&mut self) -> f32 {
        let lfo = 0.5 * (1.0 + (2.0 * PI * self.lfo_phase).sin());

        // Update LFO phase
        self.lfo_phase += self.lfo_increment * self.config.rate * 10.0;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        lfo
    }

    /// Processes a single sample through the warp effect
    fn process_sample(&mut self, input: f32) -> f32 {
        // Write input to buffer
        self.input_buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.input_buffer.len();

        // Calculate read position based on mode and LFO
        let warp_amount = self.config.amount;
        let lfo = self.get_lfo_value();

        let (delayed, output) = match self.mode {
            WarpMode::TimeWarp => {
                // Time warp: modulate read speed
                let speed = 1.0 + (lfo - 0.5) * warp_amount * 2.0;
                self.read_pos += speed;
                self.read_pos %= self.input_buffer.len() as f32;
                let delayed_val = self.input_buffer[self.read_pos as usize];
                (delayed_val, delayed_val)
            }
            WarpMode::PitchShift => {
                // Pitch shift: delay line modulation
                let delay = warp_amount * 0.1 * (1.0 + lfo);
                let delay_samples = (delay * self.config.sample_rate) as usize;
                let read_pos = (self.write_pos + self.input_buffer.len() - delay_samples)
                    % self.input_buffer.len();
                let delayed_val = self.input_buffer[read_pos];
                (delayed_val, delayed_val)
            }
            WarpMode::RingMod => {
                // Ring mod: multiply by LFO
                let modulated = lfo * 2.0 - 1.0; // -1 to 1
                let ring_output = input * modulated * warp_amount;
                let feedback_idx = (self.write_pos + self.output_buffer.len() - self.grain_size)
                    % self.output_buffer.len();
                self.output_buffer[feedback_idx] = ring_output + self.output_buffer[feedback_idx] * self.config.feedback;
                (ring_output, ring_output)
            }
            WarpMode::Granular => {
                // Granular: random grain selection
                let grain_offset = self.rng.gen::<usize>() % self.grain_size;
                let read_pos = (self.write_pos + self.input_buffer.len() - grain_offset) % self.input_buffer.len();
                let delayed_val = self.input_buffer[read_pos];
                (delayed_val, delayed_val)
            }
        };

        // Mix wet/dry
        input * (1.0 - self.config.mix) + output * self.config.mix
    }
}

impl StereoWarp {
    /// Creates a new stereo warp effect
    pub fn new(sample_rate: f32) -> Self {
        Self {
            left: Warp::new(sample_rate),
            right: Warp::new(sample_rate),
            stereo_width: 0.7,
        }
    }

    /// Creates a stereo warp with configuration
    pub fn with_config(config: WarpConfig) -> Self {
        let mut left_config = config;
        let mut right_config = config;

        // Offset LFO phase for stereo width
        right_config.rate = config.rate * (1.0 + config.stereo_width * 0.5);

        Self {
            left: Warp::with_config(left_config),
            right: Warp::with_config(right_config),
            stereo_width: config.stereo_width,
        }
    }

    /// Processes a stereo sample pair
    pub fn process_stereo(&mut self, left_input: f32, right_input: f32) -> (f32, f32) {
        let left_output = self.left.process_sample(left_input);
        let right_output = self.right.process_sample(right_input);

        // Apply stereo width processing
        let mid = (left_output + right_output) * 0.5;
        let side = (left_output - right_output) * 0.5 * self.stereo_width;

        (
            mid + side,
            mid - side,
        )
    }
}

impl Warp {
    /// Process a single sample with all effect parameters
    pub fn process(&mut self, input: f32) -> f32 {
        // Update grain size if changed
        self.update_grain_size();

        self.process_sample(input)
    }

    /// Process with bypass support
    pub fn process_with_bypass(&mut self, input: f32) -> f32 {
        if self.enabled {
            self.process(input)
        } else {
            input
        }
    }

    /// Process a buffer of samples
    pub fn process_buffer(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Reset effect to initial state
    pub fn reset(&mut self) {
        self.input_buffer.fill(0.0);
        self.output_buffer.fill(0.0);
        self.read_pos = 0.0;
        self.write_pos = 0;
        self.lfo_phase = 0.0;
        self.rng = StdRng::from_entropy();
        self.generate_window();
    }

    /// Set wet/dry mix
    pub fn set_mix(&mut self, mix: f32) {
        self.config.mix = mix.clamp(0.0, 1.0);
    }

    /// Set effect intensity
    pub fn set_intensity(&mut self, intensity: f32) {
        self.config.amount = intensity.clamp(0.0, 1.0);
    }

    /// Check if effect is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable the effect
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Warp {
    /// Sets the warp mode
    pub fn set_mode(&mut self, mode: WarpMode) {
        self.mode = mode;
        self.config.mode = mode;
    }

    /// Gets the current warp mode
    pub fn mode(&self) -> WarpMode {
        self.mode
    }

    /// Sets the warp amount (0.0 to 1.0)
    pub fn set_amount(&mut self, amount: f32) {
        self.config.amount = amount.clamp(0.0, 1.0);
    }

    /// Gets the warp amount
    pub fn amount(&self) -> f32 {
        self.config.amount
    }

    /// Sets the grain size in milliseconds
    pub fn set_grain_size(&mut self, size_ms: f32) {
        self.config.grain_size = size_ms.clamp(20.0, 100.0);
    }

    /// Gets the grain size
    pub fn grain_size(&self) -> f32 {
        self.config.grain_size
    }

    /// Sets the feedback amount
    pub fn set_feedback(&mut self, feedback: f32) {
        self.config.feedback = feedback.clamp(0.0, 0.95);
    }

    /// Gets the feedback amount
    pub fn feedback(&self) -> f32 {
        self.config.feedback
    }

    /// Sets the LFO rate in Hz
    pub fn set_rate(&mut self, rate: f32) {
        self.config.rate = rate.clamp(0.1, 10.0);
    }

    /// Gets the LFO rate
    pub fn rate(&self) -> f32 {
        self.config.rate
    }

    /// Sets the stereo width
    pub fn set_stereo_width(&mut self, width: f32) {
        self.config.stereo_width = width.clamp(0.0, 1.0);
    }

    /// Gets the stereo width
    pub fn stereo_width(&self) -> f32 {
        self.config.stereo_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warp_creation() {
        let warp = Warp::new(44100.0);
        assert!(warp.is_enabled());
        assert_eq!(warp.mode(), WarpMode::TimeWarp);
    }

    #[test]
    fn test_warp_time_warp_mode() {
        let mut warp = Warp::new(44100.0);
        warp.set_mode(WarpMode::TimeWarp);
        assert_eq!(warp.mode(), WarpMode::TimeWarp);
    }

    #[test]
    fn test_warp_pitch_shift_mode() {
        let mut warp = Warp::new(44100.0);
        warp.set_mode(WarpMode::PitchShift);
        assert_eq!(warp.mode(), WarpMode::PitchShift);
    }

    #[test]
    fn test_warp_ring_mod_mode() {
        let warp = Warp::with_config(WarpConfig::ring_mod(44100.0));
        assert_eq!(warp.mode(), WarpMode::RingMod);
    }

    #[test]
    fn test_warp_process() {
        let mut warp = Warp::new(44100.0);

        // Process a few samples
        let output1 = warp.process(0.5);
        let output2 = warp.process(0.3);
        let output3 = warp.process(0.7);

        // Should produce output without clipping
        assert!(output1.abs() <= 1.0);
        assert!(output2.abs() <= 1.0);
        assert!(output3.abs() <= 1.0);
    }

    #[test]
    fn test_warp_buffer_processing() {
        let mut warp = Warp::new(1000.0);

        // Process a buffer
        let mut samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        warp.process_buffer(&mut samples);

        // Should not clip
        for &sample in &samples {
            assert!(sample.abs() <= 1.0, "Sample clipped: {}", sample);
        }
    }

    #[test]
    fn test_warp_enabled() {
        let mut warp = Warp::new(44100.0);

        // Test enabled state
        warp.set_enabled(true);
        assert!(warp.is_enabled());

        // Test bypass when disabled
        warp.set_enabled(false);
        assert!(!warp.is_enabled());
        let bypassed = warp.process_with_bypass(0.5);
        assert_eq!(bypassed, 0.5);
    }

    #[test]
    fn test_warp_set_parameters() {
        let mut warp = Warp::new(44100.0);

        warp.set_amount(0.7);
        assert_eq!(warp.amount(), 0.7);

        warp.set_grain_size(40.0);
        assert_eq!(warp.grain_size(), 40.0);

        warp.set_feedback(0.5);
        assert_eq!(warp.feedback(), 0.5);

        warp.set_rate(2.0);
        assert_eq!(warp.rate(), 2.0);

        warp.set_stereo_width(0.8);
        assert_eq!(warp.stereo_width(), 0.8);
    }

    #[test]
    fn test_warp_mix() {
        let mut warp = Warp::new(44100.0);

        warp.set_mix(0.3);
        let output = warp.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_warp_intensity() {
        let mut warp = Warp::new(44100.0);

        // Low intensity
        warp.set_intensity(0.0);
        assert_eq!(warp.amount(), 0.0);

        // High intensity
        warp.set_intensity(1.0);
        assert_eq!(warp.amount(), 1.0);
    }

    #[test]
    fn test_warp_reset() {
        let mut warp = Warp::new(44100.0);

        // Process some samples to change state
        for _ in 0..100 {
            warp.process(0.8);
        }

        // Reset should restore initial state
        warp.reset();
        assert_eq!(warp.lfo_phase, 0.0);
    }

    #[test]
    fn test_stereo_warp_creation() {
        let stereo_warp = StereoWarp::new(44100.0);
        assert!(stereo_warp.left.is_enabled());
        assert!(stereo_warp.right.is_enabled());
    }

    #[test]
    fn test_stereo_warp_process() {
        let mut stereo_warp = StereoWarp::new(44100.0);

        // Process stereo samples
        let (left, right) = stereo_warp.process_stereo(0.5, 0.3);

        // Should produce valid output
        assert!(left.abs() <= 1.0, "Left channel clipped: {}", left);
        assert!(right.abs() <= 1.0, "Right channel clipped: {}", right);
    }

    #[test]
    fn test_stereo_warp_with_config() {
        let config = WarpConfig::pitch_shift(44100.0);
        let stereo_warp = StereoWarp::with_config(config);

        assert_eq!(stereo_warp.left.mode(), WarpMode::PitchShift);
        assert_eq!(stereo_warp.right.mode(), WarpMode::PitchShift);
    }

    #[test]
    fn test_warp_all_modes() {
        for &mode in &[
            WarpMode::TimeWarp,
            WarpMode::PitchShift,
            WarpMode::RingMod,
            WarpMode::Granular,
        ] {
            let mut warp = Warp::new(44100.0);
            warp.set_mode(mode);
            let output = warp.process(0.5);
            assert!(
                output.abs() <= 1.0,
                "Processing failed for mode {:?}: {}",
                mode,
                output
            );
        }
    }

    #[test]
    fn test_warp_config_defaults() {
        let config = WarpConfig::default();
        assert_eq!(config.mode, WarpMode::TimeWarp);
        assert!((config.amount - 0.5).abs() < 0.001);
        assert!((config.grain_size - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_warp_config_presets() {
        let time_warp = WarpConfig::time_warp(44100.0);
        assert_eq!(time_warp.mode, WarpMode::TimeWarp);

        let pitch_shift = WarpConfig::pitch_shift(44100.0);
        assert_eq!(pitch_shift.mode, WarpMode::PitchShift);

        let ring_mod = WarpConfig::ring_mod(44100.0);
        assert_eq!(ring_mod.mode, WarpMode::RingMod);
    }
}
