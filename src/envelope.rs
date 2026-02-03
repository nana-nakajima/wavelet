//! Envelope Module
//!
//! This module provides envelope generator implementations for controlling
//! the amplitude and modulation of synthesizer parameters over time.

#![allow(dead_code)] // Reserve envelope fields for future features

/// Enumeration of possible envelope stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeStage {
    /// Envelope is idle, at zero level
    Idle,

    /// Initial delay before attack phase
    Delay,

    /// Rising to peak level
    Attack,

    /// Falling to sustain level
    Decay,

    /// Holding at sustain level
    Sustain,

    /// Releasing back to zero
    Release,

    /// Envelope has finished
    Finished,
}

/// Configuration for envelope timing and levels.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeConfig {
    /// Attack time in seconds
    pub attack: f32,

    /// Decay time in seconds
    pub decay: f32,

    /// Sustain level (0.0 to 1.0)
    pub sustain: f32,

    /// Release time in seconds
    pub release: f32,

    /// Initial delay time in seconds (optional)
    pub delay: f32,

    /// Hold time at sustain level (optional)
    pub sustain_hold: f32,

    /// Peak level for attack/decay (typically 1.0)
    pub peak: f32,

    /// Sample rate for timing calculations
    pub sample_rate: f32,
}

impl Default for EnvelopeConfig {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.2,
            sustain: 0.7,
            release: 0.3,
            delay: 0.0,
            sustain_hold: 0.0,
            peak: 1.0,
            sample_rate: 44100.0,
        }
    }
}

/// ADSR (Attack, Decay, Sustain, Release) envelope generator.
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    /// Current envelope stage
    stage: EnvelopeStage,

    /// Current envelope level
    current_level: f32,

    /// Envelope parameters in samples
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    delay: f32,
    sustain_hold: f32,
    peak: f32,

    /// Timing state
    stage_time: f32,
    sample_rate: f32,
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope with default configuration.
    pub fn new() -> Self {
        Self::with_config(EnvelopeConfig::default())
    }

    /// Creates a new ADSR envelope with custom configuration.
    pub fn with_config(config: EnvelopeConfig) -> Self {
        let samples_per_second = config.sample_rate;

        Self {
            stage: EnvelopeStage::Idle,
            current_level: 0.0,
            attack: config.attack * samples_per_second,
            decay: config.decay * samples_per_second,
            sustain: config.sustain,
            release: config.release * samples_per_second,
            delay: config.delay * samples_per_second,
            sustain_hold: config.sustain_hold * samples_per_second,
            peak: config.peak,
            stage_time: 0.0,
            sample_rate: config.sample_rate,
        }
    }

    /// Triggers the envelope (note on event).
    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Delay;
        self.stage_time = 0.0;
    }

    /// Releases the envelope (note off event).
    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle && self.stage != EnvelopeStage::Release {
            self.stage = EnvelopeStage::Release;
            self.stage_time = 0.0;
        }
    }

    /// Processes one sample of the envelope.
    pub fn process(&mut self) -> f32 {
        self.advance_stage();
        self.current_level
    }

    /// Processes multiple envelope samples.
    pub fn process_samples(&mut self, count: usize) -> Vec<f32> {
        (0..count).map(|_| self.process()).collect()
    }

    /// Gets the current envelope stage.
    pub fn stage(&self) -> EnvelopeStage {
        self.stage
    }

    /// Checks if the envelope is still active (not idle or finished).
    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle && self.stage != EnvelopeStage::Finished
    }

    /// Resets the envelope to idle state.
    pub fn reset(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.current_level = 0.0;
        self.stage_time = 0.0;
    }

    /// Sets the attack time.
    pub fn set_attack(&mut self, attack: f32) {
        self.attack = attack * self.sample_rate;
    }

    /// Sets the decay time.
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay * self.sample_rate;
    }

    /// Sets the sustain level.
    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain.clamp(0.0, 1.0);
    }

    /// Sets the release time.
    pub fn set_release(&mut self, release: f32) {
        self.release = release * self.sample_rate;
    }

    /// Internal method to advance the envelope stage.
    fn advance_stage(&mut self) {
        self.stage_time += 1.0;

        match self.stage {
            EnvelopeStage::Idle => {
                self.current_level = 0.0;
            }

            EnvelopeStage::Delay => {
                if self.stage_time >= self.delay {
                    self.stage = EnvelopeStage::Attack;
                    self.stage_time = 0.0;
                }
            }

            EnvelopeStage::Attack => {
                if self.attack > 0.0 {
                    self.current_level += (self.peak - 0.0) / self.attack;
                } else {
                    self.current_level = self.peak;
                }

                if self.current_level >= self.peak {
                    self.current_level = self.peak;
                    self.stage = EnvelopeStage::Decay;
                    self.stage_time = 0.0;
                }
            }

            EnvelopeStage::Decay => {
                if self.decay > 0.0 {
                    self.current_level += (self.sustain - self.peak) / self.decay;
                } else {
                    self.current_level = self.sustain;
                }

                if self.current_level <= self.sustain {
                    self.current_level = self.sustain;
                    if self.sustain_hold > 0.0 {
                        self.stage = EnvelopeStage::Sustain;
                    } else {
                        self.stage = EnvelopeStage::Sustain;
                    }
                    self.stage_time = 0.0;
                }
            }

            EnvelopeStage::Sustain => {
                // Sustain level is held until note off
                self.current_level = self.sustain;

                if self.stage_time >= self.sustain_hold {
                    // Stay at sustain, waiting for note_off
                }
            }

            EnvelopeStage::Release => {
                if self.release > 0.0 {
                    self.current_level += (0.0 - self.sustain) / self.release;
                } else {
                    self.current_level = 0.0;
                }

                if self.current_level <= 0.0 {
                    self.current_level = 0.0;
                    self.stage = EnvelopeStage::Finished;
                }
            }

            EnvelopeStage::Finished => {
                self.current_level = 0.0;
            }
        }
    }
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

