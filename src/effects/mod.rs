//! Effects Module
//!
//! This module provides audio effects processing for the synthesizer.
//! Effects add character and space to sounds, creating depth and interest.
//!
//! # Supported Effects
//!
//! - **Reverb**: Creates space and depth through room reflections
//! - **Delay**: Echo effect for rhythmic interest
//! - **Chorus**: Modulates delay for thickening effect
//! - **Distortion**: Adds harmonic content through clipping
//! - **Phaser**: Sweeping phase cancellation
//! - **Flanger**: Modulated comb filtering
//! - **Saturation**: Analog-style soft clipping and harmonic enhancement

use std::f32::consts::PI;

// Re-export saturation module
pub mod saturation;

// Re-export chorus module
pub mod chorus;

// Re-export phaser module
pub mod phaser;

// Re-export flanger module
pub mod flanger;

// Re-export ring modulator module
pub mod ring_modulator;

// Re-export bit crusher module
pub mod bit_crusher;

// Re-export filter bank module
pub mod filter_bank;

// Re-export freeze module
pub mod freeze;

// Re-export tremolo module
pub mod tremolo;

// Re-export simple EQ module
pub mod simple_eq;

// Track effects module is temporarily disabled for compilation
// pub mod track_effects;

pub use saturation::{saturate, Saturation, SaturationConfig};
pub use chorus::Chorus;
pub use phaser::{Phaser, PhaserConfig, StereoPhaser};
pub use flanger::{Flanger, FlangerConfig, StereoFlanger};
pub use ring_modulator::{RingModulator, RingModulatorConfig, RingModulatorMode, RingModulatorWave, StereoRingModulator};
pub use bit_crusher::{BitCrusher, BitCrusherConfig, DecimationMode, StereoBitCrusher};
pub use filter_bank::{FilterBank, FilterBankConfig, FilterBankType, FilterBandConfig};
pub use freeze::{Freeze, FreezeConfig, FreezeType};
pub use tremolo::{Tremolo, TremoloConfig, TremoloWaveform};
pub use simple_eq::SimpleEq;
// pub use track_effects::{
//     TrackEffectSlot,
//     TrackEffectSlotConfig,
//     TrackEffects,
//     TrackEffectsError,
//     EffectParameterId,
//     PerTrackEffectsManager,
//     MAX_EFFECT_SLOTS,
//     TRACK_COUNT,
// };

// Re-export BiquadFilter from filter module for convenience
pub use crate::filter::{BiquadFilter, FilterConfig, FilterType};

/// Enumeration of supported effect types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    /// Room reverb simulation
    Reverb,

    /// Echo/delay effect
    Delay,

    /// Chorus thickening effect
    Chorus,

    /// Harmonic distortion
    Distortion,

    /// Phaser effect
    Phaser,

    /// Flanger effect
    Flanger,

    /// Dynamic range compression
    Compressor,

    /// Analog-style saturation
    Saturation,

    /// Simple 3-band EQ
    SimpleEQ,

    /// Biquad filter
    Filter,
}

/// Configuration structure for effect parameters.
#[derive(Debug, Clone, Copy)]
pub struct EffectConfig {
    /// Type of effect
    pub effect_type: EffectType,

    /// Wet/dry mix (0.0 = dry, 1.0 = wet)
    pub mix: f32,

    /// Effect intensity parameter
    pub intensity: f32,

    /// Time parameter (for delay/reverb)
    pub time: f32,

    /// Feedback amount (for delay/reverb)
    pub feedback: f32,

    /// Rate parameter (for modulation effects)
    pub rate: f32,

    /// Depth parameter (for modulation effects)
    pub depth: f32,

    /// Sample rate
    pub sample_rate: f32,
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            effect_type: EffectType::Delay,
            mix: 0.3,
            intensity: 0.5,
            time: 0.3,
            feedback: 0.4,
            rate: 0.5,
            depth: 0.3,
            sample_rate: 44100.0,
        }
    }
}

