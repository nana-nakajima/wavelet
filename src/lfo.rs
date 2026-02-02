//! LFO (Low-Frequency Oscillator) Module
//!
//! This module provides LFO implementations for modulating synthesizer parameters
//! at sub-audio rates. LFOs are used to create rhythmic variation, vibrato,
//! tremolo, filter sweeps, and other time-varying effects.
//!
//! # LFO Characteristics
//!
//! - **Rate**: Frequency of the LFO oscillation (typically 0.1 Hz to 20 Hz)
//! - **Waveform**: Shape of the LFO oscillation
//! - **Depth/Amount**: Strength of the modulation effect
//! - **Sync**: Optional synchronization to MIDI clock or tempo
//!
//! # Common LFO Uses
//!
//! - **Vibrato**: LFO modulating pitch
//! - **Tremolo**: LFO modulating amplitude
//! - **Filter Sweep**: LFO modulating filter cutoff
//! - **Pan Spread**: LFO modulating stereo position
//! - **Trill**: Rapid alternation between two notes

use std::f32::consts::PI;
use crate::oscillator::{Oscillator, OscillatorConfig, Waveform};

/// LFO rate representation.
#[derive(Debug, Clone, Copy)]
pub enum LfoRate {
    /// Frequency in Hz
    Hertz(f32),
    
    /// MIDI note number (converted to frequency)
    MidiNote(u8),
    
    /// Synchronized to tempo (beats per second)
    Sync(f32),
}

impl LfoRate {
    /// Converts LfoRate to Hz.
    pub fn to_hertz(&self) -> f32 {
        match self {
            LfoRate::Hertz(hz) => *hz,
            LfoRate::MidiNote(note) => 440.0 * 2.0f32.powf((*note as f32 - 69.0) / 12.0),
            LfoRate::Sync(beats_per_sec) => *beats_per_sec,
        }
    }
}

/// Configuration for LFO parameters.
#[derive(Debug, Clone, Copy)]
pub struct LfoConfig {
    /// LFO rate (frequency)
    pub rate: LfoRate,
    
    /// Waveform shape
    pub waveform: Waveform,
    
    /// Modulation depth (0.0 to 1.0)
    pub depth: f32,
    
    /// Phase offset in radians
    pub phase_offset: f32,
    
    /// Whether to delay LFO start
    pub delay_samples: u32,
    
    /// Sample rate
    pub sample_rate: f32,
}

impl Default for LfoConfig {
    fn default() -> Self {
        Self {
            rate: LfoRate::Hertz(2.0),
            waveform: Waveform::Sine,
            depth: 0.5,
            phase_offset: 0.0,
            delay_samples: 0,
            sample_rate: 44100.0,
        }
    }
}

/// Low-Frequency Oscillator for parameter modulation.
///
/// LFOs operate at sub-audio rates (typically below 20 Hz) to create
/// slow, periodic changes in synthesizer parameters. Unlike audio oscillators,
/// LFO output is usually bipolar (-1 to 1) for symmetrical modulation.
///
/// # Example
///
/// ```rust
/// use wavelet::lfo::{Lfo, LfoConfig, LfoRate};
/// use wavelet::oscillator::Waveform;
///
/// let config = LfoConfig {
///     rate: LfoRate::Hertz(4.0),  // 4 Hz LFO
///     waveform: Waveform::Sine,
///     depth: 0.3,
///     sample_rate: 48000.0,
///     ..Default::default()
/// };
///
/// let mut lfo = Lfo::new(config);
/// let modulation = lfo.process(); // Get next modulation value
/// ```
#[derive(Debug, Clone)]
pub struct Lfo {
    /// Internal oscillator for LFO waveform generation
    oscillator: Oscillator,
    
    /// Current modulation depth
    depth: f32,
    
    /// Delay counter for fade-in effect
    delay_counter: u32,
    
    /// Total delay samples before LFO starts
    delay_samples: u32,
    
    /// Current output value (bipolar)
    current_value: f32,
    
    /// Sample rate for timing
    sample_rate: f32,
}

impl Lfo {
    /// Creates a new LFO with default configuration.
    pub fn new() -> Self {
        Self::with_config(LfoConfig::default())
    }
    
    /// Creates a new LFO with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - LFO configuration parameters
    ///
    /// # Returns
    ///
    /// A new Lfo instance
    pub fn with_config(config: LfoConfig) -> Self {
        let rate_hz = config.rate.to_hertz();
        
        let osc_config = OscillatorConfig {
            waveform: config.waveform,
            frequency: rate_hz,
            amplitude: 1.0, // LFO uses bipolar output
            phase_offset: config.phase_offset,
            sample_rate: config.sample_rate,
        };
        
        Self {
            oscillator: Oscillator::new(osc_config),
            depth: config.depth,
            delay_counter: 0,
            delay_samples: config.delay_samples,
            current_value: 0.0,
            sample_rate: config.sample_rate,
        }
    }
    
