//! Oscillator Module
//!
//! This module provides the core oscillator implementation for waveform generation.
//! Oscillators are the fundamental building blocks of any synthesizer, producing
//! the basic tonal content that gets shaped by filters and envelopes.
//!
//! # Waveforms
//!
//! The oscillator supports four basic waveforms:
//! - **Sine**: Pure tone, no harmonics
//! - **Square**: Even harmonics, distinctive "8-bit" sound
//! - **Sawtooth**: All harmonics, bright and buzzy
//! - **Triangle**: Odd harmonics only, softer than sawtooth
//!
//! # Audio Rate vs Control Rate
//!
//! Oscillators can operate at audio rate (20Hz-20kHz, audible) or control rate
//! (below 20Hz, typically used for LFOs). See the [Lfo] module for control rate usage.

#![allow(dead_code)] // Reserve oversample fields for future features

use rand::Rng;
use std::f32::consts::PI;

/// Enumeration of supported oscillator waveforms.
/// Each waveform has distinct harmonic characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    /// Pure sine wave - fundamental frequency only, no harmonics
    Sine,

    /// Square wave - alternating between +1 and -1
    /// Contains only odd harmonics at decreasing amplitudes
    Square,

    /// Sawtooth wave - ramps linearly from -1 to +1
    /// Contains all harmonics at amplitudes proportional to 1/harmonic
    Sawtooth,

    /// Triangle wave - ramps linearly up and down
    /// Contains only odd harmonics with 1/harmonic^2 amplitude decay
    Triangle,

    /// White noise - random values with uniform distribution
    /// Contains equal energy at all frequencies
    Noise,

    /// Phase modulation waveform - for FM synthesis
    /// Generates carrier for phase modulation
    PM,
}

/// Oversampling factor for anti-aliasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OversampleFactor {
    /// No oversampling (1x)
    #[default]
    None = 1,

    /// 2x oversampling
    X2 = 2,

    /// 4x oversampling
    X4 = 4,

    /// 8x oversampling
    X8 = 8,
}

impl OversampleFactor {
    /// Gets the oversampling factor as u32.
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

/// Enumeration for quick oscillator type selection.
/// Maps to Waveform variants for API convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscillatorType {
    /// Sine wave oscillator
    Sine,

    /// Square wave oscillator
    Square,

    /// Sawtooth wave oscillator
    Sawtooth,

    /// Triangle wave oscillator
    Triangle,

    /// White noise oscillator
    Noise,

    /// Phase modulation oscillator
    PM,
}

impl From<OscillatorType> for Waveform {
    fn from(t: OscillatorType) -> Self {
        match t {
            OscillatorType::Sine => Waveform::Sine,
            OscillatorType::Square => Waveform::Square,
            OscillatorType::Sawtooth => Waveform::Sawtooth,
            OscillatorType::Triangle => Waveform::Triangle,
            OscillatorType::Noise => Waveform::Noise,
            OscillatorType::PM => Waveform::PM,
        }
    }
}

/// Configuration for an oscillator instance.
/// Contains all parameters needed to configure oscillator behavior.
#[derive(Debug, Clone)]
pub struct OscillatorConfig {
    /// The waveform shape to generate
    pub waveform: Waveform,

    /// Fundamental frequency in Hz (20.0 to 20000.0 typical range)
    pub frequency: f32,

    /// Waveform amplitude (0.0 to 1.0)
    pub amplitude: f32,

    /// Phase offset in radians (0.0 to 2*PI)
    pub phase_offset: f32,

    /// Sample rate of the audio system
    pub sample_rate: f32,

    /// Oversampling factor for anti-aliasing
    pub oversample_factor: OversampleFactor,
}

impl Default for OscillatorConfig {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            frequency: 440.0,
            amplitude: 0.5,
            phase_offset: 0.0,
            sample_rate: 44100.0,
            oversample_factor: OversampleFactor::None,
        }
    }
}

/// Core oscillator structure for generating periodic waveforms.
///
/// The oscillator maintains internal state (current phase) and generates
/// samples on demand based on its configuration. Phase is continuously
/// accumulated and wrapped around 2*PI radians.
///
/// # Oversampling
///
/// For waveforms with strong harmonics (sawtooth, square), the oscillator
/// can use oversampling to reduce aliasing. When oversampling is enabled:
/// 1. The oscillator runs at a higher internal sample rate
/// 2. Multiple samples are generated for each output sample
/// 3. A decimation filter reduces the output back to the base sample rate
///
/// This significantly reduces aliasing artifacts in high-frequency content.
///
/// # Example
///
/// ```rust
/// use wavelet::oscillator::{Oscillator, OscillatorConfig, Waveform, OversampleFactor};
///
/// let mut config = OscillatorConfig {
///     waveform: Waveform::Sawtooth,
///     frequency: 220.0,
///     amplitude: 0.8,
///     sample_rate: 48000.0,
///     phase_offset: 0.0,
///     oversample_factor: OversampleFactor::X4, // 4x oversampling for anti-aliasing
/// };
///
/// let mut osc = Oscillator::new(config);
/// let sample = osc.next_sample(); // Get first sample
/// ```
#[derive(Debug, Clone)]
pub struct Oscillator {
    /// Current phase position within the waveform cycle (0.0 to 1.0)
    phase: f32,

