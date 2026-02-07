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

use crate::oscillator::{Oscillator, OscillatorConfig, OversampleFactor, Waveform};

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
            oversample_factor: OversampleFactor::None,
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
    pub fn reset_phase(&mut self, _phase: f32) {
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

    // --- Helper: count positive-going zero crossings ---
    fn count_zero_crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count()
    }

    // --- LFO period matches configured rate ---
    #[test]
    fn test_lfo_period_matches_rate() {
        let rate = 5.0; // 5 Hz
        let sample_rate = 1000.0;
        let duration_secs = 2.0;
        let num_samples = (sample_rate * duration_secs) as usize;

        let mut lfo = Lfo::with_config(LfoConfig {
            rate: LfoRate::Hertz(rate),
            waveform: Waveform::Sine,
            depth: 1.0,
            sample_rate,
            ..Default::default()
        });

        let samples: Vec<f32> = (0..num_samples).map(|_| lfo.process()).collect();
        let crossings = count_zero_crossings(&samples);

        let expected = (rate * duration_secs) as usize;
        assert!(
            (crossings as i32 - expected as i32).unsigned_abs() <= 1,
            "Expected ~{} cycles for {} Hz over {}s, got {}",
            expected,
            rate,
            duration_secs,
            crossings
        );
    }

    // --- Depth scales output ---
    #[test]
    fn test_depth_scales_output() {
        let sample_rate = 1000.0;

        let mut lfo_full = Lfo::with_config(LfoConfig {
            rate: LfoRate::Hertz(2.0),
            waveform: Waveform::Sine,
            depth: 1.0,
            sample_rate,
            ..Default::default()
        });

        let mut lfo_half = Lfo::with_config(LfoConfig {
            rate: LfoRate::Hertz(2.0),
            waveform: Waveform::Sine,
            depth: 0.5,
            sample_rate,
            ..Default::default()
        });

        let full_samples: Vec<f32> = (0..1000).map(|_| lfo_full.process()).collect();
        let half_samples: Vec<f32> = (0..1000).map(|_| lfo_half.process()).collect();

        let full_peak = full_samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let half_peak = half_samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        assert!(
            (full_peak - 1.0).abs() < 0.05,
            "Full depth peak should be ~1.0, got {}",
            full_peak
        );
        assert!(
            (half_peak - 0.5).abs() < 0.05,
            "Half depth peak should be ~0.5, got {}",
            half_peak
        );
    }

    // --- Zero depth produces no modulation ---
    #[test]
    fn test_zero_depth_no_modulation() {
        let mut lfo = Lfo::with_config(LfoConfig {
            rate: LfoRate::Hertz(5.0),
            waveform: Waveform::Sine,
            depth: 0.0,
            sample_rate: 1000.0,
            ..Default::default()
        });

        for _ in 0..500 {
            let val = lfo.process();
            assert_eq!(val, 0.0, "Zero depth should produce 0.0, got {}", val);
        }
    }

    // --- Delay suppresses output ---
    #[test]
    fn test_delay_suppresses_output() {
        let sample_rate = 1000.0;
        let delay_samples = 100;

        let mut lfo = Lfo::with_config(LfoConfig {
            rate: LfoRate::Hertz(5.0),
            waveform: Waveform::Sine,
            depth: 1.0,
            delay_samples,
            sample_rate,
            ..Default::default()
        });

        // During delay, output should be 0
        for i in 0..delay_samples {
            let val = lfo.process();
            assert_eq!(val, 0.0, "During delay (sample {}), should be 0", i);
        }

        // After delay, should produce non-zero output
        let mut found_nonzero = false;
        for _ in 0..100 {
            if lfo.process().abs() > 0.01 {
                found_nonzero = true;
                break;
            }
        }
        assert!(found_nonzero, "After delay, LFO should produce output");
    }

    // --- Different waveforms produce different shapes ---
    #[test]
    fn test_different_waveforms_differ() {
        let sample_rate = 1000.0;
        let num_samples = 500;

        let waveforms = [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
        ];
        let mut outputs: Vec<Vec<f32>> = Vec::new();

        for wf in &waveforms {
            let mut lfo = Lfo::with_config(LfoConfig {
                rate: LfoRate::Hertz(2.0),
                waveform: *wf,
                depth: 1.0,
                sample_rate,
                ..Default::default()
            });
            let samples: Vec<f32> = (0..num_samples).map(|_| lfo.process()).collect();
            outputs.push(samples);
        }

        // Each pair of waveforms should differ
        for i in 0..outputs.len() {
            for j in (i + 1)..outputs.len() {
                let diff: f32 = outputs[i]
                    .iter()
                    .zip(outputs[j].iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum::<f32>()
                    / num_samples as f32;
                assert!(
                    diff > 0.01,
                    "{:?} and {:?} should differ, avg diff={}",
                    waveforms[i],
                    waveforms[j],
                    diff
                );
            }
        }
    }

    // --- set_rate changes frequency ---
    #[test]
    fn test_set_rate_changes_frequency() {
        let sample_rate = 1000.0;
        let mut lfo = Lfo::with_config(LfoConfig {
            rate: LfoRate::Hertz(2.0),
            waveform: Waveform::Sine,
            depth: 1.0,
            sample_rate,
            ..Default::default()
        });

        let samples_2hz: Vec<f32> = (0..1000).map(|_| lfo.process()).collect();
        let crossings_2 = count_zero_crossings(&samples_2hz);

        lfo.reset();
        lfo.set_rate_hz(4.0);
        let samples_4hz: Vec<f32> = (0..1000).map(|_| lfo.process()).collect();
        let crossings_4 = count_zero_crossings(&samples_4hz);

        assert!(
            crossings_4 > crossings_2,
            "4 Hz should have more crossings than 2 Hz: {} vs {}",
            crossings_4,
            crossings_2
        );
    }

    // --- process_block matches individual ---
    #[test]
    fn test_process_block_matches_individual() {
        let config = LfoConfig {
            rate: LfoRate::Hertz(3.0),
            waveform: Waveform::Sine,
            depth: 0.8,
            sample_rate: 1000.0,
            ..Default::default()
        };

        let mut lfo1 = Lfo::with_config(config);
        let individual: Vec<f32> = (0..200).map(|_| lfo1.process()).collect();

        let mut lfo2 = Lfo::with_config(config);
        let block = lfo2.process_block(200);

        for (i, (a, b)) in individual.iter().zip(block.iter()).enumerate() {
            assert!((a - b).abs() < 1e-6, "Mismatch at {}: {} vs {}", i, a, b);
        }
    }

    // --- LfoRate conversions ---
    #[test]
    fn test_lfo_rate_conversions() {
        assert!((LfoRate::Hertz(440.0).to_hertz() - 440.0).abs() < 0.001);
        // MIDI note 69 = A4 = 440 Hz
        assert!((LfoRate::MidiNote(69).to_hertz() - 440.0).abs() < 0.1);
    }

    // --- value() getter ---
    #[test]
    fn test_value_getter_matches_process() {
        let mut lfo = Lfo::with_config(LfoConfig {
            rate: LfoRate::Hertz(5.0),
            waveform: Waveform::Sine,
            depth: 1.0,
            sample_rate: 1000.0,
            ..Default::default()
        });

        for _ in 0..50 {
            let processed = lfo.process();
            assert_eq!(lfo.value(), processed);
        }
    }
}