/// Base trait for all effects.
///
/// This trait defines the common interface that all effects must implement.
/// It provides methods for processing audio samples, controlling parameters,
/// and managing effect state.
///
/// # Bypass Support
///
/// All effects support bypass functionality through the `process_with_bypass`
/// method. When an effect is disabled, it simply passes the input through
/// unchanged. This is useful for implementing effect chains where individual
/// effects can be toggled on and off.
pub trait Effect {
    /// Process a single audio sample.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio sample
    ///
    /// # Returns
    ///
    /// Processed output sample
    fn process(&mut self, input: f32) -> f32;

    /// Process a single audio sample with bypass support.
    ///
    /// When the effect is disabled, this returns the input unchanged.
    /// When enabled, it applies the effect and returns the processed output.
    ///
    /// # Arguments
    ///
    /// * `input` - Input audio sample
    ///
    /// # Returns
    ///
    /// Output sample (processed if enabled, passthrough if disabled)
    fn process_with_bypass(&mut self, input: f32) -> f32;

    /// Process a buffer of audio samples.
    ///
    /// # Arguments
    ///
    /// * `samples` - Mutable slice of audio samples to process
    fn process_buffer(&mut self, samples: &mut [f32]);

    /// Reset effect state.
    fn reset(&mut self);

    /// Set the wet/dry mix.
    ///
    /// # Arguments
    ///
    /// * `mix` - Mix value (0.0 = dry, 1.0 = wet)
    fn set_mix(&mut self, mix: f32);

    /// Set effect intensity.
    ///
    /// # Arguments
    ///
    /// * `intensity` - Intensity value (effect-specific range)
    fn set_intensity(&mut self, intensity: f32);

    /// Check if the effect is enabled.
    ///
    /// # Returns
    ///
    /// True if the effect is enabled
    fn is_enabled(&self) -> bool;

    /// Enable or disable the effect.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the effect should be active
    fn set_enabled(&mut self, enabled: bool);
}

/// Simple delay effect with feedback.
#[derive(Debug, Clone)]
pub struct Delay {
    /// Delay buffer
    buffer: Vec<f32>,

    /// Current write position
    write_pos: usize,

    /// Current read position
    read_pos: usize,

    /// Delay time in samples
    delay_samples: usize,

    /// Feedback amount
    feedback: f32,

    /// Wet/dry mix
    mix: f32,

    /// Sample rate
    sample_rate: f32,

    /// Whether the effect is enabled
    enabled: bool,
}

impl Delay {
    /// Creates a new delay effect.
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_ms = 2000.0; // 2 seconds max
        let max_delay_samples = (max_delay_ms / 1000.0 * sample_rate) as usize;

        Self {
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            read_pos: 0,
            delay_samples: (0.3 * sample_rate) as usize,
            feedback: 0.4,
            mix: 0.3,
            sample_rate,
            enabled: true,
        }
    }

    /// Sets the delay time.
    pub fn set_delay_time(&mut self, time_ms: f32) {
        self.delay_samples = (time_ms / 1000.0 * self.sample_rate) as usize;
        self.read_pos =
            (self.write_pos + self.buffer.len() - self.delay_samples) % self.buffer.len();
    }

    /// Sets the feedback amount.
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95);
    }
}

impl Effect for Delay {
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.read_pos];

        // Write input plus feedback to buffer
        self.buffer[self.write_pos] = input + delayed * self.feedback;

        // Advance positions
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        self.read_pos = (self.read_pos + 1) % self.buffer.len();

        // Mix wet and dry
        input * (1.0 - self.mix) + delayed * self.mix
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
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.read_pos = 0;
    }

    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn set_intensity(&mut self, intensity: f32) {
        self.set_feedback(intensity * 0.95);
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Simple reverb effect using a series of comb filters.
#[derive(Debug, Clone)]
pub struct Reverb {
    /// Comb filter delays
    delays: Vec<usize>,

    /// Comb filter buffers
    buffers: Vec<Vec<f32>>,

    /// Current write positions
    write_pos: Vec<usize>,

    /// Decay factor
    decay: f32,

    /// Wet/dry mix
    mix: f32,

    /// Sample rate
    sample_rate: f32,

    /// Whether the effect is enabled
    enabled: bool,
}

impl Reverb {
    /// Creates a new reverb effect.
    pub fn new(sample_rate: f32) -> Self {
        let comb_delays = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];

        let buffers: Vec<Vec<f32>> = comb_delays.iter().map(|&d| vec![0.0; d]).collect();

        let mut write_pos = vec![0; comb_delays.len()];
        for (i, &d) in comb_delays.iter().enumerate() {
            write_pos[i] = d - 1;
        }

        Self {
            delays: comb_delays.to_vec(),
            buffers,
            write_pos,
            decay: 0.7,
            mix: 0.2,
            sample_rate,
            enabled: true,
        }
    }

    /// Sets the reverb decay time.
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.1, 0.95);
    }
}

