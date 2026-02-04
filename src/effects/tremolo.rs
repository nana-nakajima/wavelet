//!
//! Tremolo Effect Module
//!
//! Amplitude modulation effect using LFO-controlled gain reduction
//!

use crate::lfo::{Lfo, LfoConfig, LfoRate};
use crate::oscillator::Waveform;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Tremolo configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TremoloConfig {
    /// LFO rate in Hz (0.1 - 20.0)
    pub rate: f64,
    /// Modulation depth 0.0 - 1.0
    pub depth: f64,
    /// LFO waveform shape
    pub waveform: TremoloWaveform,
    /// Stereo width 0.0 - 1.0 (0 = mono, 1 = full stereo)
    pub stereo_width: f64,
    /// Mix ratio 0.0 - 1.0 (dry/wet)
    pub mix: f64,
    /// Enabled state
    pub enabled: bool,
}

impl Default for TremoloConfig {
    fn default() -> Self {
        Self {
            rate: 4.0,
            depth: 0.5,
            waveform: TremoloWaveform::Sine,
            stereo_width: 0.0,
            mix: 1.0,
            enabled: true,
        }
    }
}

/// Tremolo waveform types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TremoloWaveform {
    Sine,
    Triangle,
    Square,
    SawtoothUp,
    SawtoothDown,
    SampleAndHold,
}

impl From<TremoloWaveform> for Waveform {
    fn from(val: TremoloWaveform) -> Self {
        match val {
            TremoloWaveform::Sine => Waveform::Sine,
            TremoloWaveform::Triangle => Waveform::Triangle,
            TremoloWaveform::Square => Waveform::Square,
            TremoloWaveform::SawtoothUp => Waveform::Sawtooth,
            TremoloWaveform::SawtoothDown => Waveform::Sawtooth,
            TremoloWaveform::SampleAndHold => Waveform::Sine, // Approximate with Sine
        }
    }
}

/// Tremolo effect processor
#[derive(Debug)]
pub struct Tremolo {
    config: TremoloConfig,
    lfo_left: Lfo,
    lfo_right: Lfo,
}

impl Tremolo {
    /// Create new Tremolo instance
    pub fn new(_sample_rate: f64) -> Self {
        // Configure LFOs
        let config = LfoConfig::default();
        let mut lfo_left = Lfo::with_config(config);
        let mut lfo_right = Lfo::with_config(config);

        lfo_left.set_depth(1.0);
        lfo_right.set_depth(1.0);

        Self {
            config: TremoloConfig::default(),
            lfo_left,
            lfo_right,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TremoloConfig, _sample_rate: f64) -> Self {
        // Set LFO parameters
        let lfo_config = LfoConfig {
            rate: LfoRate::Hertz(config.rate as f32),
            waveform: config.waveform.into(),
            ..Default::default()
        };
        let mut lfo_left = Lfo::with_config(lfo_config);
        let mut lfo_right = Lfo::with_config(lfo_config);

        lfo_left.set_depth(1.0);
        lfo_right.set_depth(1.0);

        // Set stereo phase offset
        lfo_right.reset_phase((config.stereo_width * PI as f64) as f32);

        Self {
            config,
            lfo_left,
            lfo_right,
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &TremoloConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: TremoloConfig) {
        self.config = config;

        // Update LFO parameters
        self.lfo_left.set_rate_hz(self.config.rate as f32);
        self.lfo_left.set_waveform(self.config.waveform.into());

        self.lfo_right.set_rate_hz(self.config.rate as f32);
        self.lfo_right.set_waveform(self.config.waveform.into());
        self.lfo_right.reset_phase((self.config.stereo_width * PI as f64) as f32);
    }

    /// Set LFO rate
    pub fn set_rate(&mut self, rate: f64) {
        self.config.rate = rate.clamp(0.1, 20.0);
        self.lfo_left.set_rate_hz(self.config.rate as f32);
        self.lfo_right.set_rate_hz(self.config.rate as f32);
    }

    /// Set modulation depth
    pub fn set_depth(&mut self, depth: f64) {
        self.config.depth = depth.clamp(0.0, 1.0);
    }

    /// Set waveform
    pub fn set_waveform(&mut self, waveform: TremoloWaveform) {
        self.config.waveform = waveform;
        self.lfo_left.set_waveform(waveform.into());
        self.lfo_right.set_waveform(waveform.into());
    }

    /// Set stereo width
    pub fn set_stereo_width(&mut self, width: f64) {
        self.config.stereo_width = width.clamp(0.0, 1.0);
        self.lfo_right.reset_phase((self.config.stereo_width * PI as f64) as f32);
    }

    /// Set mix ratio
    pub fn set_mix(&mut self, mix: f64) {
        self.config.mix = mix.clamp(0.0, 1.0);
    }

    /// Enable/disable
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Process stereo audio buffer
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(left.len(), right.len());

        if !self.config.enabled {
            return;
        }

        let depth = self.config.depth as f32;
        let mix = self.config.mix as f32;

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            // Get LFO values (range -1.0 to 1.0)
            let lfo_l = self.lfo_left.process();
            let lfo_r = self.lfo_right.process();

            // Convert to modulation factors (0.0 to 1.0 range with depth control)
            let mod_l = 1.0 - depth * (lfo_l * 0.5 + 0.5);
            let mod_r = 1.0 - depth * (lfo_r * 0.5 + 0.5);

            // Apply tremolo with mix
            let dry_l = *l;
            let dry_r = *r;

            *l = dry_l * (mod_l * mix + (1.0 - mix));
            *r = dry_r * (mod_r * mix + (1.0 - mix));
        }
    }