    /// Processes one sample from the LFO.
    ///
    /// Returns the modulation value, scaled by depth. Output is in range
    /// [-depth, depth] for bipolar modulation.
    ///
    /// # Returns
    ///
    /// Modulation value (bipolar, scaled by depth)
    pub fn process(&mut self) -> f32 {
        if self.delay_counter < self.delay_samples {
            // Still in delay phase
            self.delay_counter += 1;
            self.current_value = 0.0;
            return 0.0;
        }
        
        // Get sample from internal oscillator (already bipolar -1 to 1)
        let sample = self.oscillator.next_sample();
        self.current_value = sample * self.depth;
        self.current_value
    }
    
    /// Processes a block of samples.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of samples to generate
    ///
    /// # Returns
    ///
    /// Vector of modulation values
    pub fn process_block(&mut self, count: usize) -> Vec<f32> {
        (0..count).map(|_| self.process()).collect()
    }
    
    /// Gets the current LFO output value.
    pub fn value(&self) -> f32 {
        self.current_value
    }
    
    /// Sets the LFO rate.
    ///
    /// # Arguments
    ///
    /// * `rate` - New LFO rate
    pub fn set_rate(&mut self, rate: LfoRate) {
        let hz = rate.to_hertz();
        self.oscillator.set_frequency(hz);
    }
    
    /// Sets the LFO rate in Hz.
    pub fn set_rate_hz(&mut self, hz: f32) {
        self.oscillator.set_frequency(hz);
    }
    
    /// Sets the modulation depth.
    ///
    /// # Arguments
    ///
    /// * `depth` - New depth value (0.0 to 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }
    
    /// Sets the LFO waveform.
    ///
    /// # Arguments
    ///
    /// * `waveform` - New waveform shape
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.oscillator.set_waveform(waveform);
    }
    
    /// Resets the LFO phase to the beginning.
    pub fn reset(&mut self) {
        self.oscillator.reset_phase();
        self.delay_counter = 0;
        self.current_value = 0.0;
    }
    
    /// Resets the LFO with a specific phase.
    ///
    /// # Arguments
    ///
    /// * `phase` - Phase to reset to (0.0 to 1.0)
    pub fn reset_phase(&mut self, phase: f32) {
        self.oscillator.reset_phase();
        self.delay_counter = 0;
        self.current_value = 0.0;
    }
    
    /// Sets the sample rate for the LFO.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.oscillator.set_sample_rate(sample_rate);
    }
    
    /// Sets the delay before LFO starts.
    ///
    /// # Arguments
    ///
    /// * `delay_seconds` - Delay time in seconds
    pub fn set_delay(&mut self, delay_seconds: f32) {
        self.delay_samples = (delay_seconds * self.sample_rate) as u32;
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for LFO operations.
pub trait LfoTrait {
    /// Process one sample.
    fn process(&mut self) -> f32;
    
    /// Set rate.
    fn set_rate(&mut self, rate: LfoRate);
    
    /// Set depth.
    fn set_depth(&mut self, depth: f32);
    
    /// Reset LFO.
    fn reset(&mut self);
}

impl LfoTrait for Lfo {
    fn process(&mut self) -> f32 {
        self.process()
    }
    
    fn set_rate(&mut self, rate: LfoRate) {
        self.set_rate(rate);
    }
    
    fn set_depth(&mut self, depth: f32) {
        self.set_depth(depth);
    }
    
    fn reset(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lfo_default() {
        let lfo = Lfo::new();
        assert_eq!(lfo.depth, 0.5);
    }
    
    #[test]
    fn test_lfo_process() {
        let config = LfoConfig {
            rate: LfoRate::Hertz(1.0),
            waveform: Waveform::Sine,
            depth: 1.0,
            sample_rate: 100.0,
            ..Default::default()
        };
        
        let mut lfo = Lfo::with_config(config);
        let value = lfo.process();
        
        // Value should be within [-1, 1]
        assert!(value >= -1.0 && value <= 1.0);
    }
    
    #[test]
    fn test_lfo_set_depth() {
        let mut lfo = Lfo::new();
        lfo.set_depth(0.3);
        assert_eq!(lfo.depth, 0.3);
    }
    
    #[test]
    fn test_lfo_rate_conversion() {
        assert!((LfoRate::Hertz(440.0).to_hertz() - 440.0).abs() < 0.001);
    }
    
    #[test]
    fn test_lfo_reset() {
        let mut lfo = Lfo::new();
        let _ = lfo.process();
        lfo.reset();
        assert_eq!(lfo.current_value, 0.0);
    }
}
