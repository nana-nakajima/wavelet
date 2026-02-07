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

    // --- Helper: count zero crossings (positive-going) ---
    fn count_zero_crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count()
    }

    // --- Sine: frequency accuracy via zero crossings ---
    #[test]
    fn test_sine_frequency_accuracy() {
        let sample_rate = 48000.0;
        let freq = 440.0;
        let duration_secs = 1.0;
        let num_samples = (sample_rate * duration_secs) as usize;

        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Sine,
            frequency: freq,
            amplitude: 1.0,
            sample_rate,
            ..Default::default()
        });

        let samples: Vec<f32> = (0..num_samples).map(|_| osc.next_sample()).collect();
        let crossings = count_zero_crossings(&samples);

        // Each cycle has one positive-going zero crossing
        let expected = freq as usize;
        let tolerance = 2; // Allow +-2 crossings for edge effects
        assert!(
            (crossings as i32 - expected as i32).unsigned_abs() <= tolerance as u32,
            "Expected ~{} crossings for {} Hz, got {}",
            expected,
            freq,
            crossings
        );
    }

    // --- Sine: peak amplitude matches configured amplitude ---
    #[test]
    fn test_sine_amplitude() {
        let amplitude = 0.6;
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Sine,
            frequency: 440.0,
            amplitude,
            sample_rate: 44100.0,
            ..Default::default()
        });

        let samples: Vec<f32> = (0..4410).map(|_| osc.next_sample()).collect();
        let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        assert!(
            (peak - amplitude).abs() < 0.02,
            "Peak should be ~{}, got {}",
            amplitude,
            peak
        );
    }

    // --- Square: alternates between +amplitude and -amplitude ---
    #[test]
    fn test_square_wave_values() {
        let amplitude = 0.8;
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Square,
            frequency: 100.0,
            amplitude,
            sample_rate: 44100.0,
            ..Default::default()
        });

        let samples: Vec<f32> = (0..4410).map(|_| osc.next_sample()).collect();

        // Every sample should be either +amplitude or -amplitude
        for (i, &s) in samples.iter().enumerate() {
            assert!(
                (s - amplitude).abs() < 0.001 || (s + amplitude).abs() < 0.001,
                "Square sample {} should be +/-{}, got {}",
                i,
                amplitude,
                s
            );
        }
    }

    // --- Sawtooth: ramps from -1 to +1 within each cycle ---
    #[test]
    fn test_sawtooth_ramp() {
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Sawtooth,
            frequency: 100.0,
            amplitude: 1.0,
            sample_rate: 44100.0,
            ..Default::default()
        });

        let samples: Vec<f32> = (0..4410).map(|_| osc.next_sample()).collect();

        // Check that sawtooth reaches near +1 and -1
        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(max > 0.95, "Sawtooth max should be near 1.0, got {}", max);
        assert!(min < -0.95, "Sawtooth min should be near -1.0, got {}", min);
    }

    // --- Triangle: peak amplitude and symmetry ---
    #[test]
    fn test_triangle_wave_shape() {
        let amplitude = 1.0;
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Triangle,
            frequency: 100.0,
            amplitude,
            sample_rate: 44100.0,
            ..Default::default()
        });

        let samples: Vec<f32> = (0..4410).map(|_| osc.next_sample()).collect();

        let max = samples.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min = samples.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!(
            (max - amplitude).abs() < 0.05,
            "Triangle max should be ~{}, got {}",
            amplitude,
            max
        );
        assert!(
            (min + amplitude).abs() < 0.05,
            "Triangle min should be ~-{}, got {}",
            amplitude,
            min
        );
    }

    // --- Noise: output is within amplitude range and has variance ---
    #[test]
    fn test_noise_range_and_variance() {
        let amplitude = 0.5;
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Noise,
            frequency: 440.0,
            amplitude,
            sample_rate: 44100.0,
            ..Default::default()
        });

        let samples: Vec<f32> = (0..10000).map(|_| osc.next_sample()).collect();

        // All samples within range
        for &s in &samples {
            assert!(
                s.abs() <= amplitude + 0.001,
                "Noise sample {} out of range",
                s
            );
        }

        // Should have variance (not all the same value)
        let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        let variance: f32 =
            samples.iter().map(|s| (s - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        assert!(
            variance > 0.01,
            "Noise should have variance, got {}",
            variance
        );
    }

    // --- set_frequency changes pitch ---
    #[test]
    fn test_set_frequency_changes_pitch() {
        let sample_rate = 48000.0;
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Sine,
            frequency: 200.0,
            amplitude: 1.0,
            sample_rate,
            ..Default::default()
        });

        let samples_200: Vec<f32> = (0..48000).map(|_| osc.next_sample()).collect();
        let crossings_200 = count_zero_crossings(&samples_200);

        osc.reset_phase();
        osc.set_frequency(400.0);
        let samples_400: Vec<f32> = (0..48000).map(|_| osc.next_sample()).collect();
        let crossings_400 = count_zero_crossings(&samples_400);

        // 400 Hz should have ~2x the crossings of 200 Hz
        let ratio = crossings_400 as f32 / crossings_200 as f32;
        assert!(
            (ratio - 2.0).abs() < 0.05,
            "Frequency doubling should double crossings: ratio={}",
            ratio
        );
    }

    // --- set_sample_rate preserves frequency ---
    #[test]
    fn test_set_sample_rate_preserves_frequency() {
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Sine,
            frequency: 440.0,
            amplitude: 1.0,
            sample_rate: 44100.0,
            ..Default::default()
        });

        osc.set_sample_rate(96000.0);

        // Generate 1 second at new sample rate
        let samples: Vec<f32> = (0..96000).map(|_| osc.next_sample()).collect();
        let crossings = count_zero_crossings(&samples);

        assert!(
            (crossings as i32 - 440).unsigned_abs() <= 2,
            "After sample rate change, frequency should still be ~440 Hz, got {} crossings",
            crossings
        );
    }

    // --- Phase reset ---
    #[test]
    fn test_reset_phase_restarts_waveform() {
        let mut osc = Oscillator::new(OscillatorConfig {
            waveform: Waveform::Sine,
            frequency: 440.0,
            amplitude: 1.0,
            sample_rate: 44100.0,
            ..Default::default()
        });

        let first_sample = osc.next_sample();
        // Advance some samples
        for _ in 0..1000 {
            osc.next_sample();
        }
        osc.reset_phase();
        let after_reset = osc.next_sample();

        assert!(
            (first_sample - after_reset).abs() < 0.001,
            "After reset, first sample should match: {} vs {}",
            first_sample,
            after_reset
        );
    }

    // --- MIDI to frequency ---
    #[test]
    fn test_midi_to_frequency_known_values() {
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.01); // A4
        assert!((midi_to_frequency(60) - 261.63).abs() < 0.1); // C4
        assert!((midi_to_frequency(57) - 220.0).abs() < 0.1); // A3
        assert!((midi_to_frequency(81) - 880.0).abs() < 0.1); // A5
                                                              // Octave relationship
        let ratio = midi_to_frequency(72) / midi_to_frequency(60);
        assert!((ratio - 2.0).abs() < 0.001, "Octave should be 2:1 ratio");
    }

    // --- Frequency to MIDI roundtrip ---
    #[test]
    fn test_frequency_midi_roundtrip() {
        for note in [36, 48, 60, 69, 72, 84, 96] {
            let freq = midi_to_frequency(note);
            let back = frequency_to_midi(freq);
            assert!(
                (back - note as f32).abs() < 0.01,
                "Roundtrip failed for note {}: got {}",
                note,
                back
            );
        }
    }

    // --- next_samples batch matches individual ---
    #[test]
    fn test_next_samples_matches_individual() {
        let config = OscillatorConfig {
            waveform: Waveform::Sawtooth,
            frequency: 440.0,
            amplitude: 0.7,
            sample_rate: 44100.0,
            ..Default::default()
        };

        let mut osc1 = Oscillator::new(config.clone());
        let individual: Vec<f32> = (0..256).map(|_| osc1.next_sample()).collect();

        let mut osc2 = Oscillator::new(config);
        let batch = osc2.next_samples(256);

        for (i, (a, b)) in individual.iter().zip(batch.iter()).enumerate() {
            assert!((a - b).abs() < 1e-6, "Mismatch at {}: {} vs {}", i, a, b);
        }
    }
}