    /// Process mono audio
    pub fn process(&mut self, input: f32) -> f32 {
        if !self.config.enabled {
            return input;
        }

        let depth = self.config.depth as f32;
        let mix = self.config.mix as f32;

        // Get LFO value (range -1.0 to 1.0)
        let lfo = self.lfo_left.process();

        // Convert to modulation factor
        let mod_factor = 1.0 - depth * (lfo * 0.5 + 0.5);

        // Apply tremolo with mix
        input * (mod_factor * mix + (1.0 - mix))
    }

    /// Reset LFO phase
    pub fn reset(&mut self) {
        self.lfo_left.reset();
        self.lfo_right.reset();
        self.lfo_right.reset_phase((self.config.stereo_width * PI as f64) as f32);
    }

    /// Get current LFO values for both channels
    pub fn get_lfo_values(&self) -> (f32, f32) {
        (self.lfo_left.value(), self.lfo_right.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tremolo_creation() {
        let tremolo = Tremolo::new(44100.0);
        let config = tremolo.config();

        assert_eq!(config.rate, 4.0);
        assert_eq!(config.depth, 0.5);
        assert_eq!(config.mix, 1.0);
        assert!(config.enabled);
    }

    #[test]
    fn test_tremolo_default_config() {
        let config = TremoloConfig::default();

        assert_eq!(config.rate, 4.0);
        assert_eq!(config.depth, 0.5);
        assert_eq!(config.waveform, TremoloWaveform::Sine);
        assert_eq!(config.stereo_width, 0.0);
        assert_eq!(config.mix, 1.0);
        assert!(config.enabled);
    }

    #[test]
    fn test_tremolo_process_stereo() {
        let mut tremolo = Tremolo::new(44100.0);
        let mut left = [1.0; 1024];
        let mut right = [1.0; 1024];

        tremolo.process_stereo(&mut left, &mut right);
        // Should process without error
    }

    #[test]
    fn test_tremolo_disabled() {
        let mut tremolo = Tremolo::new(44100.0);
        tremolo.set_enabled(false);

        let mut left = [1.0; 1024];
        let mut right = [1.0; 1024];
        let left_orig = left.clone();
        let right_orig = right.clone();

        tremolo.process_stereo(&mut left, &mut right);

        // Should be unchanged when disabled
        for (l, lo) in left.iter().zip(left_orig.iter()) {
            assert!((l - lo).abs() < 0.001);
        }
        for (r, ro) in right.iter().zip(right_orig.iter()) {
            assert!((r - ro).abs() < 0.001);
        }
    }

    #[test]
    fn test_tremolo_rate_parameter() {
        let mut tremolo = Tremolo::new(44100.0);

        tremolo.set_rate(10.0);
        assert!((tremolo.config().rate - 10.0).abs() < 0.001);

        tremolo.set_rate(0.0); // Should clamp
        assert!((tremolo.config().rate - 0.1).abs() < 0.001);

        tremolo.set_rate(100.0); // Should clamp
        assert!((tremolo.config().rate - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_tremolo_depth_parameter() {
        let mut tremolo = Tremolo::new(44100.0);

        tremolo.set_depth(0.8);
        assert!((tremolo.config().depth - 0.8).abs() < 0.001);

        tremolo.set_depth(-0.5); // Should clamp
        assert!((tremolo.config().depth - 0.0).abs() < 0.001);

        tremolo.set_depth(2.0); // Should clamp
        assert!((tremolo.config().depth - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tremolo_waveforms() {
        let waveforms = [
            TremoloWaveform::Sine,
            TremoloWaveform::Triangle,
            TremoloWaveform::Square,
            TremoloWaveform::SawtoothUp,
            TremoloWaveform::SawtoothDown,
            TremoloWaveform::SampleAndHold,
        ];

        for waveform in waveforms {
            let mut tremolo = Tremolo::new(44100.0);
            tremolo.set_waveform(waveform);

            assert_eq!(tremolo.config().waveform, waveform);
        }
    }

    #[test]
    fn test_tremolo_stereo_width() {
        let mut tremolo = Tremolo::new(44100.0);

        tremolo.set_stereo_width(1.0);
        assert!((tremolo.config().stereo_width - 1.0).abs() < 0.001);

        tremolo.set_stereo_width(-0.5); // Should clamp
        assert!((tremolo.config().stereo_width - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_tremolo_reset() {
        let mut tremolo = Tremolo::new(44100.0);

        // Process some samples
        let mut left = [1.0; 100];
        let mut right = [1.0; 100];
        tremolo.process_stereo(&mut left, &mut right);

        // Reset
        tremolo.reset();

        // Should not crash
        let mut left = [1.0; 100];
        let mut right = [1.0; 100];
        tremolo.process_stereo(&mut left, &mut right);
    }

    #[test]
    fn test_tremolo_mix_parameter() {
        let mut tremolo = Tremolo::new(44100.0);
        tremolo.set_mix(0.5);

        assert!((tremolo.config().mix - 0.5).abs() < 0.001);

        // Dry/wet mix - when mix is 0, should be dry
        tremolo.set_mix(0.0);

        let mut left = [1.0; 1024];
        let mut right = [1.0; 1024];
        let left_orig = left.clone();
        let right_orig = right.clone();

        tremolo.process_stereo(&mut left, &mut right);

        // With mix=0, should be mostly unchanged
        for (l, lo) in left.iter().zip(left_orig.iter()) {
            assert!((l - lo).abs() < 0.1);
        }
        for (r, ro) in right.iter().zip(right_orig.iter()) {
            assert!((r - ro).abs() < 0.1);
        }
    }

    #[test]
    fn test_tremolo_with_config() {
        let config = TremoloConfig {
            rate: 8.0,
            depth: 0.75,
            waveform: TremoloWaveform::Triangle,
            stereo_width: 0.5,
            mix: 0.8,
            enabled: true,
        };

        let tremolo = Tremolo::with_config(config, 44100.0);
        let retrieved = tremolo.config();

        assert!((retrieved.rate - 8.0).abs() < 0.001);
        assert!((retrieved.depth - 0.75).abs() < 0.001);
        assert_eq!(retrieved.waveform, TremoloWaveform::Triangle);
        assert!((retrieved.stereo_width - 0.5).abs() < 0.001);
        assert!((retrieved.mix - 0.8).abs() < 0.001);
        assert!(retrieved.enabled);
    }

    #[test]
    fn test_tremolo_different_sample_rates() {
        let rates = [44100.0, 48000.0, 96000.0];

        for &rate in &rates {
            let mut tremolo = Tremolo::new(rate as f64);
            let mut left = [1.0; 1024];
            let mut right = [1.0; 1024];

            tremolo.process_stereo(&mut left, &mut right);
            // Should process correctly at any sample rate
        }
    }

    #[test]
    fn test_tremolo_process_mono() {
        let mut tremolo = Tremolo::new(44100.0);

        let input = 0.5;
        let output = tremolo.process(input);

        // Should process without error
        assert!(output.abs() <= 1.0);
    }
}