    /// Phase increment per sample (frequency / sample_rate)
    phase_increment: f32,

    /// Current waveform type
    waveform: Waveform,

    /// Current amplitude level
    amplitude: f32,

    /// Sample rate for phase calculations
    sample_rate: f32,

    /// Random number generator for noise
    rng: rand::rngs::ThreadRng,

    /// Oversampling factor
    oversample_factor: OversampleFactor,

    /// Oversampling buffer for decimation
    oversample_buffer: Vec<f32>,

    /// Current position in oversample buffer
    oversample_pos: usize,
}

impl Oscillator {
    /// Creates a new oscillator with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Oscillator configuration containing all parameters
    ///
    /// # Returns
    ///
    /// A new Oscillator instance ready to generate samples
    pub fn new(config: OscillatorConfig) -> Self {
        let phase_increment = config.frequency / config.sample_rate;
        let oversample_factor = config.oversample_factor;
        let oversample_count = oversample_factor.as_u32() as usize;

        Self {
            phase: 0.0,
            phase_increment,
            waveform: config.waveform,
            amplitude: config.amplitude,
            sample_rate: config.sample_rate,
            rng: rand::thread_rng(),
            oversample_factor,
            oversample_buffer: vec![0.0; oversample_count],
            oversample_pos: 0,
        }
    }
}

impl Default for Oscillator {
    fn default() -> Self {
        Self::new(OscillatorConfig::default())
    }
}

impl Oscillator {
    /// Creates a new oscillator with default configuration.
    /// Uses A4 (440 Hz) as default frequency, sine wave, and 0.5 amplitude.
    pub fn new_default() -> Self {
        Self::default()
    }

    /// Sets the oscillator frequency.
    ///
    /// # Arguments
    ///
    /// * `frequency` - New frequency in Hz
    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_increment = frequency / self.sample_rate;
    }

    /// Sets the oscillator waveform type.
    ///
    /// # Arguments
    ///
    /// * `waveform` - New waveform shape to generate
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Sets the oscillator amplitude.
    ///
    /// # Arguments
    ///
    /// * `amplitude` - New amplitude level (0.0 to 1.0)
    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude.clamp(0.0, 1.0);
    }

    /// Sets the sample rate for phase calculations.
    /// Call this when the audio system sample rate changes.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - New sample rate in Hz
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        // Calculate current frequency from old sample rate
        let current_freq = self.phase_increment * self.sample_rate;
        // Update sample rate
        self.sample_rate = sample_rate;
        // Recalculate phase increment with new sample rate
        self.phase_increment = current_freq / sample_rate;
    }

    /// Sets the oversampling factor for anti-aliasing.
    ///
    /// Higher oversampling factors reduce aliasing but increase CPU usage.
    /// 1x = no oversampling, 2x = 2x oversampling, etc.
    ///
    /// # Arguments
    ///
    /// * `factor` - Oversampling factor
    pub fn set_oversample_factor(&mut self, factor: OversampleFactor) {
        let new_factor = factor.as_u32() as usize;

        // Reallocate buffer if needed
        if new_factor > 1 {
            self.oversample_buffer.resize(new_factor, 0.0);
        }

        self.oversample_factor = factor;
    }

    /// Gets the current oversampling factor.
    ///
    /// # Returns
    ///
    /// Current oversampling factor
    pub fn oversample_factor(&self) -> OversampleFactor {
        self.oversample_factor
    }

    /// Resets the oscillator phase to the starting position.
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Synchronizes multiple oscillators to the same phase.
    /// Useful for creating oscillator sync effects.
    ///
    /// # Arguments
    ///
    /// * `source` - Reference oscillator to sync from
    pub fn sync_phase(&mut self, source: &Oscillator) {
        self.phase = source.phase;
    }

    /// Generates the next audio sample from the oscillator.
    ///
    /// This method calculates the sample value based on current phase
    /// and waveform type, then advances the phase for the next sample.
    ///
    /// When oversampling is enabled, this method handles the oversampling
    /// process automatically, generating multiple high-rate samples and
    /// applying decimation to produce the final output sample.
    ///
    /// # Returns
    ///
    /// The next sample value in the range [-amplitude, amplitude]
    pub fn next_sample(&mut self) -> f32 {
        let oversample_factor = self.oversample_factor.as_u32() as usize;

        if oversample_factor <= 1 {
            // No oversampling - generate sample directly
            let sample = self.sample_waveform();
            self.advance_phase();
            sample
        } else {
            // Oversampling mode - generate and accumulate samples
            // Calculate the phase increment for the oversampled rate
            let oversample_phase_increment = self.phase_increment / oversample_factor as f32;

            // Generate oversampled samples
            for i in 0..oversample_factor {
                self.oversample_buffer[i] = self.sample_waveform();
                self.phase += oversample_phase_increment;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
            }

            // Apply simple decimation (average the oversampled samples)
            // This acts as a low-pass filter to remove aliasing artifacts
            let sum: f32 = self.oversample_buffer.iter().sum();
            sum / oversample_factor as f32
        }
    }

    /// Generates multiple samples for batch processing.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of samples to generate
    ///
    /// # Returns
    ///
    /// A vector containing the generated samples
    pub fn next_samples(&mut self, count: usize) -> Vec<f32> {
        (0..count).map(|_| self.next_sample()).collect()
    }

    /// Internal method to sample the current waveform at current phase.
    fn sample_waveform(&mut self) -> f32 {
        // Convert phase from [0, 1) to [0, 2*PI) for trigonometric functions
        let phase_2pi = self.phase * 2.0 * PI;

        match self.waveform {
            Waveform::Sine => phase_2pi.sin() * self.amplitude,

            Waveform::Square => {
                if self.phase < 0.5 {
                    self.amplitude
                } else {
                    -self.amplitude
                }
            }

            Waveform::Sawtooth => (2.0 * self.phase - 1.0) * self.amplitude,

            Waveform::Triangle => {
                let value = if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                };
                value * self.amplitude
            }

            Waveform::Noise => {
                // White noise: random values in [-1, 1]
                (self.rng.gen::<f32>() * 2.0 - 1.0) * self.amplitude
            }

            Waveform::PM => {
                // Phase modulation carrier - sine wave for FM synthesis
                phase_2pi.sin() * self.amplitude
            }
        }
    }

    /// Internal method to advance the phase by one sample.
    fn advance_phase(&mut self) {
        self.phase += self.phase_increment;
        // Wrap phase around when it exceeds 1.0
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }
}