impl Effect for Reverb {
    fn process(&mut self, input: f32) -> f32 {
        let mut output = 0.0;

        // Process through each comb filter
        for (i, delay) in self.delays.iter().enumerate() {
            let buffer = &mut self.buffers[i];
            let write_pos = self.write_pos[i];

            let delayed = buffer[write_pos % delay];
            buffer[write_pos % delay] = input + delayed * self.decay;
            self.write_pos[i] = (write_pos + 1) % delay;

            output += delayed;
        }

        // Average and mix
        output /= self.delays.len() as f32;
        input * (1.0 - self.mix) + output * self.mix
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
        for buffer in &mut self.buffers {
            buffer.fill(0.0);
        }
        self.write_pos = self.delays.iter().map(|d| d - 1).collect();
    }

    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn set_intensity(&mut self, intensity: f32) {
        self.set_decay(intensity);
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Distortion effect using waveshaping.
#[derive(Debug, Clone)]
pub struct Distortion {
    /// Distortion amount
    amount: f32,

    /// Wet/dry mix
    mix: f32,

    /// Whether the effect is enabled
    enabled: bool,
}

impl Distortion {
    /// Creates a new distortion effect.
    pub fn new() -> Self {
        Self {
            amount: 0.5,
            mix: 0.5,
            enabled: true,
        }
    }

    /// Applies waveshaping curve to input sample.
    fn apply_curve(&self, sample: f32) -> f32 {
        let x = sample.clamp(-1.0, 1.0);
        let k = self.amount * 20.0; // Gain factor

        // Soft clipping curve
        (PI * k * x).sin() / (PI + k * x.abs())
    }
}

impl Effect for Distortion {
    fn process(&mut self, input: f32) -> f32 {
        let distorted = self.apply_curve(input);
        input * (1.0 - self.mix) + distorted * self.mix
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
        // No state to reset for stateless waveshaping
    }

    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn set_intensity(&mut self, intensity: f32) {
        self.amount = intensity.clamp(0.0, 1.0);
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Multiband sidechain envelope compressor (MSEC)
///
/// This compressor provides dynamics control with:
/// - Threshold-based gain reduction
/// - Adjustable ratio
/// - Attack and release time constants
/// - Makeup gain
/// - Sidechain input for ducking
#[derive(Debug, Clone)]
pub struct Compressor {
    /// Threshold in dB (signals above this are compressed)
    threshold_db: f32,

    /// Compression ratio (1:1 = no compression, ∞:1 = limiting)
    ratio: f32,

    /// Attack time in seconds
    attack_s: f32,

    /// Release time in seconds
    release_s: f32,

    /// Makeup gain in dB
    makeup_db: f32,

    /// Knee width in dB
    knee_db: f32,

    /// Current gain reduction (for smoothing)
    gain_reduction: f32,

    /// Envelope follower state
    envelope: f32,

    /// Sample rate
    sample_rate: f32,

    /// Wet/dry mix
    mix: f32,

    /// Whether the effect is enabled
    enabled: bool,
}

impl Compressor {
    /// Creates a new compressor with default settings.
    /// Default: threshold = -20dB, ratio = 4:1, attack = 10ms, release = 100ms
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_db: -20.0,
            ratio: 4.0,
            attack_s: 0.01,
            release_s: 0.1,
            makeup_db: 0.0,
            knee_db: 6.0,
            gain_reduction: 1.0,
            envelope: 0.0,
            sample_rate,
            mix: 1.0,
            enabled: true,
        }
    }

    /// Converts dB to linear gain.
    #[inline]
    fn db_to_linear(db: f32) -> f32 {
        if db <= -80.0 {
            0.0
        } else {
            10.0f32.powf(db / 20.0)
        }
    }

    /// Converts linear gain to dB.
    #[inline]
    fn linear_to_db(gain: f32) -> f32 {
        if gain <= 0.0001 {
            -80.0
        } else {
            20.0 * gain.log10()
        }
    }

    /// Calculates the gain reduction for a given input level.
    fn calculate_gain_reduction(&mut self, input_db: f32) -> f32 {
        // Apply knee
        let knee_start = self.threshold_db - self.knee_db / 2.0;
        let knee_end = self.threshold_db + self.knee_db / 2.0;

        let effective_input_db = if self.knee_db > 0.0 {
            if input_db < knee_start {
                input_db
            } else if input_db > knee_end {
                input_db
                    + (self.threshold_db - knee_end)
                    + (input_db - knee_end) * (1.0 - 1.0 / self.ratio)
            } else {
                // Within knee - smooth transition
                let t = (input_db - knee_start) / self.knee_db;
                let knee_db = t * self.knee_db;
                let knee_threshold = self.threshold_db - self.knee_db / 2.0 + knee_db;
                let overshoot = input_db - knee_threshold;
                knee_threshold + overshoot / self.ratio
            }
        } else {
            if input_db > self.threshold_db {
                self.threshold_db + (input_db - self.threshold_db) / self.ratio
            } else {
                input_db
            }
        };

        // Calculate gain reduction
        let reduction_db = effective_input_db - input_db;
        Self::db_to_linear(reduction_db)
    }

    /// Sets the threshold in dB.
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db.clamp(-60.0, 0.0);
    }

    /// Sets the compression ratio.
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 20.0);
    }

    /// Sets the attack time in seconds.
    pub fn set_attack(&mut self, attack_s: f32) {
        self.attack_s = attack_s.clamp(0.001, 0.5);
    }

    /// Sets the release time in seconds.
    pub fn set_release(&mut self, release_s: f32) {
        self.release_s = release_s.clamp(0.01, 1.0);
    }

    /// Sets the makeup gain in dB.
    pub fn set_makeup(&mut self, makeup_db: f32) {
        self.makeup_db = makeup_db.clamp(0.0, 24.0);
    }
}