/// Envelope trait for polymorphic envelope usage.
pub trait Envelope {
    /// Process one sample.
    fn process(&mut self) -> f32;

    /// Trigger the envelope.
    fn trigger(&mut self);

    /// Release the envelope.
    fn release(&mut self);

    /// Check if envelope is active.
    fn is_active(&self) -> bool;
}

impl Envelope for AdsrEnvelope {
    fn process(&mut self) -> f32 {
        self.process()
    }

    fn trigger(&mut self) {
        self.note_on();
    }

    fn release(&mut self) {
        self.note_off();
    }

    fn is_active(&self) -> bool {
        self.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_default() {
        let env = AdsrEnvelope::new();
        assert_eq!(env.stage(), EnvelopeStage::Idle);
        assert!(!env.is_active());
    }

    #[test]
    fn test_envelope_note_on() {
        let mut env = AdsrEnvelope::new();
        env.note_on();
        assert_eq!(env.stage(), EnvelopeStage::Delay);
    }

    #[test]
    fn test_envelope_note_off() {
        let mut env = AdsrEnvelope::new();
        env.note_on();
        env.note_off();
        assert_eq!(env.stage(), EnvelopeStage::Release);
    }

    #[test]
    fn test_envelope_process() {
        let config = EnvelopeConfig {
            attack: 0.001,
            decay: 0.001,
            sustain: 0.5,
            release: 0.001,
            sample_rate: 1000.0,
            ..Default::default()
        };

        let mut env = AdsrEnvelope::with_config(config);
        env.note_on();

        // After attack, should be at peak
        for _ in 0..2 {
            env.process();
        }

        // Should move through stages
        let level = env.process();
        assert!(level >= 0.0);
    }

    #[test]
    fn test_envelope_reset() {
        let mut env = AdsrEnvelope::new();
        env.note_on();
        env.reset();
        assert_eq!(env.stage(), EnvelopeStage::Idle);
    }
}