/// Converts MIDI note number to frequency.
///
/// MIDI notes are numbered from 0 (C-1) to 127 (G9).
/// A4 (concert A) is MIDI note 69 and corresponds to 440 Hz.
///
/// # Arguments
///
/// * `midi_note` - MIDI note number (0-127)
///
/// # Returns
///
/// Frequency in Hz
///
/// # Example
///
/// ```rust
/// use wavelet::oscillator::midi_to_frequency;
///
/// assert!((midi_to_frequency(69) - 440.0).abs() < 0.001);
/// assert!((midi_to_frequency(60) - 261.63).abs() < 0.1);
/// ```
pub fn midi_to_frequency(midi_note: u8) -> f32 {
    440.0 * 2.0f32.powf((midi_note as f32 - 69.0) / 12.0)
}

/// Converts frequency to MIDI note number with optional cents detuning.
///
/// # Arguments
///
/// * `frequency` - Frequency in Hz
/// * `cents` - Detuning in cents (-100 to 100, optional)
///
/// # Returns
///
/// Tuple of (MIDI note number, fractional part for cents)
pub fn frequency_to_midi(frequency: f32) -> f32 {
    12.0 * (frequency / 440.0).log2() + 69.0
}

/// Calculates the phase increment for a given frequency and sample rate.
///
/// # Arguments
///
/// * `frequency` - Target frequency in Hz
/// * `sample_rate` - Audio system sample rate in Hz
///
/// # Returns
///
/// Phase increment per sample (frequency / sample_rate)
pub fn calculate_phase_increment(frequency: f32, sample_rate: f32) -> f32 {
    frequency / sample_rate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_default() {
        let osc = Oscillator::default();
        // frequency is stored as phase_increment = 440.0 / 44100.0
        let expected_phase_increment = 440.0 / 44100.0;
        assert!((osc.phase_increment - expected_phase_increment).abs() < 0.00001);
        assert_eq!(osc.amplitude, 0.5);
        assert_eq!(osc.waveform, Waveform::Sine);
    }

    #[test]
    fn test_midi_to_frequency() {
        // A4 (MIDI 69) should be 440 Hz
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.001);

        // C4 (MIDI 60) should be approximately 261.63 Hz
        assert!((midi_to_frequency(60) - 261.63).abs() < 0.1);

        // C5 (MIDI 72) should be approximately 523.25 Hz
        assert!((midi_to_frequency(72) - 523.25).abs() < 0.1);
    }

    #[test]
    fn test_phase_increment() {
        let increment = calculate_phase_increment(440.0, 44100.0);
        assert!((increment - 0.009977).abs() < 0.0001);
    }

    #[test]
    fn test_waveform_conversion() {
        assert_eq!(Waveform::from(OscillatorType::Sine), Waveform::Sine);
        assert_eq!(Waveform::from(OscillatorType::Square), Waveform::Square);
        assert_eq!(Waveform::from(OscillatorType::Sawtooth), Waveform::Sawtooth);
        assert_eq!(Waveform::from(OscillatorType::Triangle), Waveform::Triangle);
    }
}