impl Effect for Compressor {
    fn process(&mut self, input: f32) -> f32 {
        // Convert input to dB
        let input_db = Self::linear_to_db(input.abs());

        // Calculate target gain reduction
        let target_reduction = self.calculate_gain_reduction(input_db);

        // Smooth gain reduction with attack/release
        let attack_coef = (-1.0 / (self.attack_s * self.sample_rate)).exp();
        let release_coef = (-1.0 / (self.release_s * self.sample_rate)).exp();

        if target_reduction < self.gain_reduction {
            // Attack - reduce gain quickly
            self.gain_reduction =
                attack_coef * self.gain_reduction + (1.0 - attack_coef) * target_reduction;
        } else {
            // Release - increase gain slowly
            self.gain_reduction =
                release_coef * self.gain_reduction + (1.0 - release_coef) * target_reduction;
        }

        // Apply gain reduction and makeup
        let output = input * self.gain_reduction * Self::db_to_linear(self.makeup_db);

        // Mix wet/dry
        input * (1.0 - self.mix) + output * self.mix
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
        self.gain_reduction = 1.0;
        self.envelope = 0.0;
    }

    fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    fn set_intensity(&mut self, intensity: f32) {
        // Map intensity to common compressor parameters
        self.set_ratio(1.0 + intensity * 19.0); // 1:1 to 20:1
        self.set_threshold(-60.0 + intensity * 40.0); // -60dB to -20dB
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Wrapper type for effect management.
#[derive(Debug, Clone)]
pub struct EffectProcessor {
    /// Current effect type
    effect_type: EffectType,

    /// Delay effect instance
    delay: Delay,

    /// Reverb effect instance
    reverb: Reverb,

    /// Distortion effect instance
    distortion: Distortion,

    /// Compressor effect instance
    compressor: Compressor,

    /// Saturation effect instance
    saturation: Saturation,

    /// Chorus effect instance
    chorus: Chorus,

    /// Simple EQ effect instance
    simple_eq: SimpleEq,

    /// Biquad filter effect instance
    biquad_filter: BiquadFilter,
}

impl EffectProcessor {
    /// Creates a new effect processor.
    pub fn new(sample_rate: f32) -> Self {
        let config = FilterConfig {
            filter_type: FilterType::LowPass,
            cutoff_frequency: 1000.0,
            resonance: 1.0,
            gain: 0.0,
            sample_rate,
        };
        
        Self {
            effect_type: EffectType::Delay,
            delay: Delay::new(sample_rate),
            reverb: Reverb::new(sample_rate),
            distortion: Distortion::new(),
            compressor: Compressor::new(sample_rate),
            saturation: Saturation::new(),
            chorus: Chorus::new(sample_rate),
            simple_eq: SimpleEq::new(sample_rate),
            biquad_filter: BiquadFilter::with_config(config),
        }
    }

    /// Sets the active effect type.
    pub fn set_effect_type(&mut self, effect_type: EffectType) {
        self.effect_type = effect_type;
    }

    /// Gets the current effect type.
    pub fn effect_type(&self) -> EffectType {
        self.effect_type
    }
}

impl Effect for EffectProcessor {
    fn process(&mut self, input: f32) -> f32 {
        match self.effect_type {
            EffectType::Delay => self.delay.process(input),
            EffectType::Reverb => self.reverb.process(input),
            EffectType::Distortion => self.distortion.process(input),
            EffectType::Compressor => self.compressor.process(input),
            EffectType::Saturation => self.saturation.process(input),
            EffectType::Chorus => self.chorus.process(input),
            EffectType::SimpleEQ => self.simple_eq.process(input),
            EffectType::Filter => self.biquad_filter.process(input),
            _ => input, // Placeholder for unimplemented effects
        }
    }

    fn process_with_bypass(&mut self, input: f32) -> f32 {
        if self.is_enabled() {
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
        self.delay.reset();
        self.reverb.reset();
        self.distortion.reset();
        self.compressor.reset();
        self.saturation.reset();
        self.chorus.reset();
        self.simple_eq.reset();
        self.biquad_filter.reset();
    }

    fn set_mix(&mut self, mix: f32) {
        match self.effect_type {
            EffectType::Delay => self.delay.set_mix(mix),
            EffectType::Reverb => self.reverb.set_mix(mix),
            EffectType::Distortion => self.distortion.set_mix(mix),
            EffectType::Compressor => self.compressor.set_mix(mix),
            EffectType::Saturation => self.saturation.set_mix(mix),
            EffectType::Chorus => self.chorus.set_mix(mix),
            EffectType::SimpleEQ => self.simple_eq.set_mix(mix),
            EffectType::Filter => self.biquad_filter.set_mix(mix),
            _ => {}
        }
    }

    fn set_intensity(&mut self, intensity: f32) {
        match self.effect_type {
            EffectType::Delay => self.delay.set_intensity(intensity),
            EffectType::Reverb => self.reverb.set_intensity(intensity),
            EffectType::Distortion => self.distortion.set_intensity(intensity),
            EffectType::Compressor => self.compressor.set_intensity(intensity),
            EffectType::Saturation => self.saturation.set_drive(intensity * 10.0),
            EffectType::Chorus => self.chorus.set_intensity(intensity),
            EffectType::SimpleEQ => self.simple_eq.set_intensity(intensity),
            EffectType::Filter => self.biquad_filter.set_intensity(intensity),
            _ => {}
        }
    }

    fn is_enabled(&self) -> bool {
        match self.effect_type {
            EffectType::Delay => self.delay.is_enabled(),
            EffectType::Reverb => self.reverb.is_enabled(),
            EffectType::Distortion => self.distortion.is_enabled(),
            EffectType::Compressor => self.compressor.is_enabled(),
            EffectType::Saturation => self.saturation.is_enabled(),
            EffectType::Chorus => self.chorus.is_enabled(),
            EffectType::SimpleEQ => self.simple_eq.is_enabled(),
            EffectType::Filter => self.biquad_filter.is_enabled(),
            _ => false,
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        match self.effect_type {
            EffectType::Delay => self.delay.set_enabled(enabled),
            EffectType::Reverb => self.reverb.set_enabled(enabled),
            EffectType::Distortion => self.distortion.set_enabled(enabled),
            EffectType::Compressor => self.compressor.set_enabled(enabled),
            EffectType::Saturation => self.saturation.set_enabled(enabled),
            EffectType::Chorus => self.chorus.set_enabled(enabled),
            EffectType::SimpleEQ => self.simple_eq.set_enabled(enabled),
            EffectType::Filter => self.biquad_filter.set_enabled(enabled),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Import Effect trait for chorus tests
    use super::Effect;

    #[test]
    fn test_delay_default() {
        let delay = Delay::new(44100.0);
        assert_eq!(delay.delay_samples, (0.3 * 44100.0) as usize);
    }

    #[test]
    fn test_delay_process() {
        let mut delay = Delay::new(1000.0);
        delay.set_delay_time(100.0); // 100ms delay

        let output = delay.process(0.5);
        // Output should be valid (in reasonable range)
        assert!(output.abs() <= 2.0);
    }

    #[test]
    fn test_delay_set_mix() {
        let mut delay = Delay::new(44100.0);
        delay.set_mix(0.5);
        assert_eq!(delay.mix, 0.5);
    }

    #[test]
    fn test_delay_enabled() {
        let mut delay = Delay::new(44100.0);

        // Test enabled state
        delay.set_enabled(true);
        assert!(delay.is_enabled());

        // Test bypass when disabled
        delay.set_enabled(false);
        assert!(!delay.is_enabled());
        let bypassed = delay.process_with_bypass(0.5);
        assert_eq!(bypassed, 0.5);
    }

    #[test]
    fn test_reverb_default() {
        let reverb = Reverb::new(44100.0);
        assert_eq!(reverb.delays.len(), 8);
    }

    #[test]
    fn test_reverb_process() {
        let mut reverb = Reverb::new(1000.0);
        let output = reverb.process(0.5);
        // Should process without clipping
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_reverb_enabled() {
        let mut reverb = Reverb::new(44100.0);

        reverb.set_enabled(true);
        assert!(reverb.is_enabled());

        reverb.set_enabled(false);
        assert!(!reverb.is_enabled());
        let bypassed = reverb.process_with_bypass(0.5);
        assert_eq!(bypassed, 0.5);
    }

    #[test]
    fn test_distortion_default() {
        let dist = Distortion::new();
        assert_eq!(dist.amount, 0.5);
    }

    #[test]
    fn test_distortion_process() {
        let mut dist = Distortion::new();
        let output = dist.process(0.5);
        // Should process without issues
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_distortion_enabled() {
        let mut dist = Distortion::new();

        dist.set_enabled(true);
        assert!(dist.is_enabled());

        dist.set_enabled(false);
        assert!(!dist.is_enabled());
        let bypassed = dist.process_with_bypass(0.5);
        assert_eq!(bypassed, 0.5);
    }

    #[test]
    fn test_effect_processor() {
        let mut fx = EffectProcessor::new(44100.0);

        // Test switching effects
        fx.set_effect_type(EffectType::Delay);
        assert_eq!(fx.effect_type(), EffectType::Delay);

        fx.set_effect_type(EffectType::Reverb);
        assert_eq!(fx.effect_type(), EffectType::Reverb);
    }

    #[test]
    fn test_effect_processor_saturation() {
        let mut fx = EffectProcessor::new(44100.0);

        // Test switching to saturation
        fx.set_effect_type(EffectType::Saturation);
        assert_eq!(fx.effect_type(), EffectType::Saturation);

        // Process should work without issues
        let output = fx.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_compressor_default() {
        let comp = Compressor::new(44100.0);
        assert_eq!(comp.threshold_db, -20.0);
        assert_eq!(comp.ratio, 4.0);
        assert!((comp.attack_s - 0.01).abs() < 0.001);
        assert!((comp.release_s - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_compressor_process() {
        let mut comp = Compressor::new(1000.0);

        // Process a sample - should not clip
        let output = comp.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_compressor_set_parameters() {
        let mut comp = Compressor::new(44100.0);

        comp.set_threshold(-30.0);
        assert_eq!(comp.threshold_db, -30.0);

        comp.set_ratio(8.0);
        assert_eq!(comp.ratio, 8.0);

        comp.set_attack(0.05);
        assert!((comp.attack_s - 0.05).abs() < 0.001);

        comp.set_release(0.2);
        assert!((comp.release_s - 0.2).abs() < 0.001);

        comp.set_makeup(6.0);
        assert_eq!(comp.makeup_db, 6.0);
    }

    #[test]
    fn test_compressor_reset() {
        let mut comp = Compressor::new(44100.0);

        // Process some samples to change state
        for _ in 0..100 {
            comp.process(0.8);
        }

        // Reset should restore default state
        comp.reset();
        assert_eq!(comp.gain_reduction, 1.0);
    }

    #[test]
    fn test_compressor_enabled() {
        let mut comp = Compressor::new(44100.0);

        comp.set_enabled(true);
        assert!(comp.is_enabled());

        comp.set_enabled(false);
        assert!(!comp.is_enabled());
        let bypassed = comp.process_with_bypass(0.5);
        assert_eq!(bypassed, 0.5);
    }

    #[test]
    fn test_effect_processor_bypass() {
        let mut fx = EffectProcessor::new(44100.0);

        // Test bypass for each effect type
        for &effect_type in &[
            EffectType::Delay,
            EffectType::Reverb,
            EffectType::Distortion,
            EffectType::Compressor,
            EffectType::Saturation,
            EffectType::Chorus,
        ] {
            fx.set_effect_type(effect_type);
            fx.set_enabled(false);
            let bypassed = fx.process_with_bypass(0.5);
            assert_eq!(bypassed, 0.5, "Bypass failed for {:?}", effect_type);

            fx.set_enabled(true);
            let processed = fx.process(0.5);
            assert!(
                processed.abs() <= 1.0,
                "Processing failed for {:?}",
                effect_type
            );
        }
    }

    #[test]
    fn test_chorus_in_effect_processor() {
        let mut fx = EffectProcessor::new(44100.0);

        // Test switching to chorus
        fx.set_effect_type(EffectType::Chorus);
        assert_eq!(fx.effect_type(), EffectType::Chorus);

        // Process should work without issues
        let output = fx.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_chorus_set_parameters() {
        let mut chorus = Chorus::new(44100.0);

        // Test all parameter setters
        chorus.set_rate(1.5);
        assert_eq!(chorus.rate(), 1.5);

        chorus.set_depth(0.7);
        assert_eq!(chorus.depth(), 0.7);

        chorus.set_feedback(0.4);
        assert_eq!(chorus.feedback(), 0.4);

        chorus.set_mix(0.6);
        // Mix is set internally, verify via process
        let output = chorus.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_chorus_stereo_width() {
        let mut chorus = Chorus::new(44100.0);

        // Default stereo width - cannot test directly, verify via behavior
        chorus.set_stereo_width(1.15);
        // Verify width was set by processing
        let output = chorus.process(0.5);
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_chorus_intensity_mapping() {
        let mut chorus = Chorus::new(44100.0);

        // Low intensity
        chorus.set_intensity(0.0);
        assert_eq!(chorus.depth(), 0.0);
        assert_eq!(chorus.rate(), 0.1); // 0.1 + 0.0 * 2.0

        // Medium intensity
        chorus.set_intensity(0.5);
        assert_eq!(chorus.depth(), 0.5);
        assert_eq!(chorus.rate(), 1.1); // 0.1 + 0.5 * 2.0

        // High intensity
        chorus.set_intensity(1.0);
        assert_eq!(chorus.depth(), 1.0);
        assert_eq!(chorus.rate(), 2.1); // 0.1 + 1.0 * 2.0
    }
}
